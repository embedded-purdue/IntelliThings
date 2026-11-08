"""build dwin_set tables"""

import argparse
import json
import math
import shutil
import struct
import time
from dataclasses import dataclass, field
from pathlib import Path

from . import design as d
from . import formats as F
from . import glyphs as g
from . import pages as p
from . import sprites as sp
from . import ui

ROOT = Path(__file__).resolve().parent.parent
BUILD = ROOT / "build"
GENERATED_RS = ROOT / "firmware/companion-ui/src/generated.rs"
SECTOR = 256 * 1024
LAST_SECTOR = 63


# collect icon sets


@dataclass
class GlyphSetEntry:
    key: tuple
    styles: list  # shared styles
    gs: g.GlyphSet = None
    first: int = 0


@dataclass
class Icons:
    images: list = field(default_factory=list)  # icon images
    sprite_ids: dict = field(default_factory=dict)  # sprite ids
    sets: list = field(default_factory=list)  # glyph sets
    style_set: dict = field(default_factory=dict)  # style mapping


def collect_icons():
    icons = Icons()
    for s in sp.catalog():
        icons.sprite_ids[s.name] = len(icons.images)
        icons.images.append(s.image)
    merged = {}
    for st in ui.STYLES:
        key = (st.size, st.weight, st.fg, st.bg)
        if key not in merged:
            merged[key] = GlyphSetEntry(key, [], None)
            merged[key].chars = ""
        e = merged[key]
        e.styles.append(st.name)
        e.chars = "".join(sorted(set(e.chars) | set(st.chars)))
    for key, e in merged.items():
        size, weight, fg, bg = key
        e.gs = g.build_glyph_set(g.TextStyle("+".join(e.styles), size, weight, fg, bg, e.chars))
        e.first = len(icons.images)
        icons.images += [e.gs.glyphs[c].image for c in e.chars]
        icons.sets.append(e)
        for name in e.styles:
            icons.style_set[name] = e
    return icons


@dataclass
class Library:
    file_id: int
    first: int  # first icon id
    end: int
    data: bytes = b""

    @property
    def name(self):
        return f"{self.file_id}.ICL"  # icl filename


def split_libraries(icons):
    """split icon libraries"""
    edges = [0] + [e.first for e in icons.sets] + [len(icons.images)]
    blocks = [(a, b) for a, b in zip(edges, edges[1:]) if b > a]
    chunks = []
    for a, b in blocks:
        assert b - a <= ui.MAX_ICONS_PER_ICL
        if chunks and b - chunks[-1][0] <= ui.MAX_ICONS_PER_ICL:
            chunks[-1] = (chunks[-1][0], b)
        else:
            chunks.append((a, b))
    return chunks


# allocate vps


@dataclass
class Part:
    layer: str
    icl: int
    vp: int
    capacity: int
    first: int
    end: int


def allocate_vps(libs):
    vp = ui.VP_START
    parts = []
    for layer, slots, cap, cap_other in ui.LAYERS:
        for k, lib in enumerate(libs):
            for _ in range(slots if k == 0 else 1):
                c = cap if k == 0 else cap_other
                parts.append(Part(layer, lib.file_id, vp, c, lib.first, lib.end))
                vp += ui.layer_words(c)
        vp = (vp + 0xFF) & ~0xFF
    trend_vps = []
    for _ in range(2 * ui.TREND_ROWS):
        trend_vps.append(vp)
        vp += ui.TREND_STRIDE
    qr_vp = vp
    vp += ui.QR_WORDS
    scrim_vp = vp
    vp += 1
    # marker vps
    session_vp, signature_vp = vp, vp + 1
    return parts, trend_vps, qr_vp, scrim_vp, (session_vp, signature_vp), vp


# canvas renderer


class Canvas:
    def __init__(self, icons, parts):
        self.icons, self.parts = icons, parts
        self.items = {id(pt): [] for pt in parts}

    def sprite(self, layer, sprite_id, x, y):
        for pt in self.parts:
            if pt.layer == layer and pt.first <= sprite_id < pt.end and len(self.items[id(pt)]) < pt.capacity:
                self.items[id(pt)].append((x, y, sprite_id - pt.first))
                return
        raise KeyError(sprite_id)

    def text(self, layer, style, s, x, y, align="l"):
        e = self.icons.style_set[style]
        w = round(e.gs.measure(s))
        x0 = x - (w // 2 if align == "m" else w if align == "r" else 0)
        pen, top = float(x0), y - e.gs.ascent
        for ch in s:
            if ch not in e.gs.glyphs:
                continue
            gl = e.gs.glyphs[ch]
            if ch != " ":
                self.sprite(layer, e.first + e.chars.index(ch), round(pen) + gl.x_offset, top)
            pen += gl.advance
        return w

    def words(self):
        out = {}
        for pt in self.parts:
            items = self.items[id(pt)]
            assert len(items) <= pt.capacity
            block = [(pt.icl << 8) | 0x07, len(items)]
            for x, y, i in items:
                block += [x, y, i]
            block.append(0xFF00)
            for k, w in enumerate(block):
                out[pt.vp + k] = w
        return out


# page controls


def page_controls(name, parts, trend_vps, qr_vp, scrim_vp):
    recs = []

    def layer(layer_name):
        for pt in parts:
            if pt.layer == layer_name:
                recs.append(F.basic_graphics(pt.vp))

    layer("SKIN")
    if name == "SENSORS":
        for vp in trend_vps:
            recs.append(F.basic_graphics(vp))
    layer("MAIN")
    if name == "HOME":
        recs.append(F.brightness_area(scrim_vp, (0, 0, d.W - 1, d.H - 1)))
    layer("OVER")
    if name == "SETUP":
        x0, y0, size = p.SETUP_QR
        side = 45 * ui.QR_UNIT
        recs.append(F.qr_code(qr_vp, x0 + (size - side) // 2, y0 + (size - side) // 2, ui.QR_UNIT))
    return recs


# generate rust


def rust_ident(name):
    out = "".join(c if c.isalnum() else "_" for c in name).upper()
    return out if not out[0].isdigit() else "_" + out


def rust_str(s):
    return '"' + "".join(c if c not in '"\\' else "\\" + c for c in s) + '"'


def write_rust(icons, libs, parts, trend_vps, qr_vp, scrim_vp, marker_vps, vp_end):
    session_vp, signature_vp = marker_vps
    L = [
        "// generated file",
        "#![allow(dead_code)]",
        "",
        "use crate::text::GlyphStyle;",
        "",
        "/// basic graphics control",
        "pub struct Part {",
        "    pub icl: u8,",
        "    pub vp: u16,",
        "    pub capacity: u16,",
        "    pub first: u16,",
        "    pub end: u16,",
        "}",
        "",
        "pub mod page {",
    ]
    for i, (name, _) in enumerate(ui.PAGES):
        L.append(f"    pub const {name}: u16 = {i};")
    L += ["}", ""]
    for layer, *_ in ui.LAYERS:
        ps = [pt for pt in parts if pt.layer == layer]
        L.append(f"pub static {layer}: [Part; {len(ps)}] = [")
        for pt in ps:
            L.append(f"    Part {{ icl: {pt.icl}, vp: {pt.vp:#06x}, capacity: {pt.capacity}, first: {pt.first}, end: {pt.end} }},")
        L.append("];")
    L += [
        "",
        f"pub const TREND_ROWS: usize = {ui.TREND_ROWS};",
        f"pub const TREND_POINTS: usize = {ui.TREND_POINTS};",
        "/// trend polylines",
        f"pub const TREND_VP: [u16; {len(trend_vps)}] = [{', '.join(f'{v:#06x}' for v in trend_vps)}];",
        f"pub const QR_VP: u16 = {qr_vp:#06x};",
        "pub const QR_MAX_BYTES: usize = 150; // max payload",
        f"pub const SCRIM_VP: u16 = {scrim_vp:#06x};",
        "/// session marker",
        f"pub const SESSION_VP: u16 = {session_vp:#06x};",
        "/// signature marker",
        f"pub const SIGNATURE_VP: u16 = {signature_vp:#06x};",
        f"pub const SIGNATURE_VALUE: u16 = {ui.SIGNATURE_VALUE:#06x};",
        f"pub const VP_BASE: u16 = {ui.VP_START:#06x};",
        f"pub const VP_WORDS: usize = {vp_end - ui.VP_START:#x};",
        "",
        "/// room tiles",
        f"pub static ROOMS: [(i16, i16, i16, i16, bool); {len(p.ROOMS)}] = [",
        *[f"    ({x}, {y}, {w}, {h}, {str(node).lower()}), // {name.lower()}" for name, x, y, w, h, node in p.ROOMS],
        "];",
        f"pub const ROOM_NAMES: [&str; {len(p.ROOMS)}] = [{', '.join(rust_str(r[0]) for r in p.ROOMS)}];",
        "",
        "pub mod sprite {",
    ]
    for name, i in icons.sprite_ids.items():
        L.append(f"    pub const {rust_ident(name)}: u16 = {i};")
    L += ["}", "", "/// sprite dimensions",
          f"pub static SPRITE_SIZE: [(u16, u16); {len(icons.sprite_ids)}] = ["]
    for name, i in icons.sprite_ids.items():
        w, h = icons.images[i].size
        L.append(f"    ({w}, {h}),")
    L += ["];", ""]
    for k, e in enumerate(icons.sets):
        adv = [round(e.gs.glyphs[c].advance * 16) for c in e.chars]
        off = [e.gs.glyphs[c].x_offset for c in e.chars]
        asc = [e.chars.index(chr(c)) if chr(c) in e.chars else 255 for c in range(0x20, 0x80)]
        L.append(f"const CHARS_{k}: &str = {rust_str(e.chars)};")
        L.append(f"static ADV_{k}: [u16; {len(adv)}] = [{', '.join(map(str, adv))}];")
        L.append(f"static OFF_{k}: [i8; {len(off)}] = [{', '.join(map(str, off))}];")
        L.append(f"static ASCII_{k}: [u8; 96] = [{', '.join(map(str, asc))}];")
    L += ["", "pub mod style {", "    use super::*;", ""]
    for st in ui.STYLES:
        e = icons.style_set[st.name]
        k = icons.sets.index(e)
        L.append(
            f"    pub static {rust_ident(st.name)}: GlyphStyle = GlyphStyle {{ first: {e.first}, chars: CHARS_{k}, "
            f"ascii: &ASCII_{k}, advance: &ADV_{k}, x_offset: &OFF_{k}, ascent: {e.gs.ascent}, height: {e.gs.height} }};")
    L += ["}", ""]
    GENERATED_RS.write_text("\n".join(L) + "\n")


# build artifacts


def build(check=False):
    t0 = time.time()
    out = BUILD / "DWIN_SET"
    for folder in (out, BUILD / "preview"):
        if folder.exists():
            shutil.rmtree(folder)
        folder.mkdir(parents=True)

    print("rendering backgrounds")
    backgrounds = [fn() for _, fn in ui.PAGES]
    for (name, _), img in zip(ui.PAGES, backgrounds):
        img.save(BUILD / "preview" / f"page_{name.lower()}.png")
    bg_icl = F.write_icl(backgrounds)
    bg_sectors = math.ceil(len(bg_icl) / SECTOR)

    print("rendering sprites and glyphs")
    icons = collect_icons()
    chunks = split_libraries(icons)
    next_sector = ui.BG_ICL + bg_sectors
    libs = []
    for a, b in chunks:
        lib = Library(next_sector, a, b)
        lib.data = F.write_icl(icons.images[a:b])
        n = math.ceil(len(lib.data) / SECTOR)
        if lib.file_id + n - 1 > LAST_SECTOR:
            raise SystemExit(f"icon libraries don't fit in flash ({lib.file_id + n - 1} > {LAST_SECTOR})")
        next_sector += n
        libs.append(lib)

    parts, trend_vps, qr_vp, scrim_vp, marker_vps, vp_end = allocate_vps(libs)
    session_vp, signature_vp = marker_vps

    # page controls
    show = F.write_show_file([page_controls(name, parts, trend_vps, qr_vp, scrim_vp) for name, _ in ui.PAGES])

    # boot vp init
    canvas = Canvas(icons, parts)
    orb = icons.sprite_ids["boot_orb_idle_0"]
    ow, oh = icons.images[orb].size
    canvas.sprite("SKIN", orb, d.W // 2 - ow // 2, 138 - oh // 2)
    canvas.text("MAIN", "BOOT_DETAIL", "Waiting for the controller…", d.W // 2, 300, "m")
    init = canvas.words()
    for vp in trend_vps:
        init.update({vp: 0x0002, vp + 1: 0, vp + 2: d.rgb565(d.MUTED), vp + 3: 0xFF00})
    init[qr_vp] = 0xFFFF
    init[scrim_vp] = 100
    init[session_vp] = 0
    init[signature_vp] = ui.SIGNATURE_VALUE

    files = {
        "T5LCFG_800480.CFG": F.write_cfg(background_icl=ui.BG_ICL),
        "13TouchFile.bin": F.TOUCH_FILE_EMPTY,
        "14ShowFile.bin": show,
        "22_Config.bin": F.write_init_file(init),
        f"{ui.BG_ICL}.ICL": bg_icl,
    }
    for lib in libs:
        files[lib.name] = lib.data
    for name, data in files.items():
        (out / name).write_bytes(data)

    manifest = {
        "pages": [name for name, _ in ui.PAGES],
        "background_icl": f"{ui.BG_ICL}.ICL",
        "libraries": [{"file": lib.name, "id": lib.file_id, "first": lib.first, "end": lib.end} for lib in libs],
        "parts": [pt.__dict__ for pt in parts],
        "trend_vps": trend_vps,
        "qr_vp": qr_vp,
        "scrim_vp": scrim_vp,
        "session_vp": session_vp,
        "signature_vp": signature_vp,
        "vp_start": ui.VP_START,
        "vp_end": vp_end,
        "sprites": icons.sprite_ids,
        "styles": {name: {"first": e.first, "chars": e.chars, "ascent": e.gs.ascent, "height": e.gs.height}
                   for name, e in icons.style_set.items()},
    }
    (BUILD / "manifest.json").write_text(json.dumps(manifest, indent=1, ensure_ascii=False))
    write_rust(icons, libs, parts, trend_vps, qr_vp, scrim_vp, marker_vps, vp_end)
    sprite_sheet(icons, BUILD / "preview" / "sprites.png")

    total = sum(len(v) for v in files.values())
    print(f"\nDWIN_SET -> {out}")
    for name, data in files.items():
        print(f"  {name:28s} {len(data) / 1024:9.1f} KB")
    used = next_sector - ui.BG_ICL
    print(f"  {len(icons.images)} icons in {len(libs)} libraries; flash sectors {ui.BG_ICL}-{next_sector - 1} "
          f"({used * 256 / 1024:.1f} MB of the 8 MB from sector 32)")
    print(f"  total {total / 1024 / 1024:.1f} MB, built in {time.time() - t0:.1f} s")
    print(f"generated {GENERATED_RS.relative_to(ROOT)}")
    if check:
        verify(out, manifest)


def sprite_sheet(icons, path):
    from PIL import Image

    imgs = [icons.images[i] for i in icons.sprite_ids.values()]
    width, x, y, row, placed = 1600, 0, 0, 0, []
    for im in imgs:
        if x + im.width > width:
            x, y, row = 0, y + row + 4, 0
        placed.append((x, y))
        x += im.width + 4
        row = max(row, im.height)
    sheet = Image.new("RGB", (width, y + row), (60, 0, 60))
    for im, xy in zip(imgs, placed):
        sheet.paste(im, xy)
    sheet.save(path)


def verify(out, manifest):
    """verify generated files"""
    print("\nverifying")
    cfg = (out / "T5LCFG_800480.CFG").read_bytes()
    assert cfg[:5] == b"T5LC1" and cfg[8] == ui.BG_ICL and cfg[0x10:0x12] == b"\x00\x00"
    assert struct.unpack(">H", cfg[10:12])[0] == 28  # baud rate
    bgs = F.read_icl((out / manifest["background_icl"]).read_bytes())
    assert len(bgs) == len(manifest["pages"])
    for e in bgs:
        assert F.decode_icl_image(e).size == (800, 480)
    for lib in manifest["libraries"]:
        ents = F.read_icl((out / lib["file"]).read_bytes())
        assert len(ents) == lib["end"] - lib["first"]
        for e in ents[:: max(1, len(ents) // 50)]:
            F.decode_icl_image(e)
    pages = F.read_show_file((out / "14ShowFile.bin").read_bytes())
    assert len(pages) == len(manifest["pages"])
    for name, recs in zip(manifest["pages"], pages):
        for r in recs:
            assert r[0] == 0x5A and r[1] in (0x21, 0x25, 0x26)
            vp = struct.unpack(">H", r[6:8])[0]
            assert manifest["vp_start"] <= vp < manifest["vp_end"]
    init = (out / "22_Config.bin").read_bytes()
    assert len(init) == 0x20004
    for pt in manifest["parts"]:
        cmd, n = struct.unpack_from(">HH", init, pt["vp"] * 2)
        assert cmd == (pt["icl"] << 8) | 0x07
        assert struct.unpack_from(">H", init, (pt["vp"] + 2 + 3 * n) * 2)[0] == 0xFF00
    print("  CFG, ICLs (CRC + JPEG decode), 14ShowFile and 22_Config all check out")


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--check", action="store_true", help="parse the output back and verify it")
    a = ap.parse_args()
    build(check=a.check)


if __name__ == "__main__":
    main()
