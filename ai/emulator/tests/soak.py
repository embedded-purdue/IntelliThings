#!/usr/bin/env python3
"""Soak test for a running emulator: soak.py [--emu 192.168.1.4] [--minutes 120] [--no-switch]

Standard library only. For the whole run:
- one simulated panel polls /api/poll every 1.5 s (open the real panel too for a second one);
- demo publishing and the scenario loop are on (presets + rolls every 90 s);
- an MQTT subscriber counts every room snapshot and demo update;
- /api/state is sampled every 30 s (uptime, free heap and its lowest point, connections);
- at the halfway point the broker is switched to an unused address and back.
Demo and broker settings are put back at the end. Prints a summary; exit code 1 on a failure.
"""

import argparse
import json
import threading
import time
import urllib.request

import mqtt_check as mc

NODES = mc.NODES


def get(emu, path):
    with urllib.request.urlopen(f"http://{emu}{path}", timeout=10) as r:
        return json.load(r)


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawTextHelpFormatter)
    ap.add_argument("--emu", default="192.168.1.4")
    ap.add_argument("--minutes", type=float, default=120)
    ap.add_argument("--no-switch", action="store_true")
    a = ap.parse_args()
    emu, total = a.emu, a.minutes * 60
    broker = mc.read_config(mc.DEFAULT_CONFIG)

    s0 = get(emu, "/api/state")
    demo0 = s0["demo"]
    cfg0 = get(emu, "/api/config")
    mc.http(emu, "POST", "/api/cmd", {"action": "demo", "op": "publish", "on": True})
    mc.http(emu, "POST", "/api/cmd", {"action": "demo", "op": "loop", "on": True, "interval_s": 90,
                                      "source": "mix", "mix_pct": 30})

    sub = mc.Mqtt(*broker)
    sub.subscribe("intellithings/#")
    stop = threading.Event()
    polls = {"ok": 0, "err": 0}

    def panel():
        seq = "log=0&cmd=0&ev=0&msg=0"
        while not stop.is_set():
            try:
                p = get(emu, f"/api/poll?{seq}")
                seq = (f"log={p['log']['next']}&cmd={p['cmds']['next']}&ev={p['routes']['next']}"
                       f"&msg={p['messages']['next']}")
                polls["ok"] += 1
            except Exception:
                polls["err"] += 1
            stop.wait(1.5)
    threading.Thread(target=panel, daemon=True).start()

    t0 = time.time()
    samples, problems = [], []
    switched = a.no_switch
    print(f"soak: {a.minutes:.0f} min against {emu}; started {time.strftime('%H:%M:%S')}", flush=True)
    try:
        while time.time() - t0 < total:
            time.sleep(30)
            el = time.time() - t0
            try:
                s = get(emu, "/api/state")
            except Exception as e:
                problems.append(f"+{el / 60:.1f} min: state unreachable ({e})")
                print(problems[-1], flush=True)
                continue
            up = sum(n["mqtt"] == "connected" for n in s["nodes"])
            row = (el, s["uptime_s"], s["free_heap"], s["min_free_heap"], up, s["mqtt"]["emulator_connected"],
                   s["wifi"]["reconnects"], [n["mqtt_reconnects"] for n in s["nodes"]], s["demo"]["state"])
            if samples and row[1] < samples[-1][1]:
                problems.append(f"+{el / 60:.1f} min: the board rebooted (uptime {samples[-1][1]} -> {row[1]})")
            if (up < 3 or not row[5]) and not s["mqtt"].get("reconfiguring"):
                problems.append(f"+{el / 60:.1f} min: MQTT {up}/3 rooms, control {row[5]}")
            samples.append(row)
            if len(samples) % 10 == 0:
                print(f"+{el / 60:5.1f} min  uptime {row[1]}  heap {row[2]}  lowest {row[3]}  mqtt {up}/3  "
                      f"demo {row[8]}  polls {polls['ok']}/{polls['err']} err", flush=True)
            if not switched and el >= total / 2:
                switched = True
                ts = time.time()
                mc.http(emu, "POST", "/api/config", {"host": "192.0.2.1"})
                time.sleep(8)
                if cfg0["saved"]:
                    mc.http(emu, "POST", "/api/config", {"host": cfg0["host"]})
                else:
                    mc.http(emu, "POST", "/api/config/reset", {})
                back = mc.wait_for(lambda: sum(n["mqtt"] == "connected" for n in get(emu, "/api/state")["nodes"]) == 3, 60)
                msg = f"+{el / 60:.1f} min: broker switched away and back; " + (
                    f"all rooms back {time.time() - ts - 8:.1f} s after switching back" if back else "rooms NOT back within 60 s")
                print(msg, flush=True)
                if not back:
                    problems.append(msg)
    finally:
        stop.set()
        mc.http(emu, "POST", "/api/cmd", {"action": "demo", "op": "loop", "on": demo0["loop"],
                                          "interval_s": demo0["interval_s"], "source": demo0["loop_source"],
                                          "mix_pct": demo0["mix_pct"]})
        mc.http(emu, "POST", "/api/cmd", {"action": "demo", "op": "publish", "on": demo0["publish"]})
        sub.close()

    el = time.time() - t0
    counts = {n: sum(1 for m in sub.msgs if m[1] == f"intellithings/{n}/sensors") for n in NODES}
    homes = sum(1 for m in sub.msgs if m[1] == "intellithings/emulator/demo/home")
    expect = el / 3
    print(f"\n=== soak summary: {el / 60:.1f} min ===")
    print(f"samples {len(samples)}; panel polls {polls['ok']} ok, {polls['err']} failed")
    if samples:
        print(f"free heap {min(r[2] for r in samples)}..{max(r[2] for r in samples)}; "
              f"lowest ever {samples[-1][3]}; uptime {samples[0][1]} -> {samples[-1][1]} s")
        print(f"Wi-Fi reconnects {samples[0][6]} -> {samples[-1][6]}; room MQTT reconnects "
              f"{samples[0][7]} -> {samples[-1][7]}")
    for n, k in counts.items():
        print(f"{n}: {k} snapshots (expected ~{expect:.0f}, minus ~3 during the broker switch)")
        if k < expect * 0.95:
            problems.append(f"{n}: only {k} snapshots of ~{expect:.0f}")
    print(f"demo home updates: {homes}")
    if samples and samples[-1][3] < 50000:
        print(f"note: lowest free heap {samples[-1][3]} B is under the 50 KB target")
    if polls["err"]:
        problems.append(f"{polls['err']} panel polls failed")
    print("problems:", "none" if not problems else "")
    for p in problems:
        print("  -", p)
    raise SystemExit(1 if problems else 0)


if __name__ == "__main__":
    main()
