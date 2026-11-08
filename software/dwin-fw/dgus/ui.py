"""ui shared contract"""

from dataclasses import dataclass

from . import design as d
from . import glyphs as g
from . import pages as p

BG_ICL = 32  # background icl id
MAX_ICONS_PER_ICL = 4000  # max icl icons

# page order list

PAGES = [
    ("BOOT", p.bg_boot),
    ("HOME", p.bg_home),
    ("SENSORS", p.bg_sensors),
    ("ROOMS", p.bg_rooms),
    ("ALERT", p.bg_alert),
    ("NIGHT", p.bg_night),
    ("AWAY", p.bg_away),
    ("SETUP", p.bg_setup),
    ("TEST", p.bg_test),
]
PAGE_ID = {name: i for i, (name, _) in enumerate(PAGES)}

# sprite layer layout

LAYERS = [
    # layer control slots
    ("SKIN", 1, 64, 24),  # background skin layer
    ("MAIN", 2, 200, 120),  # main glyph layer
    ("OVER", 1, 100, 60),  # overlay toast layer
]
VP_START = 0x5000
TREND_ROWS = 6
TREND_POINTS = 72  # trend history points
TREND_STRIDE = 0x100  # trend line stride
QR_UNIT = 5  # qr pixel scale
QR_WORDS = 0x100
# signature marker word
SIGNATURE_VALUE = 0xC0DE


def layer_words(capacity):
    return 2 + 3 * capacity + 1


# text styles

import re
from pathlib import Path

_AIR_RS = Path(__file__).resolve().parent.parent / "firmware/companion-ui/src/air.rs"


def _words_in_air_rs():
    """air state words"""
    return re.findall(r'j\(State::\w+, "([^"]+)"\)', _AIR_RS.read_text())


AIR_WORDS = _words_in_air_rs()
BOOT_WORDS = list(p.BOOT_STEPS)
WORDS = "".join(sorted(set(" ".join(AIR_WORDS + BOOT_WORDS))))  # unique style letters
LETTERS = WORDS
LABEL = "".join(sorted(set(
    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789 ·:…-'&,./%°()+₂")))
UNITS = " °FC%pmµg/³lxinde"
AMPM = "AMP"


def T(name, size, weight, fg, bg, chars=g.TEXT_CHARS):
    return g.TextStyle(name, size, weight, fg, bg, chars)


def _state_styles():
    out = []
    for state in d.STATES:
        fill = d.tile_fill(state)
        room = state != "neutral"  # exclude neutral state
        colour = d.STATE_COLOURS[state]
        alert = state in ("moderate", "poor", "bad")
        out += [
            T(f"TILE_VALUE_{state}", 40, "demi", d.TEXT, fill, g.NUMERIC),
            T(f"TILE_UNIT_{state}", 16, "medium", d.MUTED, fill, UNITS),
            T(f"TILE_WORD_{state}", 15, "demi" if alert else "medium", colour, fill, LETTERS),
            T(f"ROW_WORD_{state}", 15, "demi" if state != "good" else "medium", colour, d.SURFACE, LETTERS),
        ]
        rfill = p.room_fill(state)
        out += [] if not room else [
            T(f"ROOM_BIG_{state}", 64, "medium", d.TEXT, rfill, g.NUMERIC + "°"),
            T(f"ROOM_SMALL_{state}", 44, "medium", d.TEXT, rfill, g.NUMERIC + "°"),
            T(f"ROOM_HUM_{state}", 18, "medium", d.MUTED, rfill, g.NUMERIC),
            T(f"ROOM_WORD_{state}", 15, "demi" if alert else "medium", colour, rfill, LETTERS),
        ]
    return out


def _chip_styles():
    out = []
    from .sprites import chip_fill

    # shared chip styles
    for where, bg in (("CARD", d.SURFACE),):
        for tone, colour in (("ACT", d.ACCENT), ("ASK", d.MODERATE)):
            fill = chip_fill(colour, bg)
            out.append(T(f"CHIP_{where}_{tone}", 15, "demi", d.TEXT, fill, LABEL))
            out.append(T(f"CHIP_{where}_{tone}_HI", 15, "demi", colour, fill, LABEL))
    return out


STYLES = [
    # top bar styles
    T("TITLE", 18, "demi", d.TEXT, d.BG, LABEL),
    T("TOPBAR", 14, "medium", d.MUTED, d.BG, LABEL),
    T("TOPBAR_WARN", 14, "medium", d.MODERATE, d.BG, LABEL),
    T("ALERT_TITLE_BAR", 18, "demi", d.TEXT, d.ALERT_BG, LABEL),
    T("ALERT_TOPBAR", 14, "medium", d.MUTED, d.ALERT_BG, LABEL),
    # home page styles
    T("CLOCK", 104, "medium", d.TEXT, d.BG, g.NUMERIC),
    T("AMPM", 104 * 0.27, "demi", d.MUTED, d.BG, AMPM),
    T("DATE", 20, "medium", d.MUTED, d.BG, LABEL),
    T("CARD_HEADER", 15, "demi", d.ACCENT, d.SURFACE, LABEL),
    T("CARD_HEADER_WARN", 15, "demi", d.MODERATE, d.SURFACE, LABEL),
    T("CARD_AGE", 14, "medium", d.FAINT, d.SURFACE, LABEL),
    T("STATEMENT", 21, "medium", d.TEXT, d.SURFACE),
    T("TOAST_TITLE", 20, "demi", d.TEXT, d.SURFACE_2),
    T("TOAST_SUB", 15, "medium", d.MUTED, d.SURFACE_2),
    # sensor styles
    T("ROW_VALUE", 26, "demi", d.TEXT, d.SURFACE, g.NUMERIC),
    T("ROW_UNIT", 14, "medium", d.MUTED, d.SURFACE, UNITS),
    # room styles
    T("ROOM_NOTE", 14, "medium", d.MUTED, d.mix(d.BG, d.SURFACE, 0.5)),
    # alert styles
    T("ALERT_HEAD", 28, "demi", d.TEXT, d.ALERT_CARD),
    T("ALERT_SUB", 16, "medium", d.MUTED, d.ALERT_CARD),
    T("ALERT_VALUE", 112, "medium", d.BAD, d.ALERT_CARD, g.NUMERIC),
    T("ALERT_UNIT", 22, "medium", d.MUTED, d.ALERT_CARD, UNITS),
    T("ALERT_METRIC", 22, "demi", d.TEXT, d.ALERT_CARD, LABEL),
    T("ALERT_MSG", 18, "medium", d.TEXT, d.ALERT_CARD),
    # night away styles
    T("NIGHT_CLOCK", 150, "light", d.NIGHT, d.BLACK, g.NUMERIC),
    T("NIGHT_LINE", 20, "medium", d.NIGHT_DIM, d.BLACK),
    T("AWAY_CLOCK", 84, "light", d.AWAY_TEXT, d.AWAY_BG, g.NUMERIC),
    T("AWAY_AMPM", 22, "demi", d.AWAY_DIM, d.AWAY_BG, AMPM),
    T("AWAY_LINE", 18, "medium", d.AWAY_DIM, d.AWAY_BG),
    # boot setup styles
    T("BOOT_STEP", 17, "demi", d.TEXT, d.BG, LETTERS),
    T("BOOT_STEP_DIM", 17, "demi", d.FAINT, d.BG, LETTERS),
    T("BOOT_DETAIL", 15, "medium", d.MUTED, d.BG),
    T("BOOT_VERSION", 13, "medium", d.FAINT, d.BG, LABEL),
    T("SETUP_VALUE", 28, "demi", d.TEXT, d.BG),
    T("SETUP_WAIT", 16, "medium", d.FAINT, d.BG, LABEL),
] + _state_styles() + _chip_styles()

assert len({s.name for s in STYLES}) == len(STYLES)
