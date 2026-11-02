"""non-text sprite catalog"""

from dataclasses import dataclass

from PIL import Image

from . import design as d
from . import pages as p
from .design import Frame, icon


@dataclass
class Sprite:
    name: str
    image: Image.Image


def _canvas(w, h, bg):
    return Frame(bg, w, h)


def icon_sprite(name, kind, size, colour, bg, box=None):
    box = box or round(size * 1.25)
    f = _canvas(box, box, bg)
    icon(f, kind, box / 2, box / 2, size, colour, bg=bg)
    return Sprite(name, f.done())


def dot_sprite(name, r, colour, bg):
    box = round(r * 2 + 2)
    f = _canvas(box, box, bg)
    f.circle(box / 2, box / 2, r, fill=colour)
    return Sprite(name, f.done())


ORB_FRAMES = 16
SPIN_FRAMES = 12


def orb_sprites(prefix, r, bg):
    """orb animation frames"""
    box = round(r * 1.12 * (1 + 8 * 0.09) * 2 + 4)
    out = []
    for mode in ("idle", "thinking", "alert", "offline"):
        frames = ORB_FRAMES if mode != "offline" else 1
        for k in range(frames):
            f = _canvas(box, box, bg)
            d.orb(f, box / 2, box / 2, r, k / frames, mode)
            out.append(Sprite(f"{prefix}_{mode}_{k}", f.done()))
    return out


def spinner_sprites(prefix, r, colour, bg):
    box = round(r * 2 + 6)
    out = []
    for k in range(SPIN_FRAMES):
        f = _canvas(box, box, bg)
        d.spinner(f, box / 2, box / 2, r, k / SPIN_FRAMES, colour, 2)
        out.append(Sprite(f"{prefix}_{k}", f.done()))
    return out


CHIP_H = 34


CHIP_CAP = 18
CHIP_SLICE = 16


def chip_parts(prefix, colour, bg, fill=None):
    """chip slice parts"""
    fill = fill or d.mix(bg, colour, 0.12)
    edge = d.mix(bg, colour, 0.45)
    wide = 2 * CHIP_CAP + 4 * CHIP_SLICE
    f = _canvas(wide, CHIP_H, bg)
    f.rect(0, 0, wide, CHIP_H, fill=fill, r=17, outline=edge, width=1.2)
    img = f.done()
    mid = CHIP_CAP + CHIP_SLICE
    return [
        Sprite(f"{prefix}_l", img.crop((0, 0, CHIP_CAP, CHIP_H))),
        Sprite(f"{prefix}_m", img.crop((mid, 0, mid + CHIP_SLICE, CHIP_H))),
        Sprite(f"{prefix}_r", img.crop((wide - CHIP_CAP, 0, wide, CHIP_H))),
    ]


def chip_fill(colour, bg):
    return d.mix(bg, colour, 0.12)


def tile_skins():
    """home tile skins"""
    out = []
    for i, m in enumerate(p.METRICS):
        for state in ("moderate", "poor", "bad"):
            f = _canvas(p.TILE_W, p.TILE_H, d.BG)
            p.tile_skin(f, 0, 0, m, state)
            out.append(Sprite(f"tile_{m[0]}_{state}", f.done()))
    return out


def sensor_icons():
    """sensor row icons"""
    out = []
    for key, ic, _ in p.METRICS:
        for state in ("moderate", "poor", "bad"):
            out.append(icon_sprite(f"row_{key}_{state}", ic, 18, d.STATE_COLOURS[state], d.SURFACE, box=24))
    return out


def room_skins():
    out = []
    for i, room in enumerate(p.ROOMS):
        name, x, y, w, h, node = room
        if not node:
            continue
        for state in ("good", "moderate", "poor", "bad"):
            for here in (False, True):
                if state == "good" and not here:
                    continue  # skip background state
                f = _canvas(w, h, d.BG)
                p.room_skin(f, (name, 0, 0, w, h, node), state, here=here)
                if here:
                    # local display badge
                    tw = d.measure(name, 17, "demi")
                    f.rect(28 + tw, 16, 92, 22, fill=d.mix(p.room_fill(state), d.ACCENT, 0.18), r=11)
                    f.text(28 + tw + 46, 32, "This display", 12, "demi", d.ACCENT, "ms")
                out.append(Sprite(f"room{i}_{state}{'_here' if here else ''}", f.done()))
    return out


PULSE_FRAMES = 8


def trend_pulses():
    """trend line head"""
    out = []
    for state in d.STATES:
        colour = d.STATE_COLOURS[state]
        for k in range(PULSE_FRAMES):
            import math

            pulse = 0.5 + 0.5 * math.sin(k / PULSE_FRAMES * 2 * math.pi)
            f = _canvas(18, 18, d.SURFACE)
            f.circle(9, 9, 4 + 3 * pulse, fill=d.mix(d.SURFACE, colour, 0.3))
            f.circle(9, 9, 3.5, fill=colour)
            out.append(Sprite(f"pulse_{state}_{k}", f.done()))
    return out


def room_rings():
    """presence pulse halo"""
    out = []
    for state in ("good", "moderate", "poor", "bad"):
        fill = p.room_fill(state)
        for k in range(PULSE_FRAMES):
            ring = k / PULSE_FRAMES
            f = _canvas(52, 52, fill)
            f.circle(26, 26, 10 + ring * 14, outline=d.mix(fill, d.ACCENT, 0.6 * (1 - ring)), width=1.5)
            icon(f, "person", 26, 26, 18, d.ACCENT, bg=fill)
            out.append(Sprite(f"room_ring_{state}_{k}", f.done()))
    return out


def toast_card():
    # transparency color key
    f = _canvas(520, 76, d.BLACK)
    f.rect(0, 0, 520, 76, fill=d.SURFACE_2, r=18, outline=d.mix(d.SURFACE_2, d.ACCENT, 0.5), width=1.5)
    f.circle(44, 38, 22, fill=d.mix(d.SURFACE_2, d.ACCENT, 0.2))
    return Sprite("toast_card", f.done())


def alert_outline_frames(n=6):
    """alert card border"""
    x, y, w, h = p.ALERT_CARD_BOX
    out = []
    for k in range(n):
        pulse = k / (n - 1)
        f = Frame(d.ALERT_BG)
        f.rect(x, y, w, h, fill=d.ALERT_CARD, r=20,
               outline=d.mix(d.ALERT_CARD, d.BAD, 0.35 + 0.5 * pulse), width=2 + pulse)
        img = f.done()
        band = 22  # corner radius margin
        out += [
            Sprite(f"alert_edge_t_{k}", img.crop((x, y, x + w, y + band))),
            Sprite(f"alert_edge_b_{k}", img.crop((x, y + h - band, x + w, y + h))),
            Sprite(f"alert_edge_l_{k}", img.crop((x, y + band, x + 4, y + h - band))),
            Sprite(f"alert_edge_r_{k}", img.crop((x + w - 4, y + band, x + w, y + h - band))),
        ]
    return out


STATES5 = ["good", "moderate", "poor", "bad", "neutral"]
ALERTS = ["moderate", "poor", "bad"]
KEYS = ["temp", "humidity", "co2", "pm", "voc", "light"]
CHIP_ICONS = ["fan", "bulb", "thermostat", "purifier", "humidifier", "window", "door", "lock"]

# contiguous sprite families
FAMILIES = {
    "tile": [f"tile_{k}_{s}" for k in KEYS for s in ALERTS],
    "state_dot": [f"state_dot_{s}" for s in STATES5],
    "row": [f"row_{k}_{s}" for k in KEYS for s in ALERTS],
    "row_dot": [f"row_dot_{s}" for s in STATES5],
    "pulse": [f"pulse_{s}_{k}" for s in STATES5 for k in range(8)],
    "room": [f"room{i}_{x}" for i in range(3)
             for x in ["good_here", "moderate", "moderate_here", "poor", "poor_here", "bad", "bad_here"]],
    "room_dot": [f"room_dot_{s}" for s in STATES5],
    "room_ring": [f"room_ring_{s}_{k}" for s in STATES5[:4] for k in range(8)],
    "room_faint": [f"room_person_faint_{s}" for s in STATES5[:4]],
    "orb": [f"orb_{m}_{k}" for m in ["idle", "thinking", "alert"] for k in range(16)] + ["orb_offline_0"],
    "alert_orb": [f"alert_orb_{m}_{k}" for m in ["idle", "thinking", "alert"] for k in range(16)] + ["alert_orb_offline_0"],
    "boot_orb": [f"boot_orb_{m}_{k}" for m in ["idle", "thinking", "alert"] for k in range(16)] + ["boot_orb_offline_0"],
    "chips": [f"chip_{w}_{t}_{x}" for w in ["card", "alert", "toast"] for t in ["act", "ask"]
              for x in ["l", "m", "r"] + CHIP_ICONS + ["arrow", "check"]],
    "toast_icons": [f"toast_{x}" for x in CHIP_ICONS],
    "alert_edge": [f"alert_edge_{s}_{k}" for k in range(6) for s in "tblr"],
    "boot_spin": [f"boot_spin_{k}" for k in range(12)],
}


def check_families(names):
    index = {n: i for i, n in enumerate(names)}
    for fam, members in FAMILIES.items():
        ids = [index[m] for m in members]
        assert ids == list(range(ids[0], ids[0] + len(ids))), f"sprite family {fam} is not contiguous"


def catalog():
    """collect sprite catalog"""
    s = []
    # top bar
    s.append(icon_sprite("person", "person", 16, d.ACCENT, d.BG))
    s.append(icon_sprite("person_faint", "person", 16, d.FAINT, d.BG))
    s.append(icon_sprite("wifi", "wifi", 16, d.MUTED, d.BG))
    s.append(icon_sprite("wifi_off", "wifi", 16, d.MODERATE, d.BG))
    s.append(icon_sprite("hub", "hub", 16, d.MUTED, d.BG))
    s.append(icon_sprite("hub_off", "hub", 16, d.MODERATE, d.BG))
    s.append(dot_sprite("hub_off_dot", 3.5, d.BAD, d.BG))
    # alert top bar
    s.append(icon_sprite("alert_person", "person", 16, d.ACCENT, d.ALERT_BG))
    s.append(icon_sprite("alert_wifi", "wifi", 16, d.MUTED, d.ALERT_BG))
    s.append(icon_sprite("alert_hub", "hub", 16, d.MUTED, d.ALERT_BG))
    # home tiles
    s += tile_skins()
    for state in d.STATES:
        s.append(dot_sprite(f"state_dot_{state}", 4.5, d.STATE_COLOURS[state], d.tile_fill(state)))
    # sensors page
    s += sensor_icons()
    for state in d.STATES:
        s.append(dot_sprite(f"row_dot_{state}", 4.5, d.STATE_COLOURS[state], d.SURFACE))
    s += trend_pulses()
    # rooms page
    s += room_skins()
    for state in d.STATES:
        s.append(dot_sprite(f"room_dot_{state}", 4.5, d.STATE_COLOURS[state], p.room_fill(state)))
    s += room_rings()
    for state in ("good", "moderate", "poor", "bad"):
        s.append(icon_sprite(f"room_person_faint_{state}", "person", 18, d.FAINT, p.room_fill(state), box=24))
    s.append(icon_sprite("room_bulb", "bulb", 16, d.MODERATE, d.mix(d.BG, d.SURFACE, 0.5)))
    # agent orbs
    s += orb_sprites("orb", 11, d.SURFACE)
    s += orb_sprites("alert_orb", 10, d.ALERT_CARD)
    s += orb_sprites("boot_orb", 34, d.BG)
    s.append(Sprite("caret", Image.new("RGB", (3, 25), d.rgb666(d.ACCENT))))
    # action chips
    chip_icons = ["fan", "bulb", "thermostat", "purifier", "humidifier", "window", "door", "lock"]
    for where, bg in (("card", d.SURFACE), ("alert", d.ALERT_CARD), ("toast", d.SURFACE_2)):
        for tone, colour in (("act", d.ACCENT), ("ask", d.MODERATE)):
            # shared chip fill
            fill = chip_fill(colour, d.SURFACE if where in ("card", "alert") else bg)
            s += chip_parts(f"chip_{where}_{tone}", colour, bg, fill)
            for ic in chip_icons:
                s.append(icon_sprite(f"chip_{where}_{tone}_{ic}", ic, 16, colour, fill, box=20))
            s.append(icon_sprite(f"chip_{where}_{tone}_arrow", "arrow", 14, d.MUTED, fill, box=18))
            s.append(icon_sprite(f"chip_{where}_{tone}_check", "check", 13, d.GOOD, fill, box=18))
    # toast sprite
    s.append(toast_card())
    for ic in chip_icons:
        s.append(icon_sprite(f"toast_{ic}", ic, 22, d.ACCENT, d.mix(d.SURFACE_2, d.ACCENT, 0.2), box=28))
    # alert page
    s += alert_outline_frames()
    # boot page
    s += spinner_sprites("boot_spin", 8, d.ACCENT, d.BG)
    s.append(icon_sprite("boot_check", "check", 15, d.GOOD, d.BG, box=20))
    s.append(dot_sprite("boot_pending", 4, d.LINE, d.BG))
    # test pattern
    s.append(Sprite("test_marker", Image.new("RGB", (20, 20), d.rgb666(d.ACCENT))))
    names = [x.name for x in s]
    assert len(names) == len(set(names)), "duplicate sprite names"
    check_families(names)
    return s
