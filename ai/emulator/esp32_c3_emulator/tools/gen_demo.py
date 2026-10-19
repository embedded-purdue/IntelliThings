#!/usr/bin/env python3
"""Generate the demo-home C tables: gen_demo.py <demo_home.json> <out.c>

Run by the build (main/CMakeLists.txt). Validates the entities and presets and stops the build
with a clear message if anything is wrong, so a typo in the JSON never reaches the board.
Without <out.c> it only validates and prints a summary.
"""

import json
import sys

MAX_ENTITIES = 80  # DEMO_MAX_ENTITIES in main/demo_home.h
TYPES = {"number": "DEMO_NUM", "bool": "DEMO_BOOL", "enum": "DEMO_ENUM"}
FILLS = {"normal": "FILL_NORMAL", "presence": "FILL_PRESENCE", "light": "FILL_LIGHT",
         "lock": "FILL_LOCK", "car": "FILL_CAR", "off": "FILL_OFF"}
DERIVES = {None: "DERIVE_NONE", "pm1": "DERIVE_PM1", "pm10": "DERIVE_PM10", "lux": "DERIVE_LUX",
           "distance": "DERIVE_DISTANCE"}
REQUIRED = ("home.time_of_day", "home.anyone_home")


class Bad(Exception):
    pass


def cstr(s):
    return '"' + s.replace("\\", "\\\\").replace('"', '\\"') + '"'


def cfloat(x):
    return repr(float(x)) + "f"


def expand(doc):
    areas, entities = [], []
    for a in doc["areas"]:
        ents = [dict(e) for e in doc["templates"][a["template"]]] if "template" in a else []
        ents += [dict(e) for e in a.get("entities", [])]
        for field, ov in a.get("override", {}).items():
            match = [e for e in ents if e["field"] == field]
            if not match:
                raise Bad(f"{a['key']}: override for unknown field {field}")
            match[0].update(ov)
        areas.append({"key": a["key"], "label": a["label"], "node_room": a.get("node_room", ""),
                      "sleep": bool(a.get("sleep_area", False))})
        for e in ents:
            e["area"] = len(areas) - 1
            e["key"] = a["key"] + "." + e["field"]
            entities.append(e)
    return areas, entities


def check_entity(e, seen_presence):
    k = e["key"]
    t = e.get("type")
    if t not in TYPES:
        raise Bad(f"{k}: type must be one of {list(TYPES)}")
    fill = e.get("fill", "normal")
    if fill not in FILLS:
        raise Bad(f"{k}: fill must be one of {list(FILLS)}")
    derive = e.get("derive")
    if derive not in DERIVES:
        raise Bad(f"{k}: derive must be one of {[d for d in DERIVES if d]}")
    if t == "number":
        lo, hi = e["caps"]
        if not lo < hi:
            raise Bad(f"{k}: caps must be [low, high]")
        if derive is None:
            nlo, nhi = e["normal"]
            if not lo <= nlo <= nhi <= hi:
                raise Bad(f"{k}: normal range must sit inside the caps")
    elif derive:
        raise Bad(f"{k}: only numbers can be derived")
    if t == "bool":
        if len(e.get("labels", [])) != 2:
            raise Bad(f"{k}: a bool needs two labels [false, true]")
        if not 0 <= e.get("normal_p", 0.5) <= 1:
            raise Bad(f"{k}: normal_p must be 0..1")
    if t == "enum":
        opts = e.get("options", [])
        if not 2 <= len(opts) <= 16 or len(set(opts)) != len(opts):
            raise Bad(f"{k}: an enum needs 2..16 distinct options")
        if not set(e.get("normal", opts)) <= set(opts):
            raise Bad(f"{k}: normal options must be among the options")
    if fill == "light" and e["area"] not in seen_presence:
        raise Bad(f"{k}: fill 'light' needs a presence entity earlier in the same area")
    if fill == "presence":
        seen_presence.add(e["area"])
    if derive in ("pm1", "pm10", "lux", "distance"):
        e["_needs"] = {"pm1": "pm2_5_ugm3", "pm10": "pm2_5_ugm3", "lux": "light_on",
                       "distance": "presence"}[derive]


def encode_value(e, v, where):
    t = e["type"]
    if t == "number":
        if isinstance(v, bool) or not isinstance(v, (int, float)):
            raise Bad(f"{where}: {e['key']} needs a number")
        lo, hi = e["caps"]
        if not lo <= v <= hi:
            raise Bad(f"{where}: {e['key']} = {v} is outside its caps {lo}..{hi}")
        return float(v)
    if t == "bool":
        if not isinstance(v, bool):
            raise Bad(f"{where}: {e['key']} needs true or false")
        return 1.0 if v else 0.0
    if v not in e["options"]:
        raise Bad(f"{where}: {e['key']} must be one of {e['options']}")
    return float(e["options"].index(v))


def generate(doc):
    areas, entities = expand(doc)
    if len(entities) > MAX_ENTITIES:
        raise Bad(f"{len(entities)} entities; the firmware allows {MAX_ENTITIES}")
    keys = [e["key"] for e in entities]
    if len(set(keys)) != len(keys):
        raise Bad("duplicate entity keys")
    for r in REQUIRED:
        if r not in keys:
            raise Bad(f"the fill rules need {r}")
    seen = set()
    for e in entities:
        check_entity(e, seen)
        need = e.get("_needs")
        if need and f"{areas[e['area']]['key']}.{need}" not in keys:
            raise Bad(f"{e['key']}: derive needs {need} in the same area")
    groups = [g["key"] for g in doc["groups"]]
    presets = doc["presets"]
    names = [p["name"] for p in presets]
    if len(set(names)) != len(names):
        raise Bad("duplicate preset names")
    index = {k: i for i, k in enumerate(keys)}

    out = ["// Generated by tools/gen_demo.py from demo/demo_home.json. Do not edit.",
           '#include "demo_home.h"', ""]
    out.append("const demo_area_t DEMO_AREAS[] = {")
    for a in areas:
        out.append(f"    {{{cstr(a['key'])}, {cstr(a['label'])}, {cstr(a['node_room'])}, "
                   f"{'true' if a['sleep'] else 'false'}}},")
    out += ["};", f"const int DEMO_AREA_COUNT = {len(areas)};", ""]

    for i, e in enumerate(entities):
        opts = e.get("options") or e.get("labels")
        if opts:
            out.append(f"static const char *const OPT_{i}[] = {{{', '.join(cstr(o) for o in opts)}}};")
    out.append("")
    out.append("const demo_entity_t DEMO_ENTITIES[] = {")
    for i, e in enumerate(entities):
        t = e["type"]
        lo, hi = (e["caps"] if t == "number" else (0, len(e.get("options", [0, 1])) - 1))
        if t == "number":
            nlo, nhi = e.get("normal", e["caps"])
        elif t == "bool":
            nlo, nhi = e.get("normal_p", 0.5), 0
        else:
            nlo, nhi = 0, 0
        opts = e.get("options") or e.get("labels")
        nopts = len(opts) if opts else 0
        mask = 0
        if t == "enum":
            for o in e.get("normal", e["options"]):
                mask |= 1 << e["options"].index(o)
        out.append(
            f"    {{{cstr(e['key'])}, {cstr(e['field'])}, {cstr(e['label'])}, {cstr(e.get('unit', ''))}, "
            f"{e['area']}, {TYPES[t]}, {e.get('dec', 0)}, {FILLS[e.get('fill', 'normal')]}, "
            f"{DERIVES[e.get('derive')]}, {cfloat(lo)}, {cfloat(hi)}, {cfloat(nlo)}, {cfloat(nhi)}, "
            f"{'OPT_' + str(i) if opts else 'NULL'}, {nopts}, 0x{mask:x}}},")
    out += ["};", f"const int DEMO_ENTITY_COUNT = {len(entities)};", ""]

    out.append(f"const char *const DEMO_GROUPS[] = {{{', '.join(cstr(g) for g in groups)}}};")
    out.append("const char *const DEMO_GROUP_LABELS[] = {" +
               ", ".join(cstr(g["label"]) for g in doc["groups"]) + "};")
    out += [f"const int DEMO_GROUP_COUNT = {len(groups)};", ""]

    for i, p in enumerate(presets):
        where = f"preset {i + 1} \"{p['name']}\""
        if p.get("group") not in groups:
            raise Bad(f"{where}: group must be one of {groups}")
        sets = []
        for k, v in p["set"].items():
            if k not in index:
                raise Bad(f"{where}: unknown entity {k}")
            sets.append((index[k], encode_value(entities[index[k]], v, where)))
        out.append(f"static const demo_set_t SET_{i}[] = {{" +
                   ", ".join(f"{{{e}, {cfloat(v)}}}" for e, v in sets) + "};")
    out.append("const demo_preset_t DEMO_PRESETS[] = {")
    for i, p in enumerate(presets):
        out.append(f"    {{{cstr(p['name'])}, {groups.index(p['group'])}, SET_{i}, "
                   f"{len(p['set'])}}},")
    out += ["};", f"const int DEMO_PRESET_COUNT = {len(presets)};", ""]
    per_group = ", ".join("%s: %d" % (g, sum(p["group"] == g for p in presets)) for g in groups)
    summary = (f"demo home: {len(areas)} areas, {len(entities)} entities, {len(presets)} presets "
               f"({per_group})")
    return "\n".join(out), summary


def main():
    if len(sys.argv) not in (2, 3):
        sys.exit(__doc__)
    try:
        with open(sys.argv[1], encoding="utf-8") as f:
            doc = json.load(f)
        code, summary = generate(doc)
    except (Bad, KeyError, ValueError) as e:
        sys.exit(f"gen_demo.py: {sys.argv[1]}: {e}")
    if len(sys.argv) == 3:
        with open(sys.argv[2], "w", encoding="utf-8", newline="\n") as f:
            f.write(code)
    print(summary)


if __name__ == "__main__":
    main()
