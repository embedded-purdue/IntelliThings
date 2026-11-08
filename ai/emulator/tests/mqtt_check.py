#!/usr/bin/env python3
"""Checks the emulator's MQTT traffic on the real broker:

    python mqtt_check.py [--emu 192.168.1.4] [--seconds 30] [--demo] [--broker-switch]

Standard library only (a minimal MQTT 3.1.1 client is built in). The broker address and login
are read from the gitignored esp32_c3_emulator/sdkconfig.local and never printed; --config
points elsewhere.

Checks: retained online statuses, a sensor snapshot from every room every ~3 s with the
contract's fields and ranges, the retained emulator state, a command over MQTT, a test AI
message round trip (non-retained; the room's previous retained message is put back
afterwards), the agent's retained summary and thinking state, and optionally demo mode
(--demo) and a broker switch away and back (--broker-switch: statuses go offline, then online).
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
import urllib.error
import urllib.request

HERE = pathlib.Path(__file__).resolve().parent
DEFAULT_CONFIG = HERE.parent / "esp32_c3_emulator" / "sdkconfig.local"
NODES = ["emu-kitchen", "emu-bedroom", "emu-living-room"]
FIELDS = {  # contract fields and ranges (docs/interfaces/mqtt.md)
    "temperature_c": (0, 50), "humidity_pct": (0, 100), "voc_index": (1, 500),
    "co2_ppm": (400, 5000), "lux": (0, 65535), "distance_mm": (30, 1200),
    "pm1_0_ugm3": (0, 500), "pm2_5_ugm3": (0, 500), "pm10_ugm3": (0, 500),
}
results = []


def check(name, cond, detail=""):
    results.append((name, bool(cond)))
    print(("PASS " if cond else "FAIL ") + name + ("" if cond or detail == "" else f"  ({detail})"))
    return bool(cond)


# ---------------------------------------------------------------------------
# Minimal MQTT 3.1.1 client (QoS 0 subscribe/publish, enough for checks)

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
        cid = "emu-mqtt-check-%06d" % random.randint(0, 999999)
        flags = 0x02 | (0x80 if user else 0) | (0x40 if password else 0)
        body = _enc_str("MQTT") + bytes([4, flags]) + struct.pack("!H", 30) + _enc_str(cid)
        if user:
            body += _enc_str(user)
        if password:
            body += _enc_str(password)
        self._send(0x10, body)
        hdr, data = self._read_packet()
        if hdr >> 4 != 2 or data[1] != 0:
            raise SystemExit(f"broker refused the connection (code {data[1] if len(data) > 1 else '?'})")
        self.sock.settimeout(0.5)
        self.alive = True
        threading.Thread(target=self._reader, daemon=True).start()
        threading.Thread(target=self._pinger, daemon=True).start()

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

    def _pinger(self):
        while self.alive:
            time.sleep(10)
            try:
                self._send(0xC0, b"")
            except OSError:
                break

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

    def last(self, topic):
        for m in reversed(list(self.msgs)):
            if m[1] == topic:
                return m
        return None


# ---------------------------------------------------------------------------

def read_config(path):
    cfg = {}
    for line in pathlib.Path(path).read_text(encoding="utf-8").splitlines():
        m = re.match(r'^CONFIG_EMU_MQTT_(HOST|PORT|USERNAME|PASSWORD)=(.*)$', line.strip())
        if m:
            cfg[m.group(1)] = m.group(2).strip().strip('"')
    return cfg.get("HOST", "192.168.1.2"), int(cfg.get("PORT", "1883")), cfg.get("USERNAME", ""), cfg.get("PASSWORD", "")


def http(emu, method, path, body=None):
    data = json.dumps(body).encode() if body is not None else None
    req = urllib.request.Request(f"http://{emu}{path}", data=data, method=method,
                                 headers={"Content-Type": "application/json"})
    try:
        with urllib.request.urlopen(req, timeout=15) as r:
            return r.status, json.load(r)
    except urllib.error.HTTPError as e:
        return e.code, json.load(e)


def jload(b):
    try:
        return json.loads(b.decode())
    except ValueError:
        return None


def wait_for(cond, timeout):
    t0 = time.time()
    while time.time() - t0 < timeout:
        r = cond()
        if r:
            return r
        time.sleep(0.2)
    return cond()


def test_basics(c, emu, seconds):
    t0 = time.time()
    c.subscribe("intellithings/#")
    time.sleep(seconds)
    retained = {m[1]: m for m in c.msgs if m[3]}
    for n in NODES:
        st = retained.get(f"intellithings/{n}/status")
        check(f"{n}: retained status online", st and st[2] == b"online", st and st[2])
    st = retained.get("intellithings/emulator/status")
    check("emulator: retained status online", st and st[2] == b"online", st and st[2])
    state = retained.get("intellithings/emulator/state")
    sj = state and jload(state[2])
    check("emulator: retained state JSON", sj and len(sj.get("nodes", [])) == 3)

    for n in NODES:
        snaps = [m for m in c.since(t0, f"intellithings/{n}/sensors") if not m[3]]
        expect = seconds / 3
        check(f"{n}: ~one snapshot per 3 s", expect * 0.75 <= len(snaps) <= expect * 1.25 + 1,
              f"{len(snaps)} in {seconds} s")
        gaps = [b[0] - a[0] for a, b in zip(snaps, snaps[1:])]
        if gaps:
            check(f"{n}: gaps 2.5–3.5 s", all(2.5 <= g <= 3.5 for g in gaps),
                  f"min {min(gaps):.2f} max {max(gaps):.2f}")
        bad = []
        for m in snaps:
            j = jload(m[2])
            if not j or j.get("node_id") != n or not isinstance(j.get("presence"), bool) or "ts" not in j:
                bad.append("fields")
                continue
            for k, (lo, hi) in FIELDS.items():
                if not isinstance(j.get(k), (int, float)) or not lo <= j[k] <= hi:
                    bad.append(k)
        check(f"{n}: every snapshot has the contract's fields, ranges and ts", snaps and not bad, bad[:5])
    return retained


def test_cmd(c, emu):
    t0 = time.time()
    c.publish("intellithings/emulator/cmd",
              json.dumps({"node": "emu-kitchen", "action": "scenario", "name": "pm_spike", "duration_s": 30}).encode())

    def seen():
        for m in c.since(t0, "intellithings/emulator/state"):
            j = jload(m[2])
            lc = j and j.get("last_cmd")
            if lc and lc.get("source") == "mqtt" and "PM2.5 spike" in lc.get("msg", ""):
                return m[0] - t0
        return None
    dt = wait_for(seen, 5)
    check("command over MQTT shows up in the emulator state", dt is not None, "")
    if dt is not None:
        print(f"     state updated {dt:.2f} s after the command")
    http(emu, "POST", "/api/cmd", {"node": "kitchen", "action": "reset"})


def test_display(c, emu, retained):
    topic = "intellithings/emu-bedroom/display"
    prev = retained.get(topic)
    text = f"mqtt_check {int(time.time())}"
    t0 = time.time()
    c.publish(topic, json.dumps({"text": text, "ts": time.strftime("%Y-%m-%dT%H:%M:%S+00:00", time.gmtime())}).encode())

    def arrived():
        st, m = http(emu, "GET", "/api/messages")
        return st == 200 and any(x["text"] == text and x["node_id"] == "emu-bedroom" for x in m["messages"])
    check("test AI message reaches the emulator's bedroom", wait_for(arrived, 5))
    # Put the room's previous retained message back, so HA and the board show it again.
    if prev and prev[2]:
        c.publish(topic, prev[2], retain=True)
        time.sleep(1)
        print("     previous retained bedroom message restored")
    else:
        print("     (the bedroom had no retained message before)")


def test_ai(retained):
    s = retained.get("intellithings/ai/summary")
    i = retained.get("intellithings/ai/inference_status")
    sj, ij = s and jload(s[2]), i and jload(i[2])
    print("     agent summary:", (sj or {}).get("text", "—")[:80], "| thinking:", (ij or {}).get("state", "—"),
          (ij or {}).get("ts", ""))
    for n in NODES:
        d = retained.get(f"intellithings/{n}/display")
        dj = d and jload(d[2])
        print(f"     {n} display:", (dj or {}).get("text", "—"), (dj or {}).get("ts", ""))


def test_ai_on_board(emu, retained):
    st, p = http(emu, "GET", "/api/poll?log=999999&cmd=999999&ev=999999&msg=999999")
    ai = p["ai"]
    if "intellithings/ai/summary" in retained:
        check("board shows the agent's summary", ai["summary"] is not None and ai["summary"]["text"])
    if "intellithings/ai/inference_status" in retained:
        check("board shows the agent's thinking state", ai["inference"] is not None)
    for i, n in enumerate(NODES):
        if retained.get(f"intellithings/{n}/display"):
            check(f"board shows {n}'s latest AI message", ai["latest"][i] is not None)


def test_demo(c, emu):
    st, s = http(emu, "GET", "/api/state")
    was_on = s["demo"]["publish"]
    t0 = time.time()
    http(emu, "POST", "/api/cmd", {"action": "demo", "op": "publish", "on": True})
    http(emu, "POST", "/api/cmd", {"action": "demo", "op": "preset", "id": 35})
    home = wait_for(lambda: c.since(t0, "intellithings/emulator/demo/home"), 5)
    time.sleep(1.5)  # home comes first; let the rest of the burst (3 node topics, state) arrive
    hj = home and jload(home[-1][2])
    check("demo: home published for preset #35", hj and hj.get("scenario_id") == 35,
          hj and hj.get("scenario_id"))
    if hj:
        areas = hj.get("areas", {})
        check("demo: home has 9 areas and 61 values", len(areas) == 9 and sum(len(a) for a in areas.values()) == 61,
              (len(areas), sum(len(a) for a in areas.values())))
        check("demo: garage door open in preset #35", areas.get("garage", {}).get("door_open") is True)
    stt = c.since(t0, "intellithings/emulator/demo/state")
    sj = stt and jload(stt[-1][2])
    check("demo: state published", sj and sj.get("publish") is True and sj.get("scenario_id") == 35, sj)
    for n in NODES:
        ns = c.since(t0, f"intellithings/emulator/demo/{n}/sensors")
        nj = ns and jload(ns[-1][2])
        check(f"demo: {n} sensors in the node schema", nj and nj.get("node_id") == n
              and all(k in nj for k in FIELDS) and isinstance(nj.get("presence"), bool))
    # A fresh subscriber gets the retained demo topics
    c2 = Mqtt(*ARGS_BROKER)
    t1 = time.time()
    c2.subscribe("intellithings/emulator/demo/#")
    time.sleep(2)
    got = {m[1] for m in c2.since(t1) if m[3]}
    c2.close()
    check("demo: a new subscriber gets state, home and 3 node topics retained", len(got) == 5, sorted(got))
    if not was_on:
        t2 = time.time()
        http(emu, "POST", "/api/cmd", {"action": "demo", "op": "publish", "on": False})
        off = wait_for(lambda: [m for m in c.since(t2, "intellithings/emulator/demo/state")
                                if (jload(m[2]) or {}).get("publish") is False], 5)
        check("demo: turning publishing off announces publish:false", off)


def test_switch(c, emu):
    st, cfg = http(emu, "GET", "/api/config")
    t0 = time.time()
    try:
        http(emu, "POST", "/api/config", {"host": "192.0.2.1"})
        offs = wait_for(lambda: {m[1] for m in c.since(t0) if m[2] == b"offline" and m[1].endswith("/status")}
                        if len({m[1] for m in c.since(t0) if m[2] == b"offline" and m[1].endswith("/status")}) >= 4 else None, 20)
        check("broker switch: all 4 statuses go offline on the old broker", offs and len(offs) >= 4, offs)
    finally:
        t1 = time.time()
        if cfg["saved"]:
            http(emu, "POST", "/api/config", {"host": cfg["host"]})
        else:
            http(emu, "POST", "/api/config/reset", {})
    back = wait_for(lambda: {m[1] for m in c.since(t1) if m[2] == b"online" and m[1].endswith("/status")}
                    if len({m[1] for m in c.since(t1) if m[2] == b"online" and m[1].endswith("/status")}) >= 4 else None, 40)
    check("broker switch: all 4 back online after switching back", back and len(back) >= 4, back)
    if back:
        print(f"     back online {max(m[0] for m in c.since(t1) if m[2] == b'online') - t1:.1f} s after switching back")


ARGS_BROKER = None


def main():
    global ARGS_BROKER
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawTextHelpFormatter)
    ap.add_argument("--emu", default="192.168.1.4")
    ap.add_argument("--config", default=str(DEFAULT_CONFIG))
    ap.add_argument("--seconds", type=int, default=30)
    ap.add_argument("--demo", action="store_true")
    ap.add_argument("--broker-switch", action="store_true")
    a = ap.parse_args()
    ARGS_BROKER = read_config(a.config)
    print(f"broker {ARGS_BROKER[0]}:{ARGS_BROKER[1]} (login from {pathlib.Path(a.config).name}), emulator {a.emu}")
    c = Mqtt(*ARGS_BROKER)
    try:
        retained = test_basics(c, a.emu, a.seconds)
        test_cmd(c, a.emu)
        test_display(c, a.emu, retained)
        test_ai(retained)
        test_ai_on_board(a.emu, retained)
        if a.demo:
            test_demo(c, a.emu)
        if a.broker_switch:
            test_switch(c, a.emu)
    finally:
        c.close()
    passed = sum(ok for _, ok in results)
    print(f"\n{passed}/{len(results)} passed")
    sys.exit(0 if passed == len(results) else 1)


if __name__ == "__main__":
    main()
