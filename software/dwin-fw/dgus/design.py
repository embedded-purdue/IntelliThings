"""shared design system"""

import math
import os
from functools import lru_cache

from PIL import Image, ImageDraw, ImageFont

W, H = 800, 480
S = 2  # supersample factor

# color palette

BG = (11, 15, 20)
SURFACE = (21, 27, 35)
SURFACE_2 = (31, 39, 50)
LINE = (44, 54, 68)
TEXT = (234, 240, 246)
MUTED = (138, 151, 168)
FAINT = (78, 90, 106)
ACCENT = (94, 234, 212)  # accent color
GOOD = (74, 222, 128)
MODERATE = (251, 191, 36)
POOR = (249, 115, 22)
BAD = (239, 68, 68)
NIGHT = (128, 62, 22)
NIGHT_DIM = (74, 38, 16)
BLACK = (0, 0, 0)
AWAY_BG = (4, 6, 9)
AWAY_TEXT = (170, 180, 192)
AWAY_DIM = (90, 100, 112)
QR_PAPER = (244, 247, 250)

# air quality states
STATES = ["good", "moderate", "poor", "bad", "neutral"]
STATE_COLOURS = {"good": GOOD, "moderate": MODERATE, "poor": POOR, "bad": BAD, "neutral": MUTED}


def mix(a, b, t):
    return tuple(round(a[i] + (b[i] - a[i]) * t) for i in range(3))


def tile_fill(state):
    """tile background color"""
    return mix(SURFACE, STATE_COLOURS[state], 0.10) if state in ("moderate", "poor", "bad") else SURFACE


ALERT_BG = mix(BG, BAD, 0.07)
ALERT_CARD = mix(SURFACE, BAD, 0.06)


def rgb666(c):
    return tuple(((v >> 2) << 2) | (v >> 6) for v in c)


def rgb565(c):
    r, g, b = c
    return ((r >> 3) << 11) | ((g >> 2) << 5) | (b >> 3)


_RGB666_LUT = [((v >> 2) << 2) | (v >> 6) for v in range(256)] * 3


def rgb666_image(img):
    """quantize to rgb666"""
    return img.convert("RGB").point(_RGB666_LUT)


# system fonts

FONT_FILE = os.environ.get("DGUS_FONT", "/System/Library/Fonts/Avenir Next.ttc")
_DEFAULT_INDEXES = {"bold": 0, "demi": 2, "medium": 5, "regular": 7, "light": 10}
if "DGUS_FONT_INDEXES" in os.environ:
    WEIGHTS = dict(zip(_DEFAULT_INDEXES, (int(v) for v in os.environ["DGUS_FONT_INDEXES"].split(","))))
elif "DGUS_FONT" in os.environ:
    WEIGHTS = {k: 0 for k in _DEFAULT_INDEXES}
else:
    WEIGHTS = _DEFAULT_INDEXES


@lru_cache(maxsize=None)
def font_px(px, weight="regular"):
    """load pixel font"""
    return ImageFont.truetype(FONT_FILE, px, index=WEIGHTS[weight])


def font(size, weight="regular"):
    return font_px(round(size * S), weight)


def _segments(s, size):
    """split subscript runs"""
    out = []
    for i, part in enumerate(s.split("₂")):
        if i:
            out.append(("2", size * 0.62, size * 0.22))
        if part:
            out.append((part, size, 0))
    return out


def measure(s, size, weight="regular"):
    return sum(font(sz, weight).getlength(t) for t, sz, _ in _segments(s, size)) / S


def wrap(s, size, weight, width):
    lines, line = [], ""
    for word in s.split():
        trial = f"{line} {word}".strip()
        if line and measure(trial, size, weight) > width:
            lines.append(line)
            line = word
        else:
            line = trial
    if line:
        lines.append(line)
    return lines


# drawing canvas


class Frame:
    """supersampled drawing surface"""

    def __init__(self, bg=BG, w=W, h=H):
        self.bg = bg
        self.w, self.h = w, h
        self.img = Image.new("RGB", (w * S, h * S), bg)
        self.d = ImageDraw.Draw(self.img)

    def fill(self, colour):
        self.img.paste(colour, (0, 0, self.w * S, self.h * S))
        self.bg = colour

    def rect(self, x, y, w, h, fill=None, r=0, outline=None, width=1):
        box = [x * S, y * S, (x + w) * S - 1, (y + h) * S - 1]
        self.d.rounded_rectangle(box, radius=r * S, fill=fill, outline=outline, width=round(width * S))

    def circle(self, cx, cy, r, fill=None, outline=None, width=1):
        box = [(cx - r) * S, (cy - r) * S, (cx + r) * S, (cy + r) * S]
        self.d.ellipse(box, fill=fill, outline=outline, width=round(width * S))

    def arc(self, cx, cy, r, start, end, fill, width=1):
        box = [(cx - r) * S, (cy - r) * S, (cx + r) * S, (cy + r) * S]
        self.d.arc(box, start, end, fill=fill, width=round(width * S))

    def line(self, pts, fill, width=1):
        self.d.line([(x * S, y * S) for x, y in pts], fill=fill, width=round(width * S), joint="curve")

    def poly(self, pts, fill=None, outline=None):
        self.d.polygon([(x * S, y * S) for x, y in pts], fill=fill, outline=outline)

    def text(self, x, y, s, size, weight="regular", fill=TEXT, anchor="ls"):
        """draw text run"""
        total = measure(s, size, weight)
        h, v = anchor[0], anchor[1]
        x0 = x - (total / 2 if h == "m" else total if h == "r" else 0)
        for part, sz, drop in _segments(s, size):
            self.d.text((x0 * S, (y + drop) * S), part, font=font(sz, weight), fill=fill, anchor="l" + v)
            x0 += font(sz, weight).getlength(part) / S
        return total

    def done(self):
        return rgb666_image(self.img.resize((self.w, self.h), Image.LANCZOS))


# vector icons


def icon(f, name, cx, cy, size, colour, bg=None):
    s = size / 24
    bg = bg or f.bg
    if name == "temp":
        f.rect(cx - 3 * s, cy - 10 * s, 6 * s, 14 * s, outline=colour, r=3 * s, width=2 * s)
        f.circle(cx, cy + 6 * s, 5 * s, fill=colour)
        f.rect(cx - 1 * s, cy - 4 * s, 2 * s, 8 * s, fill=colour)
    elif name == "humidity":
        f.poly([(cx, cy - 11 * s), (cx - 7 * s, cy + 1 * s), (cx + 7 * s, cy + 1 * s)], fill=colour)
        f.circle(cx, cy + 3 * s, 7.2 * s, fill=colour)
    elif name == "co2":
        f.circle(cx - 5 * s, cy + 2 * s, 5 * s, fill=colour)
        f.circle(cx + 2 * s, cy - 2 * s, 7 * s, fill=colour)
        f.circle(cx + 7 * s, cy + 3 * s, 4.5 * s, fill=colour)
        f.rect(cx - 5 * s, cy + 2 * s, 12 * s, 5.5 * s, fill=colour)
    elif name == "pm":
        for dx, dy, r in [(-6, -5, 2.6), (3, -7, 1.8), (6, 1, 3), (-3, 4, 2), (-8, 6, 1.4), (2, 8, 1.6)]:
            f.circle(cx + dx * s, cy + dy * s, r * s, fill=colour)
    elif name == "tvoc":
        pts = [(cx - 7 * s, cy + 5 * s), (cx, cy - 7 * s), (cx + 7 * s, cy + 5 * s)]
        f.line([pts[0], pts[1], pts[2], pts[0]], fill=colour, width=1.8 * s)
        for px, py in pts:
            f.circle(px, py, 3.2 * s, fill=colour)
    elif name == "light":
        f.circle(cx, cy, 5 * s, fill=colour)
        for k in range(8):
            a = k * math.pi / 4
            f.line([(cx + math.cos(a) * 8 * s, cy + math.sin(a) * 8 * s),
                    (cx + math.cos(a) * 11 * s, cy + math.sin(a) * 11 * s)], fill=colour, width=2 * s)
    elif name == "wifi":
        for r in (4, 8, 12):
            f.arc(cx, cy + 6 * s, r * s, 225, 315, colour, 2 * s)
        f.circle(cx, cy + 6 * s, 1.6 * s, fill=colour)
    elif name == "hub":
        f.poly([(cx, cy - 10 * s), (cx - 10 * s, cy - 1 * s), (cx + 10 * s, cy - 1 * s)], fill=colour)
        f.rect(cx - 7 * s, cy - 2 * s, 14 * s, 11 * s, fill=colour)
    elif name == "person":
        f.circle(cx, cy - 6 * s, 4.2 * s, fill=colour)
        f.d.pieslice([(cx - 8 * s) * S, (cy - 0 * s) * S, (cx + 8 * s) * S, (cy + 16 * s) * S], 180, 360, fill=colour)
    elif name == "fan":
        for k in range(3):
            a = k * 2 * math.pi / 3 - math.pi / 2
            f.circle(cx + math.cos(a) * 5 * s, cy + math.sin(a) * 5 * s, 4.5 * s, fill=colour)
        f.circle(cx, cy, 2.5 * s, fill=bg)
    elif name == "bulb":
        f.circle(cx, cy - 3 * s, 7 * s, fill=colour)
        f.rect(cx - 3.5 * s, cy + 3 * s, 7 * s, 6 * s, fill=colour, r=1 * s)
    elif name == "thermostat":
        f.circle(cx, cy, 9 * s, outline=colour, width=2 * s)
        f.line([(cx, cy), (cx + 4 * s, cy - 5 * s)], fill=colour, width=2 * s)
    elif name == "check":
        f.line([(cx - 7 * s, cy), (cx - 2 * s, cy + 5 * s), (cx + 8 * s, cy - 6 * s)], fill=colour, width=2.6 * s)
    elif name == "warning":
        f.poly([(cx, cy - 11 * s), (cx - 12 * s, cy + 10 * s), (cx + 12 * s, cy + 10 * s)], fill=colour)
        f.rect(cx - 1.4 * s, cy - 3 * s, 2.8 * s, 8 * s, fill=bg, r=1 * s)
        f.circle(cx, cy + 7.2 * s, 1.6 * s, fill=bg)
    elif name == "arrow":
        f.line([(cx - 7 * s, cy), (cx + 6 * s, cy)], fill=colour, width=2 * s)
        f.line([(cx + 1 * s, cy - 5 * s), (cx + 7 * s, cy), (cx + 1 * s, cy + 5 * s)], fill=colour, width=2 * s)
    elif name == "door":
        f.rect(cx - 6 * s, cy - 10 * s, 12 * s, 20 * s, outline=colour, width=2 * s, r=1 * s)
        f.circle(cx + 3 * s, cy, 1.4 * s, fill=colour)
    elif name == "window":
        f.rect(cx - 8 * s, cy - 9 * s, 16 * s, 18 * s, outline=colour, width=2 * s, r=1 * s)
        f.line([(cx, cy - 9 * s), (cx, cy + 9 * s)], fill=colour, width=1.6 * s)
        f.line([(cx - 8 * s, cy), (cx + 8 * s, cy)], fill=colour, width=1.6 * s)
    elif name == "purifier":
        f.rect(cx - 7 * s, cy - 10 * s, 14 * s, 20 * s, outline=colour, width=2 * s, r=3 * s)
        for k in range(3):
            f.line([(cx - 3.5 * s, cy - 4 * s + k * 4 * s), (cx + 3.5 * s, cy - 4 * s + k * 4 * s)], fill=colour, width=1.5 * s)
    elif name == "humidifier":
        f.poly([(cx, cy - 10 * s), (cx - 6 * s, cy), (cx + 6 * s, cy)], fill=colour)
        f.circle(cx, cy + 2 * s, 6.2 * s, fill=colour)
        f.circle(cx + 7 * s, cy - 7 * s, 2 * s, fill=colour)
    elif name == "lock":
        f.rect(cx - 7 * s, cy - 2 * s, 14 * s, 11 * s, fill=colour, r=2 * s)
        f.arc(cx, cy - 3 * s, 4.5 * s, 180, 360, colour, 2 * s)
        f.line([(cx - 4.5 * s, cy - 3 * s), (cx - 4.5 * s, cy - 1 * s)], fill=colour, width=2 * s)
        f.line([(cx + 4.5 * s, cy - 3 * s), (cx + 4.5 * s, cy - 1 * s)], fill=colour, width=2 * s)
    elif name == "dot":
        f.circle(cx, cy, 4 * s, fill=colour)
    else:
        raise KeyError(name)


def spinner(f, cx, cy, r, phase, colour, width=2.5):
    """spinner rotation phase"""
    a = phase * 360
    f.arc(cx, cy, r, 0, 360, mix(f.bg, colour, 0.18), width)
    f.arc(cx, cy, r, a, a + 100, colour, width)


def orb(f, cx, cy, r, phase, mode="idle"):
    """orb animation loop"""
    colour = {"idle": ACCENT, "thinking": ACCENT, "alert": BAD, "offline": FAINT}[mode]
    breathe = 1
    if mode == "idle":
        breathe = 1 + 0.07 * math.sin(phase * 2 * math.pi)
    elif mode == "alert":
        breathe = 1 + 0.12 * abs(math.sin(phase * math.pi))
    for i in range(8, 0, -1):
        f.circle(cx, cy, r * breathe * (1 + i * 0.09), fill=mix(f.bg, colour, 0.035 * (9 - i)))
    f.circle(cx, cy, r * breathe, fill=mix(f.bg, colour, 0.85 if mode != "offline" else 0.5))
    f.circle(cx - r * 0.28, cy - r * 0.3, r * 0.38 * breathe, fill=mix(colour, (255, 255, 255), 0.45))
    if mode == "thinking":
        for k in range(3):
            a = phase * 2 * math.pi + k * 2 * math.pi / 3
            f.circle(cx + math.cos(a) * r * 1.55, cy + math.sin(a) * r * 1.55, r * 0.17, fill=colour)
