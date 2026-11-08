"""static page backgrounds"""

import qrcode  # preview qr code

from . import design as d
from .design import Frame, icon

# page layout geometry

TILE_W, TILE_H = 176, 124


def tile_origin(i):
    """home tile position"""
    return 412 + (i % 2) * 188, 60 + (i // 2) * 136


CARD = (24, 232, 364, 224)  # agent card bounds

# home tile descriptors
METRICS = [
    ("temp", "temp", "Temperature"),
    ("humidity", "humidity", "Humidity"),
    ("co2", "co2", "CO₂"),
    ("pm", "pm", "PM2.5"),
    ("voc", "tvoc", "VOC index"),
    ("light", "light", "Light"),
]

SENSOR_ROW_Y0, SENSOR_ROW_DY, SENSOR_ROW_H = 60, 67, 59


def sensor_row_y(i):
    return SENSOR_ROW_Y0 + i * SENSOR_ROW_DY


# trend row bounds
TREND_X0, TREND_X1 = 360, 612


def trend_box(i):
    y = sensor_row_y(i)
    return TREND_X0, y + 12, TREND_X1, y + 47  # trend line box


# room tile layout
ROOMS = [
    ("Living Room", 24, 60, 400, 232, True),
    ("Kitchen", 432, 60, 344, 150, True),
    ("Bedroom", 432, 218, 208, 204, True),
    ("Office", 648, 218, 128, 204, False),
    ("Entry", 24, 300, 400, 122, False),
]

BOOT_STEPS = ["Display", "Sensors", "Wi-Fi", "MQTT broker", "Home Assistant"]


def boot_row_y(i):
    return 288 + i * 30


SETUP_QR = (484, 104, 248)  # setup qr area


# background renderers


def tile_skin(f, x, y, metric, state):
    """render tile skin"""
    key, ic, label = metric
    colour = d.STATE_COLOURS[state]
    alert = state in ("moderate", "poor", "bad")
    fill = d.tile_fill(state)
    f.rect(x, y, TILE_W, TILE_H, fill=fill, r=14, outline=d.mix(d.SURFACE, colour, 0.55) if alert else None, width=1.5)
    icon(f, ic, x + 26, y + 26, 18, colour if alert else d.MUTED, bg=fill)
    f.text(x + 44, y + 32, label, 15, "medium", d.MUTED)


def bg_home():
    f = Frame(d.BG)
    for i, m in enumerate(METRICS):
        tile_skin(f, *tile_origin(i), m, "good")
    x, y, w, h = CARD
    f.rect(x, y, w, h, fill=d.SURFACE, r=18)
    return f.done()


def bg_sensors():
    f = Frame(d.BG)
    f.text(776, 33, "Last 12 hours", 14, "medium", d.MUTED, "rs")
    for i, (key, ic, label) in enumerate(METRICS):
        y = sensor_row_y(i)
        f.rect(24, y, 752, SENSOR_ROW_H, fill=d.SURFACE, r=12)
        icon(f, ic, 52, y + 30, 18, d.MUTED, bg=d.SURFACE)
        f.text(76, y + 36, label, 16, "medium", d.MUTED)
        x0, top, x1, bot = trend_box(i)
        f.line([(x0, bot + 2), (x1, bot + 2)], fill=d.LINE, width=1)
    return f.done()


def room_skin(f, room, state, here=False, people=None):
    name, x, y, w, h, node = room
    if not node:
        f.rect(x, y, w, h, fill=d.mix(d.BG, d.SURFACE, 0.5), r=14, outline=d.LINE, width=1)
        f.text(x + 18, y + 32, name, 17, "demi", d.MUTED)
        f.text(x + 18, y + 56, "No sensor node", 14, "medium", d.FAINT)
        return
    colour = d.STATE_COLOURS[state]
    alert = state in ("moderate", "poor", "bad")
    fill = d.mix(d.SURFACE, colour, 0.08) if alert else d.SURFACE
    outline = d.mix(d.SURFACE, d.ACCENT, 0.7) if here else d.mix(d.SURFACE, colour, 0.5) if alert else None
    f.rect(x, y, w, h, fill=fill, r=14, outline=outline, width=2 if here else 1.5)
    f.text(x + 18, y + 32, name, 17, "demi")


def room_fill(state):
    colour = d.STATE_COLOURS[state]
    return d.mix(d.SURFACE, colour, 0.08) if state in ("moderate", "poor", "bad") else d.SURFACE


def bg_rooms():
    f = Frame(d.BG)
    for room in ROOMS:
        room_skin(f, room, "good")
    y, x = 452, 24
    for colour, word in [(d.GOOD, "Good"), (d.MODERATE, "Moderate"), (d.POOR, "Poor")]:
        f.circle(x + 5, y - 5, 4.5, fill=colour)
        x += 16 + f.text(x + 16, y, word, 14, "medium", d.MUTED) + 22
    icon(f, "person", x + 8, y - 7, 14, d.ACCENT)
    x += 22 + f.text(x + 22, y, "Someone's there", 14, "medium", d.MUTED) + 22
    f.rect(x, y - 14, 18, 16, fill=d.mix(d.BG, d.SURFACE, 0.5), outline=d.LINE, r=4)
    f.text(x + 26, y, "Virtual devices only", 14, "medium", d.MUTED)
    return f.done()


ALERT_CARD_BOX = (24, 60, 752, 396)


def bg_alert():
    f = Frame(d.ALERT_BG)
    x, y, w, h = ALERT_CARD_BOX
    # static card border
    f.rect(x, y, w, h, fill=d.ALERT_CARD, r=20, outline=d.mix(d.ALERT_CARD, d.BAD, 0.7), width=2.5)
    icon(f, "warning", 64, 104, 34, d.BAD, bg=d.ALERT_CARD)
    f.line([(56, 336), (370, 336)], fill=d.mix(d.ALERT_CARD, d.BAD, 0.25), width=1)
    f.text(420, 196, "I've already", 15, "demi", d.MUTED)
    f.text(420, 324, "Could you", 15, "demi", d.MUTED)
    return f.done()


def bg_night():
    return Frame(d.BLACK).done()


def bg_away():
    return Frame(d.AWAY_BG).done()


def bg_boot():
    f = Frame(d.BG)
    f.text(d.W / 2, 236, "IntelliThings", 36, "demi", d.TEXT, "ms")
    return f.done()


def bg_setup():
    f = Frame(d.BG)
    f.text(40, 74, "Let's get this set up", 32, "demi")
    f.text(40, 104, "Connect from your phone or laptop.", 18, "medium", d.MUTED)
    for i, label in enumerate(["Join Wi-Fi", "Password", "Then open"]):
        y = 160 + i * 84
        f.circle(58, y + 22, 18, fill=d.mix(d.SURFACE, d.ACCENT, 0.2))
        f.text(58, y + 29, str(i + 1), 18, "demi", d.ACCENT, "ms")
        f.text(92, y + 14, label, 16, "medium", d.MUTED)
    x0, y0, size = SETUP_QR
    f.rect(x0 - 20, y0 - 20, size + 40, size + 40, fill=d.QR_PAPER, r=20)
    f.text(x0 + size / 2, y0 + size + 58, "Or scan to join", 16, "medium", d.MUTED, "ms")
    return f.done()


def bg_test():
    f = Frame(d.BLACK)
    bars = [(255, 255, 255), (255, 255, 0), (0, 255, 255), (0, 255, 0), (255, 0, 255), (255, 0, 0), (0, 0, 255), (0, 0, 0)]
    for i, c in enumerate(bars):
        f.rect(i * 100, 0, 100, 180, fill=c)
    for j, c in enumerate([(255, 0, 0), (0, 255, 0), (0, 0, 255), (255, 255, 255)]):
        for k in range(64):
            f.rect(k * 12.5, 180 + j * 40, 12.6, 40, fill=tuple(round(v * k / 63) for v in c))
    for gx in range(0, d.W + 1, 40):
        f.line([(gx, 340), (gx, d.H)], fill=d.LINE, width=1)
    for gy in range(340, d.H + 1, 40):
        f.line([(0, gy), (d.W, gy)], fill=d.LINE, width=1)
    f.rect(0, 0, d.W, d.H, outline=(255, 255, 255), width=1)
    f.text(d.W / 2, 418, "800 × 480 · RGB666 · DMG80480T050_09WN", 18, "demi", d.TEXT, "ms")
    return f.done()
