"""render demo video"""

import shutil
import struct
import subprocess
import tempfile
from pathlib import Path

from PIL import Image, ImageChops, ImageDraw, ImageFilter, ImageFont

from .sim import BUILD, Panel

FPS = 10
W, H = 1040, 760
SHELL = (38, 43, 50)
SHELL_EDGE = (62, 69, 79)
BG = (12, 16, 21)
BAR_BG = (7, 9, 11)
LED_OFF = (21, 25, 30)
LABEL = (232, 237, 243)
SUB = (139, 151, 166)
FONT = "/System/Library/Fonts/Avenir Next.ttc"


def read_frame(path, panel):
    data = path.read_bytes()
    page = struct.unpack_from("<H", data, 0)[0]
    n = data[2]
    name = data[3:3 + n].decode()
    at = 3 + n
    leds = [tuple(data[at + 3 * i: at + 3 * i + 3]) for i in range(24)]
    at += 72
    words = list(struct.unpack_from(f"<{(len(data) - at) // 2}H", data, at))
    mem = list(panel.init) + [0] * (0x10000 - len(panel.init))
    base = panel.manifest["vp_start"]
    mem[base:base + len(words)] = words
    return name, page, leds, panel.render(page, mem)


def draw_bar(img, x, y, w, h, leds):
    """render led bar"""
    d = ImageDraw.Draw(img)
    d.rounded_rectangle([x, y, x + w - 1, y + h - 1], radius=h // 2, fill=BAR_BG)
    pad, gap, n = h * 0.45, 4, len(leds)
    led_w = (w - 2 * pad - gap * (n - 1)) / n
    led_h = max(5, h * 0.42)
    ly = y + (h - led_h) / 2
    glow = Image.new("RGB", (w + 60, h + 60), (0, 0, 0))
    gd = ImageDraw.Draw(glow)
    for i, (r, g, b) in enumerate(leds):
        a = max(r, g, b) / 255
        lx = x + pad + i * (led_w + gap)
        k = min(1.0, a * 1.6)
        # diffuse led glow
        hue = (r, g, b) if a == 0 else tuple(min(255, round(c / a)) for c in (r, g, b))
        colour = tuple(round(LED_OFF[c] + (hue[c] - LED_OFF[c]) * k) for c in range(3))
        d.rounded_rectangle([lx, ly, lx + led_w, ly + led_h], radius=3, fill=colour)
        if a > 0.01:
            gx, gy = lx - x + 30, ly - y + 30
            gd.rounded_rectangle([gx - 4, gy - 4, gx + led_w + 4, gy + led_h + 4], radius=5,
                                 fill=tuple(round(c * 0.8) for c in hue))
    glow = glow.filter(ImageFilter.GaussianBlur(8))
    box = (x - 30, y - 30, x + w + 30, y + h + 30)
    img.paste(ImageChops.add(img.crop(box), Image.eval(glow, lambda v: round(v * 0.9))), box)


def compose(name, leds, screen, fonts, scene_idx, scene_count):
    img = Image.new("RGB", (W, H), BG)
    d = ImageDraw.Draw(img)
    title, sub = fonts
    d.text((40, 52), "IntelliThings companion · demo", font=title, fill=LABEL, anchor="ls")
    d.text((W - 40, 52), f"{scene_idx + 1} / {scene_count}  ·  {name}", font=sub, fill=SUB, anchor="rs")
    # render device frame
    sx, sy = (W - 800) // 2 - 22, 84
    d.rounded_rectangle([sx, sy, sx + 800 + 44, sy + 480 + 44 + 70], radius=26, fill=SHELL, outline=SHELL_EDGE)
    d.rounded_rectangle([sx + 14, sy + 14, sx + 14 + 816, sy + 14 + 496], radius=10, fill=(5, 6, 8))
    img.paste(screen, (sx + 22, sy + 22))
    draw_bar(img, sx + 30, sy + 480 + 44 + 18, 800 - 16, 30, leds)
    return img


def main():
    panel = Panel()
    frames = sorted((BUILD / "demo").glob("frame_*.bin"))
    if not frames:
        raise SystemExit("no frames: run the scenarios example in demo mode first")
    fonts = (ImageFont.truetype(FONT, 26, index=2), ImageFont.truetype(FONT, 18, index=5))
    out = BUILD / "demo.mp4"
    names = []
    with tempfile.TemporaryDirectory() as tmp:
        tmp = Path(tmp)
        for k, path in enumerate(frames):
            name, page, leds, screen = read_frame(path, panel)
            if not names or names[-1] != name:
                names.append(name)
            compose(name, leds, screen, fonts, len(names) - 1, 13).save(tmp / f"f_{k:05d}.png")
        ffmpeg = shutil.which("ffmpeg") or "/opt/homebrew/bin/ffmpeg"
        subprocess.run([ffmpeg, "-y", "-loglevel", "error", "-framerate", str(FPS), "-i", str(tmp / "f_%05d.png"),
                        "-c:v", "libx264", "-pix_fmt", "yuv420p", "-crf", "18", "-movflags", "+faststart",
                        str(out)], check=True)
    print(f"{out}  ({len(frames)} frames, {len(frames) / FPS:.0f} s)")


if __name__ == "__main__":
    main()
