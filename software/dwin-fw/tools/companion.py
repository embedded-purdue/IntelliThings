#!/usr/bin/env python3
"""companion host tool"""

import argparse
import json
import math
import sys
import time
from pathlib import Path

MAP = json.loads((Path(__file__).resolve().parent.parent / "handoff/vp_map.json").read_text())
VARS = {v["name"]: v for v in MAP["variables"]}
NO_READING = MAP["no_reading"]
ACK = bytes([0x5A, 0xA5, 0x03, 0x82, 0x4F, 0x4B])


def frame(vp, words):
    data = b"".join((w & 0xFFFF).to_bytes(2, "big") for w in words)
    return bytes([0x5A, 0xA5, 3 + len(data), 0x82]) + vp.to_bytes(2, "big") + data


def text_words(s, nbytes):
    b = s.encode("utf-8")
    b = b[: nbytes - 1]
    while b and (b[-1] & 0xC0) == 0x80:  # utf8 boundary check
        b = b[:-1]
    if b and b[-1] >= 0xC0:
        b = b[:-1]
    b += b"\0" * (1 + (len(b) + 1) % 2)
    return [int.from_bytes(b[i : i + 2], "big") for i in range(0, len(b), 2)]


def encode(name, value):
    """encode variable value"""
    v = VARS[name]
    if v["type"] == "text":
        return text_words(str(value), v["bytes"])
    if isinstance(value, str):
        low = value.strip().lower()
        if low in ("none", "-") and v["default"] == NO_READING:
            return [NO_READING]
        if name == "unix_time" and low == "now":
            value = int(time.time())
        elif v["values"] and not low.lstrip("-").replace(".", "").isdigit() and not low.startswith("0x"):
            hits = [int(c) for c, m in v["values"].items()
                    if low in (m.lower(), m.lower().split(" ")[0].strip(":"), m.lower().lstrip("°"))]
            if not hits:
                raise SystemExit(f"{name}: '{value}' isn't one of: " + ", ".join(f"{c}={m}" for c, m in v["values"].items()))
            value = hits[0]
        else:
            value = float(int(low, 16)) if low.startswith("0x") else float(low)
    if "× 10" in v["unit"]:
        value = value * 10
    value = round(value)
    if v["type"] == "u32":
        return [(value >> 16) & 0xFFFF, value & 0xFFFF]
    return [value & 0xFFFF]


class Display:
    def __init__(self, port, verbose=False):
        import serial

        self.ser = serial.Serial()
        self.ser.port, self.ser.baudrate, self.ser.timeout = port, 115200, 0.05
        self.ser.dtr = self.ser.rts = False  # prevent reset pulse
        try:
            self.ser.open()
        except serial.SerialException as e:
            raise SystemExit(f"{e}\n(ls /dev/cu.usbmodem* lists the ports; close anything else using it)")
        self.verbose = verbose
        self.buf = b""  # receive byte buffer
        self.text = b""

    def _pump(self, timeout):
        """pump serial frames"""
        end = time.time() + timeout
        frames = []
        while time.time() < end and not frames:
            self.buf += self.ser.read(256)
            while True:
                i = self.buf.find(b"\x5a\xa5")
                if i < 0:
                    keep = 1 if self.buf.endswith(b"\x5a") else 0  # potential header byte
                    self._log(self.buf[: len(self.buf) - keep])
                    self.buf = self.buf[len(self.buf) - keep :]
                    break
                self._log(self.buf[:i])
                self.buf = self.buf[i:]
                if len(self.buf) < 3 or len(self.buf) < 3 + self.buf[2]:
                    break
                n = 3 + self.buf[2]
                frames.append(self.buf[:n])
                self.buf = self.buf[n:]
        return frames

    def _log(self, b):
        self.text += b
        while b"\n" in self.text:
            line, self.text = self.text.split(b"\n", 1)
            if self.verbose and line.strip():
                print("  |", line.decode(errors="replace").rstrip())

    def write(self, vp, words):
        for k in range(0, len(words), 120):
            self.ser.write(frame(vp + k, words[k : k + 120]))
            got = self._pump(0.5)
            if ACK not in got:
                raise SystemExit(f"no OK for the write at {vp + k:#06x} - is the companion firmware running on this port?")

    def set(self, **values):
        for name, value in values.items():
            self.write(VARS[name]["vp"], encode(name, value))

    def read(self, vp, n):
        words = []
        for k in range(0, n, 120):  # chunk read words
            words += self._read(vp + k, min(120, n - k))
        return words

    def _read(self, vp, n):
        self.ser.write(bytes([0x5A, 0xA5, 0x04, 0x83, vp >> 8, vp & 0xFF, n]))
        end = time.time() + 1.0
        while time.time() < end:
            for f in self._pump(0.2):
                if f[3] == 0x83 and int.from_bytes(f[4:6], "big") == vp:
                    return [int.from_bytes(f[7 + 2 * i : 9 + 2 * i], "big") for i in range(f[6])]
        raise SystemExit(f"no answer to the read at {vp:#06x}")

    def get(self, name):
        v = VARS[name]
        words = self.read(v["vp"], v["words"])
        if v["type"] == "text":
            b = b"".join(w.to_bytes(2, "big") for w in words).split(b"\0")[0]
            return b.decode(errors="replace")
        if v["type"] == "u32":
            return words[0] << 16 | words[1]
        w = words[0]
        if w == NO_READING and v["default"] == NO_READING:
            return "no reading"
        if v["type"] == "i16" and w >= 0x8000:
            w -= 0x10000
        if "× 10" in v["unit"]:
            return w / 10
        meaning = (v["values"] or {}).get(str(w))
        return f"{w} ({meaning})" if meaning else w


def play(d):
    """scripted demo sequence"""
    t0 = time.time()

    block = ["temperature", "humidity", "voc_index", "co2", "lux", "distance", "pm1_0", "pm2_5", "pm10", "presence"]

    def readings(co2, pm=5, temp=22.3):
        """pack sensor block"""
        jitter = math.sin(time.time() - t0)
        values = dict(temperature=round(temp + 0.1 * jitter, 1), humidity=round(41 + jitter, 1), voc_index=110,
                      co2=co2, lux=320, distance=1180, pm1_0=max(1, pm // 2), pm2_5=pm, pm10=pm + 2, presence=1)
        d.write(VARS["temperature"]["vp"], [w for name in block for w in encode(name, values[name])])

    def hold(seconds, co2=640, pm=5, every=1.0, **kw):
        end = time.time() + seconds
        while time.time() < end:
            readings(co2 if not callable(co2) else co2(), pm if not callable(pm) else pm(), **kw)
            d.set(synced_ago=0)
            time.sleep(every)

    print("boot checklist")
    d.set(inputs_reset=0x5AA5)
    steps = [("boot1", "6 of 6 responding"), ("boot2", "HomeNet · −52 dBm"),
             ("boot3", "intellithings/living-room"), ("boot4", "MCP ready")]
    for name, _ in steps:  # reset step states
        d.write(VARS[f"{name}_state"]["vp"], [0])
    d.set(boot_version="v0.1.0 · node living-room-01")
    t = time.localtime()
    if not VARS["night_end"]["default"] <= t.tm_hour * 60 + t.tm_min < VARS["night_start"]["default"]:
        print("(it's night here, so the night page is off for this demo; `reset` turns it back on)")
        d.set(night_start=0xFFFF)
    for name, detail in steps:
        d.write(VARS[f"{name}_state"]["vp"], [1])
        time.sleep(0.9)
        d.set(**{f"{name}_detail": detail})
        d.write(VARS[f"{name}_state"]["vp"], [2])
    d.set(unix_time="now", utc_offset=time.localtime().tm_gmtoff // 60, wifi=1, hub=1, synced_ago=0)
    rooms = {"room2": [1, 233, 480, 700, 6, 120, 1], "room3": [1, 239, 390, 610, 4, 95, 0]}
    for room, words in rooms.items():
        d.write(VARS[f"{room}_online"]["vp"], words)
    d.set(room5_note="Lights on")

    print("home: calm")
    d.set(message_ts=int(time.time()) - 120,
          message_text="Everything looks comfortable. Air is fresh, and the room is holding at 72°F.")
    hold(10)

    print("CO₂ rising, the agent thinks and acts")
    start = time.time()
    hold(8, co2=lambda: int(800 + (time.time() - start) * 45))
    d.set(thinking="Looking at the air in here…")
    hold(3, co2=1170)
    d.write(VARS["chip1_device"]["vp"], [0])
    d.set(chip1_label="Fan", chip1_value="40%", message_ts=int(time.time()),
          message_text="CO₂ hit 1,180 ppm, so I turned the fan up to 40%. Opening a window would help too.")
    d.set(toast_device=0, toast_title="Fan set to 40%", toast_detail="Living Room · CO₂ was 1,180 ppm")
    d.set(toast_trigger=1)  # trigger toast display
    hold(10, co2=lambda: int(max(700, 1180 - (time.time() - start - 11) * 30)))

    print("sensors and rooms pages")
    d.set(screen="sensors")
    hold(6, co2=760)
    d.set(screen="rooms")
    hold(6, co2=740)
    d.set(screen="automatic")

    print("alert: particles in the kitchen")
    d.write(VARS["room2_online"]["vp"], [1, 233, 480, 700, 87, 140, 1])
    d.set(alert_metric=3, alert_value=87, alert_since=120, alert_title="Smoke or cooking particles",
          alert_place="Kitchen", alert_normal="Normal is under 12",
          alert_message="Probably the stove. I'll tell you when the air clears.")
    for name, device, label, value in [("alert_done1", 0, "Range hood", "High"), ("alert_done2", 3, "Purifier", "Turbo"),
                                       ("alert_ask", 6, "Close the bedroom door", "")]:
        d.write(VARS[f"{name}_device"]["vp"], [device])
        d.set(**{f"{name}_label": label, f"{name}_value": value})
    d.set(alert_active=1)
    hold(10, co2=720)
    d.set(alert_active=0)
    d.write(VARS["room2_online"]["vp"], rooms["room2"])

    print("hub offline, then back")
    d.set(hub="offline", retry_in=12)
    hold(8, co2=700)
    d.set(hub="online", synced_ago=0)
    d.set(message_ts=int(time.time()), message_text="Back online. The air cleared up in the kitchen, so I turned the range hood off.")
    hold(10, co2=690)
    print("done: the display stays live; `reset` returns it to the demo")


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--port", help="the companion's USB port (e.g. /dev/cu.usbmodem211101) or a USB-UART")
    ap.add_argument("-v", "--verbose", action="store_true", help="also print the companion's log")
    sub = ap.add_subparsers(dest="cmd", required=True)
    p = sub.add_parser("set", help="write variables: name=value ...")
    p.add_argument("pairs", nargs="+")
    p = sub.add_parser("message", help="the AI message (message_ts = now)")
    p.add_argument("text")
    p = sub.add_parser("get", help="read variables back")
    p.add_argument("names", nargs="+")
    sub.add_parser("status", help="the display's read-only status words")
    sub.add_parser("reset", help="every input back to its power-up value")
    sub.add_parser("play", help="a scripted live demo")
    sub.add_parser("list", help="every variable name")
    a = ap.parse_args(argv)

    if a.cmd == "list":
        for v in MAP["variables"]:
            print(f"0x{v['vp']:04X}  {v['name']:24} {v['type']:5} {v['unit']}")
        return
    if not a.port:
        raise SystemExit("--port is required (ls /dev/cu.usbmodem*)")
    d = Display(a.port, a.verbose)
    if a.cmd == "set":
        for pair in a.pairs:
            name, _, value = pair.partition("=")
            if name not in VARS:
                raise SystemExit(f"unknown variable {name!r} (try: list)")
            if VARS[name]["read_only"]:
                raise SystemExit(f"{name} is read-only: the display writes it")
            d.write(VARS[name]["vp"], encode(name, value))
        print("ok")
    elif a.cmd == "message":
        d.set(message_ts=int(time.time()), message_text=a.text)
        print("ok")
    elif a.cmd == "get":
        for name in a.names:
            print(f"{name} = {d.get(name)!r}")
    elif a.cmd == "status":
        for name in ["interface_version", "panel_state", "screen_shown", "mode_now", "writes_ok", "writes_rejected"]:
            print(f"{name:18} {d.get(name)}")
    elif a.cmd == "reset":
        d.set(inputs_reset=0x5AA5)
        print("ok: power-up values, demo again")
    elif a.cmd == "play":
        play(d)


if __name__ == "__main__":
    sys.exit(main())
