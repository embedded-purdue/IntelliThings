#!/usr/bin/env python3
"""Checks the emulator's HTTP API end to end against a running board.

    python api_check.py [--host 192.168.1.4] [--broker-test]

Standard library only. Covers the v0.1.0 API (commands, validation, state, meta, messages)
and the v2.1.0 additions (poll, history, config, guide). It changes the rooms while it runs and
resets them at the end, so tell the team before running it on the shared board.

--broker-test also points the board at an unused broker address and back. Settings are left
as found (saved values restored, or unsaved defaults reset) even if a check fails; if the run
is interrupted, fix the host in Settings or POST /api/config/reset.
"""

import argparse
import gzip
import json
import struct
import sys
import time
import urllib.error
import urllib.request

BASE = "http://192.168.1.4"
results = []


def http(method, path, body=None, raw=False):
    data = None
    headers = {}
    if body is not None:
        data = body if isinstance(body, bytes) else json.dumps(body).encode()
        headers["Content-Type"] = "application/json"
    req = urllib.request.Request(BASE + path, data=data, headers=headers, method=method)
    try:
        with urllib.request.urlopen(req, timeout=15) as r:
            payload = r.read()
            status, hdrs = r.status, r.headers
    except urllib.error.HTTPError as e:
        payload, status, hdrs = e.read(), e.code, e.headers
    if hdrs.get("Content-Encoding") == "gzip":
        payload = gzip.decompress(payload)
    if raw:
        return status, hdrs, payload
    try:
        return status, json.loads(payload)
    except ValueError:
        return status, payload.decode(errors="replace")


def get(path):
    return http("GET", path)


def cmd(body):
    return http("POST", "/api/cmd", body)


def check(name, cond, detail=""):
    results.append((name, bool(cond)))
    print(("PASS " if cond else "FAIL ") + name + ("" if cond or not detail else f"  ({detail})"))
    return bool(cond)


def node(state, room):
    return next(n for n in state["nodes"] if n["room"] == room)


def wait_publish(seconds=None):
    s = get("/api/state")[1]
    time.sleep(seconds if seconds is not None else s["publish_interval_s"] + 0.7)


# ---------------------------------------------------------------------------


def test_v01():
    st, s = get("/api/state")
    check("state: 200 and core fields", st == 200 and all(
        k in s for k in ("fw", "uptime_s", "free_heap", "wifi", "mqtt", "time", "nodes")), st)
    check("state: three nodes with readings", len(s["nodes"]) == 3 and all(
        n["reading"] is not None for n in s["nodes"]))
    print(f"     fw {s['fw']}, uptime {s['uptime_s']} s, heap {s['free_heap']} "
          f"(min {s['min_free_heap']}), MQTT emulator={s['mqtt']['emulator_connected']}, "
          f"nodes={[n['mqtt'] for n in s['nodes']]}")

    st, m = get("/api/meta")
    check("meta: 9 metrics, 12 scenarios, 3 nodes",
          st == 200 and len(m["metrics"]) == 9 and len(m["scenarios"]) == 12
          and len(m["nodes"]) == 3)
    st, msgs = get("/api/messages")
    check("messages: list", st == 200 and isinstance(msgs.get("messages"), list))

    st, hdrs, body = http("GET", "/", raw=True)
    check("panel: 200, gzip, HTML", st == 200 and hdrs.get("Content-Encoding") == "gzip"
          and b"IntelliThings Emulator" in body)

    defaults = {sc["name"]: sc["default_room"] or "kitchen" for sc in m["scenarios"]}
    for name, room in defaults.items():
        st, r = cmd({"node": room, "action": "scenario", "name": name})
        n = node(get("/api/state")[1], room)
        check(f"scenario {name} on {room}",
              st == 200 and r["ok"] and n["mode"] == "scenario" and n["scenario"] == name, r)

    st, r = cmd({"node": "bedroom", "action": "scenario", "name": "hot", "duration_s": 5})
    n = node(get("/api/state")[1], "bedroom")
    check("scenario with duration_s 5", st == 200 and 0 <= (n["scenario_remaining_s"] or 99) <= 5)
    st, r = cmd({"node": "all", "action": "scenario", "name": "enter", "duration_s": 0})
    s = get("/api/state")[1]
    check("scenario on all, until cleared", st == 200 and all(
        n["scenario"] == "enter" and n["scenario_remaining_s"] is None for n in s["nodes"]))
    cmd({"node": "all", "action": "reset"})

    cmd({"node": "kitchen", "action": "set", "ranges": {"co2_ppm": 1500}, "instant": True})
    wait_publish()
    co2 = node(get("/api/state")[1], "kitchen")["reading"]["co2_ppm"]
    check("set fixed CO2 1500 instant", co2 == 1500, co2)

    cmd({"node": "living-room", "action": "set", "instant": True,
         "ranges": {"temperature_c": {"min": 25, "max": 26}}})
    wait_publish()
    t = node(get("/api/state")[1], "living-room")["reading"]["temperature_c"]
    check("set range 25..26 instant", 24.8 <= t <= 26.2, t)

    st, r = cmd({"node": "bedroom", "action": "set", "presence": False, "duration_s": 2,
                 "ranges": {"lux": 0}, "instant": True})
    n = node(get("/api/state")[1], "bedroom")
    check("set presence false + duration", st == 200 and n["presence_override"] is False)
    time.sleep(3.5)
    n = node(get("/api/state")[1], "bedroom")
    check("override expires after duration", n["presence_override"] is None
          and not n["overrides"], n["overrides"])

    st, r = cmd({"node": "kitchen", "action": "clear"})
    check("clear by room name", st == 200 and r["ok"])
    st, r = cmd({"node": "emu-kitchen", "action": "reset"})
    check("reset by node ID", st == 200 and r["ok"])

    bad = [
        ("min > max", {"node": "kitchen", "action": "set",
                       "ranges": {"co2_ppm": {"min": 900, "max": 800}}}),
        ("out of sensor range", {"node": "kitchen", "action": "set", "ranges": {"co2_ppm": 5001}}),
        ("unknown metric", {"node": "kitchen", "action": "set", "ranges": {"co3": 1}}),
        ("unknown node", {"node": "garage", "action": "clear"}),
        ("unknown action", {"node": "kitchen", "action": "dance"}),
        ("unknown scenario", {"node": "kitchen", "action": "scenario", "name": "party"}),
        ("missing node", {"action": "clear"}),
        ("missing action", {"node": "kitchen"}),
        ("negative duration", {"node": "kitchen", "action": "scenario", "name": "hot",
                               "duration_s": -1}),
        ("nothing to set", {"node": "kitchen", "action": "set"}),
        ("display to all", {"node": "all", "action": "display", "text": "x"}),
        ("display without text", {"node": "kitchen", "action": "display"}),
        ("display over 400 chars", {"node": "kitchen", "action": "display", "text": "x" * 401}),
    ]
    for name, body in bad:
        st, r = cmd(body)
        check(f"rejected: {name}", st == 400 and isinstance(r, dict) and r.get("ok") is False, r)
    st, r = http("POST", "/api/cmd", b"{not json")
    check("rejected: bad JSON", st == 400)
    st, r = http("POST", "/api/cmd", b"")
    check("rejected: empty body", st == 400)
    last = get("/api/state")[1]["last_cmd"]
    check("last_cmd shows the rejection", last and last["ok"] is False)


def test_display(mqtt_up):
    text = f"api_check {int(time.time())}"
    st, r = cmd({"node": "emu-bedroom", "action": "display", "text": text, "retain": False})
    if not mqtt_up:
        check("display: rejected while MQTT is down", st == 400 and "MQTT" in r.get("msg", ""))
        return
    check("display: published", st == 200 and r["ok"], r)
    time.sleep(2)
    msgs = get("/api/messages")[1]["messages"]
    check("display: arrives back on the node's topic",
          any(m["text"] == text and m["node_id"] == "emu-bedroom" for m in msgs))
    p = get("/api/poll?log=0&cmd=0&ev=0&msg=0")[1]
    latest = p["ai"]["latest"][[n["id"] for n in p["state"]["nodes"]].index("emu-bedroom")]
    check("display: latest per node", latest and latest["text"] == text)
    kinds = {e["kind"] for e in p["routes"]["items"]}
    check("display: display_out and display_in routes", {"display_out", "display_in"} <= kinds,
          kinds)


def test_poll():
    st, p = get("/api/poll?log=0&cmd=0&ev=0&msg=0")
    check("poll: 200 and sections", st == 200 and all(
        k in p for k in ("state", "log", "cmds", "routes", "messages", "ai")), st)
    check("poll: system log has lines", len(p["log"]["items"]) > 0)
    check("poll: route events include cmd_http",
          any(e["kind"] == "cmd_http" for e in p["routes"]["items"]))
    nxt = {k: p[k]["next"] for k in ("log", "cmds", "routes", "messages")}
    cmd({"node": "kitchen", "action": "clear"})
    q = "/api/poll?log={log}&cmd={cmds}&ev={routes}&msg={messages}".format(**nxt)
    p2 = get(q)[1]
    check("poll: incremental returns only the new command",
          len(p2["cmds"]["items"]) == 1 and p2["cmds"]["items"][0]["msg"].startswith("emu-kitchen"),
          p2["cmds"]["items"])
    p3 = get("/api/poll?log=999999&cmd=999999&ev=999999&msg=999999")[1]
    check("poll: a seq from before a reboot starts over",
          len(p3["cmds"]["items"]) > 0 and p3["cmds"]["gap"] in (True, False))


def parse_history(body):
    magic, ver, tier, nodes, series, count, interval, up, epoch, _ = struct.unpack_from(
        "<4sBBBBHHIII", body, 0)
    scales = struct.unpack_from(f"<{series}H", body, 24)
    off = 24 + 2 * series
    times = struct.unpack_from(f"<{count}I", body, off)
    off += 4 * count
    vals = struct.unpack_from(f"<{nodes * series * count}H", body, off)
    return dict(magic=magic, ver=ver, tier=tier, nodes=nodes, series=series, count=count,
                interval=interval, up=up, epoch=epoch, scales=scales, times=times, vals=vals,
                size_ok=len(body) == off + 2 * nodes * series * count)


def test_history():
    st, hdrs, body = http("GET", "/api/history?tier=10m", raw=True)
    h = parse_history(body)
    check("history 10m: header", st == 200 and h["magic"] == b"EMUH" and h["ver"] == 1
          and h["nodes"] == 3 and h["series"] == 10 and h["count"] > 0, h["count"])
    check("history 10m: size matches", h["size_ok"])
    check("history 10m: times ascending", list(h["times"]) == sorted(h["times"]))
    c = h["count"]
    temps = [v / h["scales"][0] for v in h["vals"][0:c]]  # node 0, series 0 = temperature
    check("history 10m: kitchen temperatures plausible", all(5 <= t <= 45 for t in temps),
          temps[-3:])
    st, hdrs, body = http("GET", "/api/history?tier=1h", raw=True)
    h = parse_history(body)
    check("history 1h: header", st == 200 and h["magic"] == b"EMUH" and h["tier"] == 1
          and h["size_ok"])
    st, _ = get("/api/history?tier=2h")
    check("history: unknown tier rejected", st == 400)


def test_guide():
    st, hdrs, body = http("GET", "/guide.md", raw=True)
    check("guide: 200 gzip markdown", st == 200 and hdrs.get("Content-Encoding") == "gzip"
          and body.startswith(b"# IntelliThings emulator"))
    st, hdrs, body = http("GET", "/guide.md?download=1", raw=True)
    check("guide: download sets attachment",
          "attachment" in (hdrs.get("Content-Disposition") or ""))


def test_config():
    st, c = get("/api/config")
    check("config: fields, no password", st == 200 and all(
        k in c for k in ("host", "port", "username", "password_set", "keepalive_s",
                         "publish_interval_ms", "saved", "defaults")) and "password" not in c)
    bad = [
        ("unknown field", {"colour": "red"}),
        ("port 0", {"port": 0}),
        ("host with a space", {"host": "192.168.1 .2"}),
        ("publish 500 ms", {"publish_interval_ms": 500}),
        ("keepalive 2 s", {"keepalive_s": 2}),
        ("port not a number", {"port": "1883"}),
    ]
    for name, body in bad:
        st, r = http("POST", "/api/config", body)
        check(f"config rejected: {name}", st == 400 and r.get("ok") is False, r)
    after = get("/api/config")[1]
    check("config unchanged after rejections", after == c)

    st, r = http("POST", "/api/config", {"publish_interval_ms": 5000})
    s = get("/api/state")[1]
    check("config: publish interval 5000 applies", st == 200 and r["ok"]
          and s["publish_interval_s"] == 5 and r["reconnecting"] is False, r)
    st, r = http("POST", "/api/config", {"publish_interval_ms": c["publish_interval_ms"]})
    check("config: publish interval restored", st == 200 and r["ok"])
    return c


def restore_config(original):
    """Leave the settings as found: unsaved defaults stay unsaved (also tests reset)."""
    if original["saved"]:
        st, r = http("POST", "/api/config", {"host": original["host"]})
        check("config: original host restored", st == 200 and r["ok"], r)
    else:
        st, r = http("POST", "/api/config/reset", b"{}")
        c = get("/api/config")[1]
        check("config: reset back to unsaved defaults", st == 200 and r["ok"]
              and c["saved"] is False and c["host"] == original["host"], r)


def mqtt_counts():
    s = get("/api/state")[1]
    return s["mqtt"]["emulator_connected"], [n["mqtt"] for n in s["nodes"]]


def test_broker(original):
    was_up, _ = mqtt_counts()
    try:
        st, r = http("POST", "/api/config", {"host": "192.0.2.1"})
        check("broker change: accepted, reconnecting", st == 200 and r["ok"]
              and r["reconnecting"], r)
        st, s = get("/api/state")
        check("broker change: new broker shown at once",
              s["mqtt"]["broker"] == "mqtt://192.0.2.1:%d" % original["port"], s["mqtt"])
        t0 = time.time()
        while time.time() - t0 < 20 and s["mqtt"].get("reconfiguring"):
            time.sleep(0.5)
            st, s = get("/api/state")
        check("broker change: panel answers during the switch", st == 200)
        check("broker change: switch finished, not connected",
              not s["mqtt"].get("reconfiguring") and not s["mqtt"]["emulator_connected"],
              s["mqtt"])
        print(f"     switch took {time.time() - t0:.1f} s")
    finally:
        restore_config(original)
    if was_up:
        for _ in range(30):
            emu, nodes = mqtt_counts()
            if emu and all(n == "connected" for n in nodes):
                break
            time.sleep(1)
        check("broker change: all four connections back", emu and all(
            n == "connected" for n in nodes), nodes)
    else:
        print("     (broker was down before the test; reconnection not checked)")


def main():
    global BASE
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawTextHelpFormatter)
    ap.add_argument("--host", default="192.168.1.4")
    ap.add_argument("--broker-test", action="store_true")
    a = ap.parse_args()
    BASE = "http://" + a.host

    mqtt_up = get("/api/state")[1]["mqtt"]["emulator_connected"]
    try:
        test_v01()
        test_display(mqtt_up)
        test_poll()
        test_history()
        test_guide()
        cfg = test_config()
        if a.broker_test:
            test_broker(cfg)
        else:
            restore_config(cfg)
    finally:
        cmd({"node": "all", "action": "reset"})
    passed = sum(ok for _, ok in results)
    print(f"\n{passed}/{len(results)} passed")
    sys.exit(0 if passed == len(results) else 1)


if __name__ == "__main__":
    main()
