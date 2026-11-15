#!/usr/bin/env python3
"""Serve the demo master's pages from the checkout, for editing them without reflashing.

    python tools/page_dev_server.py                      # built-in mock of the board's API
    python tools/page_dev_server.py --board 192.168.1.5  # forward /api/* to a real board

Then open http://localhost:8766/ (demo page) and /admin (PIN 1234 on the mock).
The pages are read from main/web/ on every request, so a browser reload shows each edit.
The mock follows demo/demo_home.json and the board's API closely enough to exercise the pages;
it doesn't apply the fill and derive rules exactly. Standard library only.
"""

import argparse
import json
import math
import random
import sys
import threading
import time
import urllib.error
import urllib.request
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import parse_qs, urlparse

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "tools"))
import gen_demo  # noqa: E402  (expand() reads the JSON the same way the build does)

PIN = "1234"


class Mock:
    def __init__(self):
        doc = json.loads((ROOT / "demo/demo_home.json").read_text(encoding="utf-8"))
        self.areas, self.ents = gen_demo.expand(doc)
        self.groups = doc["groups"]
        self.builtin = doc["presets"]
        self.lock = threading.Lock()
        self.reset_library()
        self.settings = {"host": "192.168.1.2", "port": 1883, "username": "demo", "has_password": True,
                         "keepalive_s": 15, "prefix": "intellithings/demo", "theme": "dark",
                         "animation": True, "fahrenheit": True}
        self.pin = PIN
        self.publish, self.loop = True, True
        self.interval, self.source, self.mix = 90, "presets", 30
        self.roll_ms, self.hold_s, self.spread, self.abnormal = 4000, 30, "normal", 15
        self.expect, self.changed_at = "", time.time()
        self.presses = {"buzzer": 0, "web": 0, "key": 0, "admin": 0}
        self.total, self.ignored, self.rolls, self.steps, self.doorbells = 0, 0, 0, 0, 0
        self.press_n, self.press_src, self.press_result, self.press_left = 0, "web", "rolling", 0
        self.events, self.event_id = [], 0
        self.t0 = time.time()
        self.seq, self.state, self.roll = 0, "looping", 0
        self.apply(0)
        self.next_at = time.time() + self.interval
        self.log("Mock demo master ready")

    # -- library and limits
    def reset_library(self):
        self.lib = [{"name": p["name"], "group": [g["key"] for g in self.groups].index(p["group"]),
                     "doorbell": bool(p.get("doorbell")), "expect": p.get("expect", ""),
                     "set": dict(p["set"])} for p in self.builtin]
        self.limits = {}
        for e in self.ents:
            if e["type"] == "number":
                n = e.get("normal", e["caps"])
                self.limits[e["key"]] = [e["caps"][0], e["caps"][1], n[0], n[1]]
            elif e["type"] == "bool":
                self.limits[e["key"]] = [0, 1, e.get("normal_p", 0.5), 0]
        self.default_limits = json.loads(json.dumps(self.limits))
        self.lib_ver = getattr(self, "lib_ver", 0) + 1
        self.factory = True

    def log(self, text):
        self.event_id += 1
        self.events.append({"id": self.event_id, "t": int(time.time() - self.t0), "text": text})
        self.events = self.events[-40:]

    # -- values
    def value(self, e, wild):
        k = e["key"]
        if e["type"] == "number":
            lo, hi, nlo, nhi = self.limits[k]
            v = random.uniform(lo, hi) if wild else random.uniform(nlo, nhi)
            return round(v, e.get("dec", 0))
        if e["type"] == "bool":
            return 1 if random.random() < (0.5 if wild else self.limits[k][2]) else 0
        opts = e["options"] if wild else e.get("normal", e["options"])
        return e["options"].index(random.choice(opts))

    def encode(self, e, v):
        if e["type"] == "bool":
            return 1 if v else 0
        if e["type"] == "enum":
            return e["options"].index(v)
        return v

    def apply(self, i):
        sc = self.lib[i]
        state = random.getstate()
        random.seed(sc["name"])  # same scenario, same values, like the board
        self.vals = [self.encode(e, sc["set"][e["key"]]) if e["key"] in sc["set"] else self.value(e, False)
                     for e in self.ents]
        random.setstate(state)
        self.cur, self.name, self.group, self.roll = i, sc["name"], sc["group"], 0
        self.expect, self.changed_at = sc.get("expect", ""), time.time()
        self.seq += 1
        self.steps += 1
        if sc.get("doorbell"):
            self.doorbells += 1
            self.log(f"Doorbell #{self.doorbells}")

    def start_roll(self, from_loop):
        self.rolls += 1
        self.target = [self.value(e, self.spread == "uniform" or random.random() * 100 < self.abnormal)
                       for e in self.ents]
        self.state, self.from_loop = "rolling", from_loop
        self.next_at = time.time() + self.roll_ms / 1000

    def tick(self):
        now = time.time()
        if self.state == "rolling" and now >= self.next_at:
            self.vals, self.cur, self.roll = self.target, -1, self.rolls
            self.name = f"Random roll #{self.roll}"
            self.expect, self.changed_at = "", now
            self.seq += 1
            self.log(f"Roll #{self.roll} landed")
            if self.from_loop:
                self.state = "looping" if self.loop else "idle"
                self.next_at = now + self.interval
            else:
                self.state, self.next_at = "hold", now + self.hold_s
        elif self.state == "hold" and now >= self.next_at:
            self.state, self.next_at = ("looping" if self.loop else "idle"), now
        elif self.state == "looping" and now >= self.next_at:
            if self.source == "random" or (self.source == "mix" and random.random() * 100 < self.mix):
                self.start_roll(True)
            else:
                choices = [i for i in range(len(self.lib)) if i != self.cur] or [0]
                self.apply(random.choice(choices))
                self.next_at = now + self.interval

    def lock_left(self):
        now = time.time()
        if self.state == "rolling" and not self.from_loop:
            return math.ceil(self.next_at - now) + self.hold_s
        return max(0, math.ceil(self.next_at - now)) if self.state == "hold" else 0

    # -- API
    def state_json(self, since):
        self.tick()
        now = time.time()
        g = self.groups[self.group]
        s = {"seq": self.seq, "state": self.state, "publish": self.publish, "loop": self.loop,
             "scenario": self.name, "scenario_id": self.cur + 1 if self.roll == 0 and self.cur >= 0 else None,
             "group": "random" if self.roll else g["key"], "group_label": "Random roll" if self.roll else g["label"],
             "roll": self.roll, "step_left_s": None if self.state == "idle" else max(0, math.ceil(self.next_at - now)),
             "lock_left_s": self.lock_left(),
             "roll_left_ms": int(max(0, self.next_at - now) * 1000) if self.state == "rolling" else 0,
             "roll_ms": self.roll_ms, "roll_from_loop": self.state == "rolling" and self.from_loop,
             "interval_s": self.interval, "hold_s": self.hold_s, "presses": self.total,
             "doorbells": self.doorbells, "mqtt": True, "buzzer_held": False,
             "press": {"n": self.press_n, "src": self.press_src, "result": self.press_result,
                       "lock_left_s": self.press_left},
             "look": {"theme": self.settings["theme"], "animation": self.settings["animation"],
                      "fahrenheit": self.settings["fahrenheit"]}, "lib": self.lib_ver}
        if since is None or since != self.seq:
            s["v"] = self.vals
        return s

    def meta(self):
        ents = []
        for e in self.ents:
            d = {"key": e["key"], "field": e["field"], "label": e["label"], "unit": e.get("unit", ""),
                 "area": e["area"], "type": e["type"], "dec": e.get("dec", 0), "derived": bool(e.get("derive")),
                 "fill": e.get("fill", "normal"), "kind": e["kind"], "internal": bool(e.get("internal"))}
            if e["type"] != "number":
                d["options"] = e.get("options") or e.get("labels")
            if e["type"] == "enum":
                d["normal_mask"] = sum(1 << e["options"].index(o) for o in e.get("normal", e["options"]))
            else:
                d["limits"], d["default"] = self.limits[e["key"]], self.default_limits[e["key"]]
            ents.append(d)
        return {"fw": "mock", "areas": [{"key": a["key"], "label": a["label"], "node": bool(a["node_room"])}
                                        for a in self.areas],
                "entities": ents, "groups": [{"key": g["key"], "label": g["label"]} for g in self.groups],
                "scenarios": [{"name": s["name"], "group": s["group"], "n": len(s["set"]),
                               "doorbell": s.get("doorbell", False)} for s in self.lib],
                "lib": self.lib_ver, "factory": self.factory, "max_sets": 20, "max_scenarios": 200}

    def press(self, src):
        self.tick()
        self.presses[src] += 1
        if src != "admin":
            self.total += 1
        if self.state == "rolling":
            self.ignored += 1
            res = "busy"
        elif self.lock_left() > 0 and src != "admin":
            self.ignored += 1
            res = "locked"
        else:
            self.start_roll(False)
            res = "rolling"
        self.log(f"Press ({src}): {res}")
        self.press_n += 1
        self.press_src, self.press_result, self.press_left = src, res, self.lock_left()
        return {"ok": res == "rolling", "result": res, "lock_left_s": self.lock_left()}

    def status(self, after):
        return {"fw": "mock", "idf": "-", "uptime_s": int(time.time() - self.t0), "heap_free": 150000,
                "heap_min": 120000, "time": time.strftime("%Y-%m-%dT%H:%M:%S-05:00"),
                "wifi": {"connected": True, "ip": "127.0.0.1", "rssi": -50, "reconnects": 0,
                         "hostname": "intellithings-demo"},
                "mqtt": {"connected": True, "uri": f"mqtt://{self.settings['host']}:{self.settings['port']}",
                         "sent": self.seq * 5, "reconnects": 0, "error": ""},
                "button": {"gpio": 3, "held": False, "level": False, "held_ms": 0, "rejected": 0},
                "engine": {"presses_total": self.total, "presses": self.presses, "ignored": self.ignored,
                           "rolls": self.rolls, "steps": self.steps, "published": self.seq,
                           "doorbells": self.doorbells, "changed_ago_s": int(time.time() - self.changed_at),
                           "expect": self.expect,
                           "source": self.source, "mix_pct": self.mix, "spread": self.spread,
                           "abnormal_pct": self.abnormal},
                "settings": {**self.settings, "topics": [a["key"] for a in self.areas]},
                "ai": {"rooms": [{"node": "emu-kitchen", "text": "Mock: slightly stuffy; ventilate",
                                  "ts": time.strftime("%Y-%m-%dT%H:%M:%S%z"), "age_s": 5}],
                       "summary": None, "thinking": {"state": "stop thinking", "ts": "", "age_s": 5}},
                "events": [e for e in self.events if e["id"] > after]}

    def cmd(self, b):
        op = b.get("op")
        if op == "roll":
            r = self.press("admin")
            return r["ok"], "rolling the house" if r["ok"] else "already rolling"
        if op == "publish":
            self.publish = b["on"]
        elif op == "loop":
            self.loop = b.get("on", self.loop)
            self.interval = b.get("interval_s", self.interval)
            self.source = b.get("source", self.source)
            self.mix = b.get("mix_pct", self.mix)
            if self.loop and self.state == "idle":
                self.state, self.next_at = "looping", time.time() + self.interval
            if not self.loop and self.state == "looping":
                self.state = "idle"
        elif op == "settings":
            self.roll_ms = b.get("roll_s", self.roll_ms // 1000) * 1000
            self.hold_s = b.get("hold_s", self.hold_s)
            self.spread = b.get("spread", self.spread)
            self.abnormal = b.get("abnormal_pct", self.abnormal)
        elif op == "play":
            self.apply(b["id"] - 1)
            self.state, self.next_at = "hold", time.time() + self.hold_s
        elif op == "next":
            self.next_at = 0
            self.state = "looping"
            self.tick()
        elif op == "doorbell":
            self.doorbells += 1
        elif op == "reset_counts":
            self.presses = dict.fromkeys(self.presses, 0)
            self.total = self.ignored = 0
        elif op != "republish":
            return False, f"unknown op {op}"
        self.log(f"Admin: {op}")
        return True, f"{op}: done"

    def scenario_put(self, b):
        sc = {"name": b["name"], "group": [g["key"] for g in self.groups].index(b["group"]),
              "doorbell": bool(b.get("doorbell")), "expect": b.get("expect", ""), "set": b["set"]}
        if not sc["name"]:
            return False, "the scenario needs a name"
        if len(sc["set"]) > 20:
            return False, "a scenario sets at most 20 values"
        if b.get("id"):
            self.lib[b["id"] - 1] = sc
        else:
            self.lib.append(sc)
        self.lib_ver += 1
        self.factory = False
        return True, f"scenario \"{sc['name']}\" saved"

    def limit(self, b):
        lim = self.limits[b["key"]]
        for i, k in enumerate(("min", "max", "nmin", "nmax")):
            if k in b:
                lim[i] = b[k]
        if lim[0] >= lim[1]:
            return False, "caps must be low < high"
        self.lib_ver += 1
        self.factory = False
        return True, f"{b['key']} limits saved"


class Handler(BaseHTTPRequestHandler):
    mock = None
    board = None

    def log_message(self, fmt, *args):
        pass

    def send(self, code, body, ctype="application/json"):
        data = body if isinstance(body, bytes) else json.dumps(body).encode()
        self.send_response(code)
        self.send_header("Content-Type", ctype)
        self.send_header("Content-Length", str(len(data)))
        self.send_header("Cache-Control", "no-store")
        self.end_headers()
        self.wfile.write(data)

    def page(self, name):
        self.send(200, (ROOT / "main/web" / name).read_bytes(), "text/html; charset=utf-8")

    def proxy(self, body):
        req = urllib.request.Request(f"http://{self.board}{self.path}", data=body, method=self.command)
        for h in ("Content-Type", "X-Admin-Pin"):
            if self.headers.get(h):
                req.add_header(h, self.headers[h])
        try:
            with urllib.request.urlopen(req, timeout=5) as r:
                self.send(r.status, r.read(), r.headers.get("Content-Type", "application/json"))
        except urllib.error.HTTPError as e:
            self.send(e.code, e.read())
        except OSError as e:
            self.send(502, {"ok": False, "msg": f"board unreachable: {e}"})

    def handle_api(self, body):
        m, url = self.mock, urlparse(self.path)
        q = {k: v[0] for k, v in parse_qs(url.query).items()}
        b = json.loads(body or b"{}") if self.command == "POST" else {}
        p = url.path
        with m.lock:
            if p == "/api/state":
                return self.send(200, m.state_json(int(q["since"]) if "since" in q else None))
            if p == "/api/meta":
                return self.send(200, m.meta())
            if p == "/api/press":
                return self.send(200, m.press("key" if b.get("src") == "key" else "web"))
            if p.startswith("/api/admin/"):
                if self.headers.get("X-Admin-Pin") != m.pin:
                    return self.send(401, {"ok": False, "msg": "wrong PIN"})
                ok, msg = True, "done"
                if p == "/api/admin/status":
                    return self.send(200, m.status(int(q.get("after", 0))))
                if p == "/api/admin/scenario" and self.command == "GET":
                    i = int(q["id"]) - 1
                    s = m.lib[i]
                    return self.send(200, {"id": i + 1, "name": s["name"], "group": s["group"],
                                           "doorbell": s.get("doorbell", False),
                                           "expect": s.get("expect", ""), "set": s["set"]})
                if p == "/api/admin/cmd":
                    ok, msg = m.cmd(b)
                elif p == "/api/admin/scenario":
                    ok, msg = m.scenario_put(b)
                elif p == "/api/admin/scenario/delete":
                    del m.lib[b["id"] - 1]
                    m.lib_ver += 1
                    m.factory = False
                    msg = "deleted"
                elif p == "/api/admin/library/reset":
                    m.reset_library()
                    msg = "library reset"
                elif p == "/api/admin/limit":
                    ok, msg = m.limit(b)
                elif p == "/api/admin/limits/reset":
                    m.limits = json.loads(json.dumps(m.default_limits))
                    m.lib_ver += 1
                    msg = "limits reset"
                elif p == "/api/admin/settings":
                    for k in ("host", "port", "username", "keepalive_s", "prefix", "theme", "animation",
                              "fahrenheit"):
                        if k in b:
                            m.settings[k] = b[k]
                    if "pin" in b:
                        m.pin = b["pin"]
                    msg = "saved"
                elif p not in ("/api/admin/login", "/api/admin/settings/reset", "/api/admin/reboot"):
                    return self.send(404, {"ok": False, "msg": "no such endpoint"})
                return self.send(200 if ok else 400, {"ok": ok, "msg": msg})
        return self.send(404, {"ok": False, "msg": "no such endpoint"})

    def route(self):
        body = self.rfile.read(int(self.headers.get("Content-Length") or 0)) if self.command == "POST" else None
        path = urlparse(self.path).path
        if path == "/":
            return self.page("demo.html")
        if path == "/admin":
            return self.page("admin.html")
        if path.startswith("/api/"):
            return self.proxy(body) if self.board else self.handle_api(body)
        self.send(404, {"ok": False, "msg": "not found"})

    do_GET = route
    do_POST = route


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--board", help="forward /api/* to this board (IP or host) instead of the mock")
    ap.add_argument("--port", type=int, default=8766)
    a = ap.parse_args()
    Handler.board = a.board
    Handler.mock = None if a.board else Mock()
    srv = ThreadingHTTPServer(("127.0.0.1", a.port), Handler)
    print(f"http://localhost:{a.port}/  ({'board ' + a.board if a.board else 'mock API, admin PIN ' + PIN})")
    srv.serve_forever()


if __name__ == "__main__":
    main()
