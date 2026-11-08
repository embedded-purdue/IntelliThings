"""dwin vp map"""

import calendar
import csv
import html
import json
import re
from dataclasses import dataclass, field
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
VERSION = 0x0100  # interface version
BASE, WORDS = 0x1000, 0x800  # input memory range
NO_READING = 0x8000
RESET_MAGIC = 0x5AA5
AI_TEXT_FIT = (110, 80)  # text fit limits

# value lists

HUB = [(0, "connecting"), (1, "online"), (2, "offline")]
DEVICES = [(0, "fan"), (1, "light"), (2, "thermostat"), (3, "purifier"), (4, "humidifier"),
           (5, "window"), (6, "door"), (7, "lock")]
SCREENS = [(0, "automatic"), (1, "home"), (2, "sensors"), (3, "rooms"), (4, "alert"),
           (5, "night"), (6, "away"), (7, "boot"), (8, "setup"), (9, "test pattern")]
METRICS = [(0, "temperature"), (1, "humidity"), (2, "CO₂"), (3, "PM2.5"), (4, "VOC index"),
           (5, "light")]
STEP_STATES = [(0, "pending"), (1, "running"), (2, "done"), (3, "failed"), (0xFFFF, "not used")]
MODES = [(0, "auto: demo until the first write, then live"), (1, "live only"), (2, "demo only")]
ROOM_NAMES = ["Living Room", "Kitchen", "Bedroom", "Office", "Entry"]  # room names
BOOT_NAMES = ["Display", "Sensors", "Wi-Fi", "MQTT broker", "Home Assistant"]


@dataclass(frozen=True)
class Var:
    name: str  # lower snake name
    vp: int  # address or offset
    kind: str  # data type
    unit: str = ""
    desc: str = ""
    default: int = 0  # default word value
    bytes: int = 0  # byte capacity
    values: list = field(default_factory=list)  # enum variants
    mqtt: str = ""
    read_only: bool = False

    @property
    def words(self):
        return {"u32": 2, "text": self.bytes // 2}.get(self.kind, 1)


@dataclass(frozen=True)
class Struct:
    """repeated field layout"""

    array: str  # address array
    module: str  # offset module
    stride: int
    fields: list
    entries: list  # struct entries


@dataclass(frozen=True)
class Section:
    key: str
    title: str
    note: str
    parts: list  # map parts


def text(name, vp, nbytes, desc, **kw):
    return Var(name, vp, "text", "", desc, bytes=nbytes, **kw)


def reading(name, vp, unit, desc, mqtt):
    return Var(name, vp, "i16" if name == "temperature" else "u16", unit, desc,
               default=NO_READING, mqtt=mqtt)


CHIP_FIELDS = [
    Var("device", 0, "enum", "", "Icon", default=0xFFFF, values=DEVICES + [(0xFFFF, "no chip")]),
    text("label", 1, 28, "“Fan”, “Close the bedroom door”"),
    text("value", 0x10, 12, "After an arrow: “40%”. Empty for none."),
]

ROOM_FIELDS = [
    Var("online", 0, "enum", "", "Automatic: this node's own room fills itself from the sensor "
        "block; other rooms show offline.", default=0xFFFF,
        values=[(0, "offline"), (1, "online"), (0xFFFF, "automatic")]),
    reading("temperature", 1, "°C × 10", "", "temperature_c"),
    reading("humidity", 2, "%RH × 10", "", "humidity_pct"),
    reading("co2", 3, "ppm", "", "co2_ppm"),
    reading("pm2_5", 4, "µg/m³", "", "pm2_5_ugm3"),
    reading("voc_index", 5, "index", "", "voc_index"),
    Var("presence", 6, "enum", "", "", values=[(0, "nobody"), (1, "someone's there")], mqtt="presence"),
    text("note", 7, 16, "Rooms without a node: “Lights on”"),
]

STEP_FIELDS = [
    Var("state", 0, "enum", "", "", default=0xFFFF, values=STEP_STATES),
    text("detail", 1, 32, "Shown once done: “HomeNet · −52 dBm”"),
]

SECTIONS = [
    Section("sensors", "Sensor readings",
        "Same order as the MQTT <code>…/sensors</code> snapshot, so one frame of 10 words sends "
        "them all. Write every 0.5–1 s or on change. <code>0x8000</code> = no reading (sensor "
        "missing or faulted): the tile shows <i>Waiting</i>.", [
        reading("temperature", 0x1000, "°C × 10, signed", "224 = 22.4 °C. Shown in °F or °C (see <code>units</code>).", "temperature_c"),
        reading("humidity", 0x1001, "%RH × 10", "412 = 41.2 %", "humidity_pct"),
        reading("voc_index", 0x1002, "index 1–500", "SGP40 VOC index (≈100 is typical)", "voc_index"),
        reading("co2", 0x1003, "ppm", "SCD41, 400–5000", "co2_ppm"),
        reading("lux", 0x1004, "lux, whole", "BH1750. Clamp brighter readings to 32767.", "lux"),
        reading("distance", 0x1005, "mm", "VL53L0X. Accepted, not shown yet.", "distance_mm"),
        reading("pm1_0", 0x1006, "µg/m³", "Accepted, not shown yet.", "pm1_0_ugm3"),
        reading("pm2_5", 0x1007, "µg/m³", "PMS5003, 0–500", "pm2_5_ugm3"),
        reading("pm10", 0x1008, "µg/m³", "Accepted, not shown yet.", "pm10_ugm3"),
        Var("presence", 0x1009, "enum", "", "LD2410C OUT. With no sensor (<code>0x8000</code>) "
            "the screen never switches to the away page.", default=NO_READING,
            values=[(0, "nobody"), (1, "someone's there")], mqtt="presence"),
    ]),
    Section("connection", "Connection", "Shown in the top bar and on the offline card.", [
        Var("wifi", 0x1010, "enum", "", "Wi-Fi icon", values=[(0, "disconnected"), (1, "connected")]),
        Var("hub", 0x1011, "enum", "", "MQTT broker / Home Assistant link. <i>offline</i> turns "
            "the AI card into the offline card.", values=HUB),
        Var("synced_ago", 0x1012, "u16", "seconds", "Since the last good exchange with the hub. "
            "Write 0 after each publish or message; the display counts up (“Synced 2 min ago”). "
            "<code>0xFFFF</code> = never.", default=0xFFFF),
        Var("retry_in", 0x1013, "u16", "seconds", "To the next reconnect attempt; the offline "
            "card counts down (“retry in 0:42”). 0 = no countdown."),
    ]),
    Section("clock", "Clock",
        "Write once after SNTP sync (again every hour is fine); the display keeps time from there.", [
        Var("unix_time", 0x1020, "u32", "seconds since 1970 UTC", "High word first. 0 = unknown: no clock shown."),
        Var("utc_offset", 0x1022, "i16", "minutes", "Local offset from UTC: −240 = EDT, −300 = EST."),
    ]),
    Section("settings", "Settings", "Optional; the defaults suit the bench.", [
        Var("units", 0x1030, "enum", "", "Temperature unit", values=[(0, "°F"), (1, "°C")]),
        Var("clock_24h", 0x1031, "enum", "", "Clock format", values=[(0, "12-hour"), (1, "24-hour")]),
        Var("room", 0x1032, "enum", "", "This node's room: its name in the top bar and the "
            "<i>This display</i> tile on the rooms page.", values=list(enumerate(ROOM_NAMES))),
        Var("night_start", 0x1033, "u16", "minutes after midnight", "Night page from 22:30. "
            "<code>0xFFFF</code> = never.", default=1350),
        Var("night_end", 0x1034, "u16", "minutes after midnight", "Night page until 6:30", default=390),
        Var("away_after", 0x1035, "u16", "seconds", "Away page after this long without presence. 0 = never.", default=600),
        Var("rotate_pages", 0x1036, "enum", "", "Off keeps the AI message on screen at all times "
            "(the team's layout rule).", values=[(0, "off"), (1, "sensors and rooms pages, 10 s each per minute")]),
        Var("screen", 0x1037, "enum", "", "Force a screen, for testing", values=SCREENS),
        Var("mode", 0x1038, "enum", "", "Demo loop or live data", values=MODES),
        Var("led_brightness", 0x1039, "u16", "%", "Light bar, as a share of its safe maximum (0–100)", default=100),
        Var("inputs_reset", 0x103F, "u16", "command", f"Write <code>0x{RESET_MAGIC:04X}</code> to put every "
            "input back to its power-up value. Do it when the node boots."),
        text("room_name", 0x1040, 24, "Overrides the top bar's room name. Empty = the name of <code>room</code>."),
    ]),
    Section("status", "Display status",
        "Read with <code>5A A5 04 83 10 F0 06</code> to check the display is alive and what it shows.", [
        Var("interface_version", 0x10F0, "u16", "", f"<code>0x{VERSION:04X}</code> = v{VERSION >> 8}.{VERSION & 0xFF}", default=VERSION, read_only=True),
        Var("panel_state", 0x10F1, "enum", "", "The DWIN panel behind the display",
            values=[(0, "no panel"), (1, "panel without the companion files"), (2, "drawing")], read_only=True),
        Var("screen_shown", 0x10F2, "enum", "", "Same codes as <code>screen</code>", values=SCREENS[1:], read_only=True),
        Var("mode_now", 0x10F3, "enum", "", "", values=[(0, "demo"), (1, "live")], read_only=True),
        Var("writes_ok", 0x10F4, "u16", "count", "Write frames applied (wraps)", read_only=True),
        Var("writes_rejected", 0x10F5, "u16", "count", "Write frames outside the map or to read-only words", read_only=True),
    ]),
    Section("message", "AI message",
        "The card shows the text until a different text arrives: no timeout, and rewriting the "
        f"same text changes nothing. About <b>{AI_TEXT_FIT[0]} characters</b> fit (4 lines; "
        f"{AI_TEXT_FIT[1]} with action chips); longer text is cut with “…”. A new text also "
        "clears <code>thinking</code>.", [
        Var("message_ts", 0x10FE, "u32", "seconds since 1970 UTC", "The payload's <code>ts</code>, "
            "for “2 min ago”. 0 = none: counted from arrival.", mqtt="ts"),
        text("message_text", 0x1100, 512, "The payload's <code>text</code>. Empty = the card's idle "
             "line. 512 bytes take 3 frames: at 0x1100, 0x1178 and 0x11F0.", mqtt="text"),
        text("thinking", 0x1200, 64, "While not empty the card shows the spinning orb and this "
             "line (“Reading 6 sensors across 3 rooms…”). Write empty, or a new message, to end it."),
        Struct("MESSAGE_CHIPS", "chip", 0x20, CHIP_FIELDS, [
            ("chip1", 0x1220, "Chip 1", False), ("chip2", 0x1240, "Chip 2", False)]),
    ]),
    Section("toast", "Toast",
        "A banner over the home page for 5.5 s, for an action the user should notice.", [
        Var("toast_trigger", 0x1300, "u16", "", "Write a new value (count up) to show the toast. "
            "Fill the fields in the same frame or before."),
        Var("toast_device", 0x1301, "enum", "", "Icon", values=DEVICES),
        text("toast_title", 0x1302, 40, "“Lights dimmed to 30%”"),
        text("toast_detail", 0x1316, 60, "“Living Room · after sunset, someone's here”"),
    ]),
    Section("alert", "Alert", "Takes over the screen while active.", [
        Var("alert_active", 0x1400, "enum", "", "", values=[(0, "no alert"), (1, "alert page")]),
        Var("alert_metric", 0x1401, "enum", "", "The reading in trouble", values=METRICS),
        Var("alert_value", 0x1402, "i16", "as the sensor block", "Shown big. Same scaling as "
            "that reading: temperature °C × 10, humidity × 10, others whole."),
        Var("alert_since", 0x1403, "u16", "seconds", "Since it started; the display counts up."),
        text("alert_title", 0x1404, 40, "“Smoke or cooking particles”"),
        text("alert_place", 0x1418, 24, "“Kitchen”"),
        text("alert_normal", 0x1424, 28, "“Normal is under 12”"),
        text("alert_message", 0x1432, 160, "“Probably the stove. I'll tell you when the air clears.”"),
        Struct("ALERT_CHIPS", "chip", 0x20, CHIP_FIELDS, [
            ("alert_done1", 0x1490, "Done 1", False), ("alert_done2", 0x14B0, "Done 2", False),
            ("alert_ask", 0x14D0, "User request", False)]),
    ]),
    Section("rooms", "Rooms",
        "One entry per room in the rooms page's layout. Fill them from the other nodes' "
        "<code>…/sensors</code> snapshots; the display works out each room's colour and word "
        "(“Air fresh”, “Particles up”) from the readings.", [
        Struct("ROOMS", "room", 0x10, ROOM_FIELDS,
               [(f"room{i + 1}", 0x1500 + 0x10 * i, n, False) for i, n in enumerate(ROOM_NAMES)]),
    ]),
    Section("boot", "Boot checklist",
        "The boot page shows while any step is pending or running, and for 1.5 s after the "
        "last one finishes. Write all four of yours (done for any you don't use). Step 0 is the "
        "display's own.", [
        Struct("BOOT_STEPS", "step", 0x20, STEP_FIELDS,
               [(f"boot{i}", 0x1600 + 0x20 * i, n, i == 0) for i, n in enumerate(BOOT_NAMES)]),
        text("boot_version", 0x16A0, 40, "Bottom line of the boot page: “v0.1.0 · node living-room-01”"),
    ]),
    Section("setup", "Wi-Fi setup", "The setup page (join this network, with a QR code) shows while the SSID isn't empty.", [
        text("setup_ssid", 0x1700, 32, "“IntelliThings-3F2A”"),
        text("setup_password", 0x1710, 32, "“48213907”"),
        text("setup_url", 0x1720, 32, "“192.168.4.1”"),
    ]),
]


# flattened view


@dataclass(frozen=True)
class Entry:
    name: str
    vp: int
    var: Var
    section: str
    read_only: bool


def entries():
    out = []
    for s in SECTIONS:
        for p in s.parts:
            if isinstance(p, Var):
                out.append(Entry(p.name, p.vp, p, s.key, p.read_only))
            else:
                for name, vp, _, ro in p.entries:
                    for f in p.fields:
                        out.append(Entry(f"{name}_{f.name}", vp + f.vp, f, s.key, ro or f.read_only))
    return out


def span(section):
    es = [e for e in entries() if e.section == section.key]
    return min(e.vp for e in es), max(e.vp + e.var.words for e in es) - 1


def check():
    from . import pages

    assert ROOM_NAMES == [r[0] for r in pages.ROOMS], "ROOM_NAMES must follow dgus/pages.py ROOMS"
    used = {}
    for e in entries():
        for w in range(e.vp, e.vp + e.var.words):
            assert BASE <= w < BASE + WORDS, f"{e.name} at {w:#06x} is outside the map"
            assert w not in used, f"{e.name} overlaps {used[w]} at {w:#06x}"
            used[w] = e.name
    for s in SECTIONS:
        for p in s.parts:
            if isinstance(p, Struct):
                assert all(f.vp + f.words <= p.stride for f in p.fields), p.array
    spans = sorted(span(s) for s in SECTIONS)
    assert all(a[1] < b[0] for a, b in zip(spans, spans[1:])), "sections overlap"
    return used


def defaults():
    """power-up default values"""
    out = []
    for e in entries():
        if e.var.default:
            out += [(e.vp + w, e.var.default) for w in range(e.var.words)]
    return sorted(out)


# frame encoding


def frame(vp, words):
    data = b"".join((w & 0xFFFF).to_bytes(2, "big") for w in words)
    return bytes([0x5A, 0xA5, 3 + len(data), 0x82]) + vp.to_bytes(2, "big") + data


def text_words(s, nbytes):
    b = s.encode("utf-8")[: nbytes - 1] + b"\0"
    b += b"\0" * (len(b) % 2)
    return [int.from_bytes(b[i : i + 2], "big") for i in range(0, len(b), 2)]


def hexs(b):
    return " ".join(f"{x:02X}" for x in b)


def examples():
    t = calendar.timegm((2026, 10, 18, 18, 5, 0))
    return [
        ("CO₂ = 812 ppm", frame(0x1003, [812])),
        ("Temperature 22.4 °C (224)", frame(0x1000, [224])),
        ("Temperature −5.0 °C (−50 = 0xFFCE)", frame(0x1000, [-50])),
        ("All ten readings: 22.4 °C, 41.2 %, VOC 118, 642 ppm, 320 lx, 1180 mm, PM 3/6/7, someone there",
         frame(0x1000, [224, 412, 118, 642, 320, 1180, 3, 6, 7, 1])),
        ("PM2.5 sensor faulted", frame(0x1007, [NO_READING])),
        ("Wi-Fi connected, hub online, just synced", frame(0x1010, [1, 1, 0])),
        ("Clock: 2026-10-18 14:05 EDT (18:05 UTC, offset −240)", frame(0x1020, [t >> 16, t & 0xFFFF, -240])),
        ("AI message “Hi!”", frame(0x1100, text_words("Hi!", 512))),
        ("Clear the AI message", frame(0x1100, [0])),
        ("Reset every input (when the node boots)", frame(0x103F, [RESET_MAGIC])),
        ("Read the display status (6 words)", bytes([0x5A, 0xA5, 0x04, 0x83, 0x10, 0xF0, 0x06])),
    ]


# writers


def plain(s):
    return html.unescape(re.sub(r"<[^>]+>", "", s))


def fmt_vp(vp, words=1):
    return f"0x{vp:04X}" if words == 1 else f"0x{vp:04X}–0x{vp + words - 1:04X}"


def fmt_code(c):
    return f"0x{c:04X}" if c >= 0x8000 else str(c)


def fmt_default(v):
    if v.kind == "text":
        return "empty"
    d = f"0x{v.default:04X}" if v.default in (NO_READING, 0xFFFF, VERSION) else str(v.default)
    meaning = dict(v.values).get(v.default)
    return f"{d} ({meaning})" if meaning else d


def type_label(v):
    return {"enum": "u16"}.get(v.kind, v.kind)


def values_html(v):
    if not v.values:
        return ""
    items = "".join(f"<li><code>{fmt_code(c)}</code> {html.escape(m)}</li>" for c, m in v.values)
    return f"<ul class=values>{items}</ul>"


def write_csv(path):
    with path.open("w", newline="") as f:
        w = csv.writer(f)
        w.writerow(["vp", "last_vp", "words", "name", "section", "type", "unit", "default",
                    "access", "mqtt_field", "values", "description"])
        for e in entries():
            v = e.var
            w.writerow([f"0x{e.vp:04X}", f"0x{e.vp + v.words - 1:04X}", v.words, e.name, e.section,
                        type_label(v) + (f" ({v.bytes} bytes)" if v.kind == "text" else ""),
                        v.unit, fmt_default(v), "read" if e.read_only else "read/write", v.mqtt,
                        "; ".join(f"{fmt_code(c)}={m}" for c, m in v.values), plain(v.desc)])


def write_json(path):
    doc = {
        "version": f"{VERSION >> 8}.{VERSION & 0xFF}",
        "base": BASE,
        "words": WORDS,
        "no_reading": NO_READING,
        "reset_magic": RESET_MAGIC,
        "frame": "5A A5 <3 + data bytes> 82 <VP hi> <VP lo> <data: big-endian words>",
        "variables": [
            {"name": e.name, "vp": e.vp, "words": e.var.words, "section": e.section,
             "type": type_label(e.var), "unit": e.var.unit, "default": e.var.default,
             "bytes": e.var.bytes or None, "read_only": e.read_only, "mqtt": e.var.mqtt or None,
             "values": {str(c): m for c, m in e.var.values} or None,
             "description": plain(e.var.desc)}
            for e in entries()
        ],
    }
    path.write_text(json.dumps(doc, indent=1, ensure_ascii=False) + "\n")


def rust_consts():
    lines, modules = [], set()
    for s in SECTIONS:
        lines += ["", f"// {plain(s.title).lower()}"]
        for p in s.parts:
            if isinstance(p, Var):
                lines.append(f"pub const {p.name.upper()}: u16 = {p.vp:#06x};")
                if p.kind == "text":
                    lines.append(f"pub const {p.name.upper()}_WORDS: usize = {p.words};")
                continue
            for name, vp, label, _ in p.entries:
                lines.append(f"pub const {name.upper()}: u16 = {vp:#06x}; // {label.lower()}")
            names = ", ".join(n.upper() for n, _, _, _ in p.entries)
            lines.append(f"pub const {p.array}: [u16; {len(p.entries)}] = [{names}];")
            if p.module not in modules:
                modules.add(p.module)
                lines.append("/// entry field offsets")
                lines.append(f"pub mod {p.module} {{")
                for f in p.fields:
                    lines.append(f"    pub const {f.name.upper()}: u16 = {f.vp:#x};")
                    if f.kind == "text":
                        lines.append(f"    pub const {f.name.upper()}_WORDS: usize = {f.words};")
                lines.append("}")
    return lines


def write_rust(path):
    lines = [
        "//! input vp map",
        "",
        f"pub const BASE: u16 = {BASE:#06x};",
        f"pub const WORDS: usize = {WORDS:#x};",
        f"pub const VERSION: u16 = {VERSION:#06x};",
        "/// missing sensor reading",
        f"pub const NO_READING: u16 = {NO_READING:#06x};",
        f"pub const RESET_MAGIC: u16 = {RESET_MAGIC:#06x};",
        "/// rooms in order",
        f"pub const ROOM_NAMES: [&str; {len(ROOM_NAMES)}] = [{', '.join(json.dumps(n) for n in ROOM_NAMES)}];",
        *rust_consts(),
        "",
        "/// power-up non-zero words",
        "pub const DEFAULTS: &[(u16, u16)] = &[",
        *[f"    ({vp:#06x}, {val:#06x})," for vp, val in defaults()],
        "];",
        "",
        "/// display-only words",
        "pub const READ_ONLY: &[(u16, usize)] = &[",
        *[f"    ({e.vp:#06x}, {e.var.words})," for e in entries() if e.read_only],
        "];",
    ]
    path.write_text("\n".join(lines) + "\n")


TEAM_RUST_HEAD = """//! input vp map
#![allow(dead_code)]

pub const BASE: u16 = {base:#06x};
pub const NO_READING: u16 = {no_reading:#06x};
pub const RESET_MAGIC: u16 = {reset:#06x};
/// max data bytes
pub const MAX_DATA: usize = 240;
/// max words read
pub const MAX_READ: u8 = 125;
"""

TEAM_RUST_TAIL = """
/// encode write frame
pub fn write_frame(out: &mut [u8], vp: u16, words: &[u16]) -> usize {
    assert!(words.len() * 2 <= MAX_DATA);
    out[0] = 0x5a;
    out[1] = 0xa5;
    out[2] = (3 + 2 * words.len()) as u8;
    out[3] = 0x82;
    out[4..6].copy_from_slice(&vp.to_be_bytes());
    for (i, w) in words.iter().enumerate() {
        out[6 + 2 * i..8 + 2 * i].copy_from_slice(&w.to_be_bytes());
    }
    6 + 2 * words.len()
}

/// encode read request
pub fn read_frame(vp: u16, n: u8) -> [u8; 7] {
    assert!(n <= MAX_READ);
    let a = vp.to_be_bytes();
    [0x5a, 0xa5, 0x04, 0x83, a[0], a[1], n]
}

/// pack text field
pub fn text_words(s: &str, out: &mut [u16]) -> usize {
    let bytes = s.as_bytes();
    let mut n = bytes.len().min(out.len() * 2 - 1);
    while !s.is_char_boundary(n) {
        n -= 1;
    }
    let words = n / 2 + 1;
    for (i, w) in out[..words].iter_mut().enumerate() {
        let at = |k: usize| if k < n { bytes[k] } else { 0 };
        *w = u16::from_be_bytes([at(2 * i), at(2 * i + 1)]);
    }
    words
}

/// scale x10 reading
pub fn x10(value: Option<f32>) -> u16 {
    value.map_or(NO_READING, |v| ((v * 10.0).round() as i16) as u16)
}

/// whole-number reading
pub fn whole(value: Option<f32>) -> u16 {
    value.map_or(NO_READING, |v| v.round().clamp(0.0, 32767.0) as u16)
}
"""


def write_team_rust(path):
    head = TEAM_RUST_HEAD.format(version=f"{VERSION >> 8}.{VERSION & 0xFF}", base=BASE,
                                 no_reading=NO_READING, reset=RESET_MAGIC)
    path.write_text(head + "\n".join(rust_consts()) + "\n" + TEAM_RUST_TAIL)


def write_html(path):
    toc, body = [], []
    for s in SECTIONS:
        a, b = span(s)
        toc.append(f'<li><a href="#{s.key}"><code class=addr>{fmt_vp(a, b - a + 1)}</code> {s.title}</a></li>')
        body.append(f'<section id="{s.key}"><h2>{s.title}</h2>')
        if s.note:
            body.append(f"<p class=note>{s.note}</p>")
        rows = [p for p in s.parts if isinstance(p, Var)]
        if rows:
            body.append("<div class=scroll><table><thead><tr><th>VP</th><th>Name</th><th>Type</th>"
                        "<th>Units</th><th>Power-up</th><th>Meaning</th></tr></thead><tbody>")
            for v in rows:
                ro = " <span class=tag>read-only</span>" if v.read_only else ""
                mq = f"<div class=mqtt>MQTT <code>{v.mqtt}</code></div>" if v.mqtt else ""
                kind = type_label(v) + (f"<div class=sub>{v.bytes} bytes</div>" if v.kind == "text" else "")
                body.append(
                    f"<tr><td class=vp><code>{fmt_vp(v.vp, v.words)}</code></td>"
                    f"<td class=name><code>{v.name}</code>{ro}</td><td>{kind}</td><td>{v.unit}</td>"
                    f"<td><code>{html.escape(fmt_default(v))}</code></td>"
                    f"<td>{v.desc}{values_html(v)}{mq}</td></tr>")
            body.append("</tbody></table></div>")
        for p in s.parts:
            if not isinstance(p, Struct):
                continue
            heads = "".join(f"<th>{html.escape(label)}{' <span class=tag>read-only</span>' if ro else ''}</th>"
                            for _, _, label, ro in p.entries)
            body.append(f"<div class=scroll><table class=matrix><thead><tr><th>Field</th><th>Type</th>"
                        f"<th>Meaning</th>{heads}</tr></thead><tbody>")
            for f in p.fields:
                cells = "".join(f"<td class=vp><code>{fmt_vp(vp + f.vp, f.words)}</code></td>" for _, vp, _, _ in p.entries)
                kind = type_label(f) + (f"<div class=sub>{f.bytes} bytes</div>" if f.kind == "text" else "")
                unit = f"<div class=sub>{f.unit}</div>" if f.unit else ""
                body.append(f"<tr><td class=name><code>{f.name}</code></td><td>{kind}{unit}</td>"
                            f"<td>{f.desc}{values_html(f)}</td>{cells}</tr>")
            body.append("</tbody></table></div>")
            first = p.entries[0][0].upper()
            body.append(f"<p class=fine>Every entry has the same layout (stride <code>0x{p.stride:X}</code>). In "
                        f"<code>dwin_vp.rs</code>: <code>{first} + {p.module}::{p.fields[-1].name.upper()}</code>, "
                        f"or <code>{p.array}[k]</code>.</p>")
        body.append("</section>")
    ex = "".join(f"<tr><td>{html.escape(t)}</td><td><code class=bytes>{hexs(b)}</code></td></tr>" for t, b in examples())
    chars = html.escape("A–Z a–z 0–9 ! \" # $ % & ' ( ) * + , - . / : ; < = > ? @ [ \\ ] ^ _ ` { | } ~ ° ₂ µ ³ · … ’ ‘ “ ” – — − ×")
    path.write_text(TEMPLATE.format(
        version=f"{VERSION >> 8}.{VERSION & 0xFF}", toc="".join(toc), body="\n".join(body),
        examples=ex, chars=chars, fit=AI_TEXT_FIT[0], fit_chips=AI_TEXT_FIT[1], style=STYLE))


STYLE = """\
:root {
  --bg: #ffffff; --fg: #17202a; --muted: #5b6878; --line: #dfe4ea; --soft: #f4f6f9;
  --accent: #0f766e; --accent-soft: #e3f4f1; --code: #24303d; --tag: #9a3412; --tag-bg: #fff1e6;
}
@media (prefers-color-scheme: dark) {
  :root:not([data-theme="light"]) {
    --bg: #0b0f14; --fg: #e8eef4; --muted: #93a1b2; --line: #243040; --soft: #121a23;
    --accent: #5eead4; --accent-soft: #10282a; --code: #d6e2ee; --tag: #fdba74; --tag-bg: #2a1a0e;
  }
}
:root[data-theme="dark"] {
  --bg: #0b0f14; --fg: #e8eef4; --muted: #93a1b2; --line: #243040; --soft: #121a23;
  --accent: #5eead4; --accent-soft: #10282a; --code: #d6e2ee; --tag: #fdba74; --tag-bg: #2a1a0e;
}
* { box-sizing: border-box; }
body { margin: 0; background: var(--bg); color: var(--fg);
  font: 15px/1.5 -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif; }
main { max-width: 1180px; margin: 0 auto; padding: 32px 16px 64px; }
h1 { font-size: 28px; margin: 0 0 4px; letter-spacing: -0.01em; }
h2 { font-size: 19px; margin: 40px 0 6px; }
h3 { font-size: 15px; margin: 0 0 6px; }
p { margin: 6px 0 10px; }
a { color: var(--accent); text-decoration: none; }
a:hover { text-decoration: underline; }
code { font: 13px/1.4 ui-monospace, SFMono-Regular, Menlo, Consolas, monospace; color: var(--code); }
.lede { color: var(--muted); max-width: 800px; }
.cards { display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 14px; margin: 20px 0 6px; }
.card { border: 1px solid var(--line); border-radius: 12px; padding: 14px 16px; background: var(--soft); }
.card ul { margin: 6px 0 0; padding-left: 18px; }
.card li { margin: 4px 0; }
.toc { list-style: none; padding: 0 !important; }
.toc code.addr { display: inline-block; min-width: 118px; }
.scroll { overflow-x: auto; border: 1px solid var(--line); border-radius: 10px; margin: 10px 0; }
table { border-collapse: collapse; width: 100%; font-size: 14px; }
th, td { text-align: left; vertical-align: top; padding: 8px 10px; border-bottom: 1px solid var(--line); }
thead th { background: var(--soft); font-size: 12px; text-transform: uppercase; letter-spacing: 0.04em; color: var(--muted); white-space: nowrap; }
tbody tr:last-child td { border-bottom: 0; }
td.vp { white-space: nowrap; }
td.vp code { color: var(--accent); font-weight: 600; }
td.name { white-space: nowrap; }
.sub, .mqtt { color: var(--muted); font-size: 12px; }
.mqtt { margin-top: 4px; }
.note { color: var(--muted); max-width: 900px; }
.fine { color: var(--muted); font-size: 13px; }
.tag { display: inline-block; font-size: 11px; color: var(--tag); background: var(--tag-bg); border-radius: 6px; padding: 0 6px; margin-left: 4px; white-space: nowrap; }
ul.values { list-style: none; padding: 0; margin: 4px 0 0; }
ul.values li { margin: 1px 0; }
ul.values code { display: inline-block; min-width: 56px; }
code.bytes { white-space: normal; word-break: break-word; }
.key { display: inline-block; background: var(--accent-soft); border-radius: 6px; padding: 0 6px; font-weight: 600; }
footer { margin-top: 48px; color: var(--muted); font-size: 13px; }
@media print { .scroll { overflow: visible; } body { font-size: 12px; } h2 { break-after: avoid; } tr { break-inside: avoid; } }
pre { background: var(--soft); border: 1px solid var(--line); border-radius: 10px; padding: 12px 14px; overflow-x: auto; }
pre code { font-size: 12.5px; }
.doc { max-width: 900px; }
.doc h2 { margin-top: 36px; }
.doc img { max-width: 100%; height: auto; border-radius: 10px; border: 1px solid var(--line); margin-top: 8px; }
.doc em { color: var(--muted); font-size: 13px; }
.doc li { margin: 4px 0; }
.doc td code, .doc li code, .doc p code { background: var(--soft); border-radius: 5px; padding: 0 4px; }
"""

TEMPLATE = """<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Display VP Map</title>
<style>
{style}</style>
</head>
<body>
<main>
<h1>Companion display VP map</h1>
<p class=lede>Version {version}. What the node firmware writes to show real data on the IntelliThings
display. Every value is a 16-bit word at a VP address, written with an ordinary DGUS II write
frame, as the firmware architecture plans for the DWIN display. The display turns the values
into the screens and the light bar; anything you don't write keeps its power-up value.</p>

<div class=cards>
 <div class=card>
  <h3>Write frame</h3>
  <p><code>5A A5 &lt;len&gt; 82 &lt;VP hi&gt; &lt;VP lo&gt; &lt;data…&gt;</code></p>
  <ul>
   <li><code>len</code> = 3 + data bytes. Data is whole 16-bit words, high byte first: at most 240 bytes (120 words) per frame. One frame may cover several variables in a row.</li>
   <li>The display answers <code>5A A5 03 82 4F 4B</code> ("OK"). No CRC.</li>
   <li>Read back: <code>5A A5 04 83 &lt;VP&gt; &lt;n&gt;</code> → <code>5A A5 &lt;4+2n&gt; 83 &lt;VP&gt; &lt;n&gt; &lt;words&gt;</code>, at most 125 words per read.</li>
  </ul>
 </div>
 <div class=card>
  <h3>Values</h3>
  <ul>
   <li>Readings: <code>0x8000</code> = no reading. Temperature and humidity are × 10 (<code>224</code> = 22.4 °C); temperature is signed.</li>
   <li><code>u32</code>: two words, high word first.</li>
   <li>Text: UTF-8, two bytes per word (first byte in the high half), ending with a <code>0x00</code> byte. A first word of <code>0x0000</code> = empty.</li>
   <li>Characters drawn: {chars}. Others are skipped.</li>
  </ul>
 </div>
 <div class=card>
  <h3>Where to send it</h3>
  <ul>
   <li><b>Bench, now:</b> a UART to the companion ESP32-C6 that drives the panel, 115200 8N1, 3.3 V: node TX → companion <span class=key>GP2</span>, companion <span class=key>GP3</span> → node RX (for the "OK" replies), and GND to GND. Or from a laptop over the companion's USB with <code>tools/companion.py</code>.</li>
   <li><b>One-chip node, later:</b> the same VPs, in-process: <code>companion_ui::vp::Inputs::write(vp, words, now)</code>.</li>
  </ul>
 </div>
</div>

<div class=cards>
 <div class=card>
  <h3>The minimum for a working demo</h3>
  <ul>
   <li><a href="#sensors">Sensor readings</a> every 0.5–1 s</li>
   <li><a href="#connection"><code>wifi</code>, <code>hub</code>, <code>synced_ago</code></a> when they change</li>
   <li><a href="#clock">The clock</a> once after SNTP</li>
   <li><a href="#message"><code>message_text</code></a> (and <code>message_ts</code>) for each MQTT <code>…/display</code> message. The AI section shows about <b>{fit} characters</b> ({fit_chips} with action chips).</li>
  </ul>
 </div>
 <div class=card>
  <h3>Map at a glance</h3>
  <ul class=toc>{toc}</ul>
 </div>
</div>

{body}

<section id=examples><h2>Example frames</h2>
<div class=scroll><table><thead><tr><th>What</th><th>Bytes</th></tr></thead><tbody>{examples}</tbody></table></div>
</section>

<footer>Generated from <code>dgus/vpmap.py</code>, with <code>vp_map.csv</code>, <code>vp_map.json</code> and the drop-in <code>dwin_vp.rs</code> next to this file. 0x1000–0x17FF is the companion firmware's input memory; the DWIN panel's own VPs are internal and never written by the node.</footer>
</main>
</body>
</html>
"""


def write_doc_html(src, path, title):
    """A Markdown file from handoff/ as a page with the VP map's look."""
    import markdown

    body = markdown.markdown(src.read_text(), extensions=["tables", "fenced_code"])
    body = body.replace("<table>", "<div class=scroll><table>").replace("</table>", "</table></div>")
    path.write_text(
        "<!doctype html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n"
        "<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n"
        f"<title>{title}</title>\n<style>\n{STYLE}</style>\n</head>\n<body>\n"
        f"<main class=doc>\n{body}\n</main>\n</body>\n</html>\n")


def main():
    check()
    out = ROOT / "handoff"
    out.mkdir(exist_ok=True)
    write_html(out / "VP_MAP.html")
    write_csv(out / "vp_map.csv")
    write_json(out / "vp_map.json")
    write_team_rust(out / "dwin_vp.rs")
    write_rust(ROOT / "firmware/companion-ui/src/vp_map.rs")
    if (out / "HANDOFF.md").exists():
        write_doc_html(out / "HANDOFF.md", out / "HANDOFF.html", "Display Handoff")
    print(f"VP map v{VERSION >> 8}.{VERSION & 0xFF}: {len(entries())} variables in "
          f"{BASE:#06x}-{BASE + WORDS - 1:#06x} -> handoff/ and firmware/companion-ui/src/vp_map.rs")


if __name__ == "__main__":
    main()
