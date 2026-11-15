#!/usr/bin/env python3
"""MQTT checks for the demo master, against the real broker. Standard library only.

    python tests/mqtt_check.py --board 192.168.1.3            # topics and payloads only
    python tests/mqtt_check.py --board 192.168.1.3 --press    # + a press, the landing, the doorbell

Checks what reaches the broker under the prefix (default intellithings/demo):
- the retained status is "online";
- every area of demo/demo_home.json has a retained snapshot with exactly its published fields,
  in table order, with the right JSON types, plus "ts" once the clock is synced;
- nothing else is published there (no emulator-style state or per-node copies);
- with --press: a web press starts a roll, every area gets a new snapshot when it lands, and an
  admin "doorbell" sends one non-retained event with a fresh event_id.

The broker login is read from the gitignored sdkconfig.local and never printed. --press starts
a real roll (and the hold after it) and rings the demo doorbell, so tell the team first.
"""

import argparse
import json
import pathlib
import random
import re
import socket
import struct
import sys
import threading
import time
import urllib.request

HERE = pathlib.Path(__file__).resolve().parent
ROOT = HERE.parent
sys.path.insert(0, str(ROOT / "tools"))
import gen_demo  # noqa: E402

results = []


def check(name, cond, detail=""):
    results.append((name, bool(cond)))
    print(("PASS " if cond else "FAIL ") + name + ("" if cond or detail == "" else f"  ({detail})"))
    return bool(cond)


# ---------------------------------------------------------------------------
# Minimal MQTT 3.1.1 client (from ai/emulator/tests/mqtt_check.py)

def _enc_str(s):
    b = s.encode()
    return struct.pack("!H", len(b)) + b


def _enc_len(n):
    out = bytearray()
    while True:
        d, n = n % 128, n // 128
        out.append(d | (0x80 if n else 0))
        if not n:
            return bytes(out)


class Mqtt:
    def __init__(self, host, port, user, password):
        self.sock = socket.create_connection((host, port), timeout=10)
        self.lock = threading.Lock()
        self.msgs = []  # (time, topic, payload bytes, retained)
        cid = "demo-mqtt-check-%06d" % random.randint(0, 999999)
        flags = 0x02 | (0x80 if user else 0) | (0x40 if password else 0)
        body = _enc_str("MQTT") + bytes([4, flags]) + struct.pack("!H", 30) + _enc_str(cid)
        if user:
            body += _enc_str(user)
        if password:
            body += _enc_str(password)
        self._send(0x10, body)
        first, data = self._read_packet()
        if first >> 4 != 2 or data[1] != 0:
            raise SystemExit(f"broker refused the connection (code {data[1] if len(data) > 1 else '?'})")
        self.sock.settimeout(0.5)
        self.alive = True
        threading.Thread(target=self._reader, daemon=True).start()

    def _send(self, first, body):
        with self.lock:
            self.sock.sendall(bytes([first]) + _enc_len(len(body)) + body)

    def _read_exact(self, n):
        buf = b""
        while len(buf) < n:
            try:
                chunk = self.sock.recv(n - len(buf))
            except socket.timeout:
                if not buf:
                    raise
                continue
            if not chunk:
                raise ConnectionError("broker closed the connection")
            buf += chunk
        return buf

    def _read_packet(self):
        first = self._read_exact(1)[0]
        mult, n = 1, 0
        while True:
            d = self._read_exact(1)[0]
            n += (d & 0x7F) * mult
            mult *= 128
            if not d & 0x80:
                break
        return first, self._read_exact(n) if n else b""

    def _reader(self):
        while self.alive:
            try:
                first, data = self._read_packet()
            except socket.timeout:
                continue
            except (OSError, ConnectionError):
                break
            if first >> 4 == 3:  # PUBLISH
                tl = struct.unpack("!H", data[:2])[0]
                topic = data[2:2 + tl].decode(errors="replace")
                off = 2 + tl + (2 if (first >> 1) & 3 else 0)
                self.msgs.append((time.time(), topic, data[off:], bool(first & 1)))

    def subscribe(self, topic):
        self._send(0x82, struct.pack("!H", 1) + _enc_str(topic) + b"\x00")

    def publish(self, topic, payload, retain=False):
        self._send(0x30 | (1 if retain else 0), _enc_str(topic) + payload)

    def close(self):
        self.alive = False
        try:
            self._send(0xE0, b"")
            self.sock.close()
        except OSError:
            pass

    def since(self, t0, prefix=""):
        return [m for m in list(self.msgs) if m[0] >= t0 and m[1].startswith(prefix)]


# ---------------------------------------------------------------------------

def read_config(path):
    cfg = {}
    for line in pathlib.Path(path).read_text(encoding="utf-8").splitlines():
        m = re.match(r'^CONFIG_DM_MQTT_(HOST|PORT|USERNAME|PASSWORD)=(.*)$', line.strip())
        if m:
            cfg[m.group(1)] = m.group(2).strip().strip('"')
    return cfg.get("HOST", "192.168.1.2"), int(cfg.get("PORT", "1883")), cfg.get("USERNAME", ""), cfg.get("PASSWORD", "")


def http(board, path, body=None, pin=None):
    req = urllib.request.Request(f"http://{board}{path}", method="POST" if body is not None else "GET",
                                 data=json.dumps(body).encode() if body is not None else None)
    req.add_header("Content-Type", "application/json")
    if pin:
        req.add_header("X-Admin-Pin", pin)
    with urllib.request.urlopen(req, timeout=10) as r:
        return json.loads(r.read())


def wait_for(cond, timeout):
    end = time.time() + timeout
    while time.time() < end:
        if cond():
            return True
        time.sleep(0.2)
    return cond()


def expected_areas():
    doc = json.loads((ROOT / "demo/demo_home.json").read_text(encoding="utf-8"))
    areas, ents = gen_demo.expand(doc)
    return {a["key"]: [e for e in ents if e["area"] == i and not e.get("internal")]
            for i, a in enumerate(areas)}


def type_ok(e, v):
    t = e["type"]
    if t == "bool":
        return isinstance(v, bool)
    if t == "enum":
        return isinstance(v, str) and v in e["options"]
    if isinstance(v, bool) or not isinstance(v, (int, float)):
        return False
    return isinstance(v, int) if e.get("dec", 0) == 0 else True


def check_snapshot(topic, payload, ents):
    try:
        doc = json.loads(payload)
    except ValueError:
        return check(f"{topic}: valid JSON", False)
    keys = [k for k in doc if k != "ts"]
    want = [e["field"] for e in ents]
    ok = check(f"{topic}: {len(want)} fields in inventory order", keys == want,
               f"got {keys}" if keys != want else "")
    bad = [e["field"] for e in ents if e["field"] in doc and not type_ok(e, doc[e["field"]])]
    ok &= check(f"{topic}: value types", not bad, f"wrong type: {bad}")
    ts = doc.get("ts")
    ok &= check(f"{topic}: ts with offset", isinstance(ts, str) and re.match(r"^\d{4}-\d\d-\d\dT\d\d:\d\d:\d\d[+-]\d\d:\d\d$", ts),
                f"ts={ts!r}")
    return ok


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--board", required=True, help="the demo master's IP")
    ap.add_argument("--config", default=str(ROOT / "sdkconfig.local"))
    ap.add_argument("--prefix", default="intellithings/demo")
    ap.add_argument("--pin", default="1234", help="admin PIN, for --press (default the factory PIN)")
    ap.add_argument("--press", action="store_true", help="also press, wait for the landing, ring the doorbell")
    a = ap.parse_args()

    host, port, user, password = read_config(a.config)
    print(f"broker {host}:{port} (login from {pathlib.Path(a.config).name}), board {a.board}")
    c = Mqtt(host, port, user, password)
    t0 = time.time()
    c.subscribe(a.prefix + "/#")
    areas = expected_areas()
    want = {f"{a.prefix}/{k}" for k in areas}
    retained = lambda: {m[1]: m for m in c.since(t0) if m[3]}
    wait_for(lambda: want <= set(retained()), 5)
    r = retained()

    st = r.get(a.prefix + "/status")
    check("status retained online", st and st[2] == b"online", st and st[2])
    for key, ents in areas.items():
        m = r.get(f"{a.prefix}/{key}")
        if check(f"{key}: retained snapshot", m is not None):
            check_snapshot(key, m[2], ents)
    extra = sorted(set(r) - want - {a.prefix + "/status"})
    check("nothing else retained under the prefix", not extra, f"extra: {extra}")

    if a.press:
        s0 = http(a.board, "/api/state")
        if s0["state"] in ("rolling", "hold"):
            print(f"board is {s0['state']}; waiting up to {s0['lock_left_s'] + 5} s for it to unlock")
            wait_for(lambda: http(a.board, "/api/state")["lock_left_s"] == 0, s0["lock_left_s"] + 5)
        t1 = time.time()
        res = http(a.board, "/api/press", {"src": "web"})
        check("web press starts a roll", res.get("result") == "rolling", res)
        roll_s = s0["roll_ms"] / 1000
        landed = wait_for(lambda: want <= {m[1] for m in c.since(t1)}, roll_s + 5)
        msgs = c.since(t1)
        first = min((m[0] for m in msgs), default=None)
        check(f"every area republished when the roll lands (~{roll_s:.0f} s)", landed,
              f"{len({m[1] for m in msgs} & want)}/{len(want)} areas")
        if first:
            check("nothing published during the roll", first - t1 >= roll_s - 0.5, f"first after {first - t1:.1f} s")
        s1 = http(a.board, "/api/state")
        check("state shows the roll", s1["roll"] > 0 and s1["state"] == "hold", f"{s1['state']}, roll {s1['roll']}")
        res = http(a.board, "/api/press", {"src": "web"})
        check("a second press is locked", res.get("result") == "locked", res)
        for key, ents in areas.items():
            m = [x for x in c.since(t1) if x[1] == f"{a.prefix}/{key}"]
            if m:
                check_snapshot(key + " (after roll)", m[-1][2], ents)

        t2 = time.time()
        res = http(a.board, "/api/admin/cmd", {"op": "doorbell"}, a.pin)
        check("admin doorbell accepted", res.get("ok"), res)
        bell = a.prefix + "/home/doorbell"
        wait_for(lambda: any(m[1] == bell for m in c.since(t2)), 3)
        rings = [m for m in c.since(t2) if m[1] == bell]
        if check("one doorbell event", len(rings) == 1, f"{len(rings)} events"):
            ev = json.loads(rings[0][2])
            check("doorbell not retained", not rings[0][3])
            check("doorbell payload", ev.get("event") == "doorbell_pressed" and
                  re.match(r"^boot-\d+-press-\d+$", ev.get("event_id", "")) and "ts" in ev, ev)
            print(f"     doorbell event_id {ev.get('event_id')}")
    c.close()
    failed = [n for n, ok in results if not ok]
    print(f"\n{len(results) - len(failed)}/{len(results)} passed")
    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    main()
