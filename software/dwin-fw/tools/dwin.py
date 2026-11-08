#!/usr/bin/env python3
"""dwin uart tool"""

import argparse
import json
import sys
import time
from pathlib import Path

HEADER = b"\x5a\xa5"
WRITE, READ = 0x82, 0x83

# system registers
VERSION = 0x000F
PIC_NOW = 0x0014
LED_CONFIG = 0x0082
PIC_SET = 0x0084
SYS_RESET = 0x0004

CRC = False  # optional crc flag


def crc16(data):
    crc = 0xFFFF
    for b in data:
        crc ^= b
        for _ in range(8):
            crc = (crc >> 1) ^ 0xA001 if crc & 1 else crc >> 1
    return crc


def frame(cmd, payload):
    body = bytes([cmd]) + payload
    if CRC:
        body += crc16(body).to_bytes(2, "little")
    if len(body) > 0xFF:
        raise ValueError("frame too long")
    return HEADER + bytes([len(body)]) + body


def write_vp(vp, data):
    if len(data) % 2:
        raise ValueError("VP writes are whole 16-bit words")
    return frame(WRITE, vp.to_bytes(2, "big") + data)


def read_vp(vp, words):
    return frame(READ, vp.to_bytes(2, "big") + bytes([words]))


def words(*values):
    return b"".join(v.to_bytes(2, "big") for v in values)


class Panel:
    def __init__(self, port, baud, verbose=False):
        import serial  # optional pyserial import

        self.s = serial.Serial(port, baud, timeout=0.3)
        self.verbose = verbose

    def tx(self, data):
        if self.verbose:
            print(">", data.hex(" "))
        self.s.write(data)
        self.s.flush()

    def rx_frame(self, timeout=0.5):
        """read reply frame"""
        deadline = time.monotonic() + timeout
        buf = b""
        while time.monotonic() < deadline:
            buf += self.s.read(64)
            start = buf.find(HEADER)
            if start >= 0 and len(buf) >= start + 3:
                n = buf[start + 2]
                if len(buf) >= start + 3 + n:
                    f = buf[start:start + 3 + n]
                    if self.verbose:
                        print("<", f.hex(" "))
                    return f
        return None

    def read(self, vp, n):
        self.s.reset_input_buffer()
        self.tx(read_vp(vp, n))
        while True:
            f = self.rx_frame()
            if f is None:
                return None
            if f[3] == READ and int.from_bytes(f[4:6], "big") == vp:
                data = f[7:7 + 2 * f[6]]
                return [int.from_bytes(data[i:i + 2], "big") for i in range(0, len(data), 2)]

    def write(self, vp, data):
        self.tx(write_vp(vp, data))


def parse_int(s):
    return int(s, 0)


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--port", required=True)
    ap.add_argument("--baud", type=int, default=115200)
    ap.add_argument("-v", "--verbose", action="store_true")
    ap.add_argument("--crc", action="store_true", help="append CRCs (if the panel's CFG has CRC on)")
    sub = ap.add_subparsers(dest="cmd", required=True)
    sub.add_parser("info", help="read firmware versions and the current page")
    p = sub.add_parser("page", help="switch page")
    p.add_argument("page", type=parse_int)
    p = sub.add_parser("bright", help="backlight 0-100 (the panel flickers below ~30)")
    p.add_argument("level", type=parse_int)
    p = sub.add_parser("write", help="write 16-bit words to a VP")
    p.add_argument("vp", type=parse_int)
    p.add_argument("values", type=parse_int, nargs="+")
    p = sub.add_parser("text", help="write a text string (GBK/ASCII) to a VP, 0xFFFF-terminated")
    p.add_argument("vp", type=parse_int)
    p.add_argument("text")
    p = sub.add_parser("read", help="read words from a VP")
    p.add_argument("vp", type=parse_int)
    p.add_argument("count", type=parse_int)
    p = sub.add_parser("send", help="send hex frames from a file ('-' for stdin)")
    p.add_argument("file")
    p.add_argument("--speed", type=float, default=1.0, help="replay speed multiplier")
    sub.add_parser("reset", help="reboot the panel")
    a = ap.parse_args(argv)
    global CRC
    CRC = a.crc

    panel = Panel(a.port, a.baud, a.verbose)
    if a.cmd == "info":
        v = panel.read(VERSION, 1)
        if v is None and not CRC:
            CRC = True  # retry with crc
            v = panel.read(VERSION, 1)
        if v is None:
            sys.exit("no reply: check the TTL jumper, TX/RX crossover, ground and baud rate")
        page = panel.read(PIC_NOW, 1)
        print(f"GUI (DGUS) version {v[0] >> 8:#04x}, OS (8051) version {v[0] & 0xFF:#04x}"
              f"{' (CRC on)' if CRC else ''}")
        print(f"current page {page[0] if page else '?'}")
        manifest = Path(__file__).resolve().parent.parent / "build/manifest.json"
        if manifest.exists():
            vp = json.loads(manifest.read_text())["signature_vp"]
            sig = panel.read(vp, 1)
            if sig and sig[0] == 0xC0DE:
                print("companion DWIN_SET loaded")
            else:
                print("the companion DWIN_SET is not loaded (or is from a different build)")
    elif a.cmd == "page":
        panel.write(PIC_SET, words(0x5A01, a.page))
    elif a.cmd == "bright":
        level = max(0, min(100, a.level))
        # backlight config tuple
        panel.write(LED_CONFIG, bytes([level, level]) + words(0))
    elif a.cmd == "write":
        panel.write(a.vp, words(*a.values))
    elif a.cmd == "text":
        data = a.text.encode("gbk") + b"\xff\xff"
        if len(data) % 2:
            data += b"\x00"
        panel.write(a.vp, data)
    elif a.cmd == "read":
        print(" ".join(f"{w:#06x}" for w in (panel.read(a.vp, a.count) or [])) or "no reply")
    elif a.cmd == "send":
        src = sys.stdin if a.file == "-" else open(a.file)
        for line in src:
            line = line.strip()
            if not line:
                continue
            if line.startswith("# sleep"):
                time.sleep(int(line.split()[2]) / 1000 / a.speed)
                continue
            if line.startswith("#"):
                print(line)
                continue
            panel.tx(bytes.fromhex(line))
    elif a.cmd == "reset":
        panel.write(SYS_RESET, bytes([0x55, 0xAA, 0x5A, 0xA5]))


if __name__ == "__main__":
    main()
