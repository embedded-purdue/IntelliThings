"""software panel simulator"""

import argparse
import json
import struct
from pathlib import Path

import qrcode
from PIL import Image, ImageDraw, ImageFont

from . import formats as F

ROOT = Path(__file__).resolve().parent.parent
BUILD = ROOT / "build"
CONCEPTS = ROOT.parent / "repo/software/display-concepts/previews"

# transparency color threshold
TRANSPARENT_BELOW = 8


class Panel:
    def __init__(self, dwin_set=BUILD / "DWIN_SET", manifest=BUILD / "manifest.json"):
        self.dir = Path(dwin_set)
        self.manifest = json.loads(Path(manifest).read_text())
        self.backgrounds = [F.decode_icl_image(e) for e in F.read_icl((self.dir / self.manifest["background_icl"]).read_bytes())]
        self.libraries = {}
        for lib in self.manifest["libraries"]:
            self.libraries[lib["id"]] = F.read_icl((self.dir / lib["file"]).read_bytes())
        self.cache = {}
        self.pages = F.read_show_file((self.dir / "14ShowFile.bin").read_bytes())
        init = (self.dir / "22_Config.bin").read_bytes()
        self.init = list(struct.unpack(f">{len(init) // 2}H", init[: len(init) // 2 * 2]))

    def icon(self, icl, i):
        key = (icl, i)
        if key not in self.cache:
            entry = self.libraries[icl][i]
            img = F.decode_icl_image(entry).convert("RGBA")
            px = img.load()
            for y in range(img.height):
                for x in range(img.width):
                    r, g, b, _ = px[x, y]
                    if r < TRANSPARENT_BELOW and g < TRANSPARENT_BELOW and b < TRANSPARENT_BELOW:
                        px[x, y] = (0, 0, 0, 0)
            self.cache[key] = img
        return self.cache[key]

    def render(self, page, mem):
        """render display page"""
        img = self.backgrounds[page].copy().convert("RGBA")
        for rec in self.pages[page]:
            kind = rec[1]
            vp = struct.unpack(">H", rec[6:8])[0]
            if kind == 0x21:
                self._basic_graphics(img, mem, vp)
            elif kind == 0x25:
                x, y, unit = struct.unpack(">HHH", rec[8:14])
                self._qr(img, mem, vp, x, y, unit)
            elif kind == 0x26:
                xs, ys, xe, ye = struct.unpack(">HHHH", rec[8:16])
                level = min(100, mem[vp])
                if level < 100:
                    region = img.crop((xs, ys, xe + 1, ye + 1))
                    region = Image.blend(Image.new("RGBA", region.size, (0, 0, 0, 255)), region, level / 100)
                    img.paste(region, (xs, ys))
        return img.convert("RGB")

    def _basic_graphics(self, img, mem, vp):
        cmd, n = mem[vp], mem[vp + 1]
        if cmd & 0xFF == 0x07:
            icl = cmd >> 8
            at = vp + 2
            for _ in range(n):
                x, y, i = mem[at], mem[at + 1], mem[at + 2]
                at += 3
                if x >> 8 == 0xFF:
                    break
                if x >> 8 == 0xFE:
                    continue
                icon = self.icon(icl, i)
                img.alpha_composite(icon, (x, y))
        elif cmd == 0x0002 and n:
            colour = mem[vp + 2]
            rgb = (((colour >> 11) & 0x1F) << 3, ((colour >> 5) & 0x3F) << 2, (colour & 0x1F) << 3, 255)
            pts = []
            for k in range(n + 1):
                x, y = mem[vp + 3 + 2 * k], mem[vp + 4 + 2 * k]
                if x >> 8 == 0xFF:
                    break
                pts.append((x, y))
            ImageDraw.Draw(img).line(pts, fill=rgb, width=1)

    def _qr(self, img, mem, vp, x, y, unit):
        data = bytearray()
        for k in range(230):
            w = mem[vp + k]
            if w in (0x0000, 0xFFFF):
                break
            hi, lo = w >> 8, w & 0xFF
            data.append(hi)
            if lo in (0x00, 0xFF):
                break
            data.append(lo)
        if not data:
            return
        qr = qrcode.QRCode(version=7 if len(data) < 155 else 13, border=0, box_size=1,
                           error_correction=qrcode.constants.ERROR_CORRECT_L)
        qr.add_data(bytes(data))
        qr.make(fit=False)
        d = ImageDraw.Draw(img)
        for r, row in enumerate(qr.get_matrix()):
            for c, on in enumerate(row):
                d.rectangle([x + c * unit, y + r * unit, x + (c + 1) * unit - 1, y + (r + 1) * unit - 1],
                            fill=(0, 0, 0, 255) if on else (255, 255, 255, 255))


def memory(panel, dump):
    mem = list(panel.init) + [0] * (0x10000 - len(panel.init))
    base = dump["base"]
    mem[base:base + len(dump["words"])] = dump["words"]
    return mem


def render_dump(panel, path):
    dump = json.loads(Path(path).read_text())
    return panel.render(dump["page"], memory(panel, dump))


def compare_sheet(names, out):
    font = ImageFont.truetype("/System/Library/Fonts/Avenir Next.ttc", 22, index=2)
    tw, th, pad, cap = 400, 240, 16, 36
    sheet = Image.new("RGB", (2 * tw + 3 * pad, len(names) * (th + cap) + pad), (17, 17, 17))
    d = ImageDraw.Draw(sheet)
    d.text((pad, 4), "concept", font=font, fill=(160, 170, 180))
    d.text((2 * pad + tw, 4), "generated DWIN_SET + firmware layout (simulated)", font=font, fill=(160, 170, 180))
    for i, name in enumerate(names):
        y = cap + i * (th + cap)
        concept = CONCEPTS / f"{name}.png"
        if concept.exists():
            sheet.paste(Image.open(concept).convert("RGB").resize((tw, th), Image.LANCZOS), (pad, y))
        sim = BUILD / "sim" / f"{name}.png"
        sheet.paste(Image.open(sim).resize((tw, th), Image.LANCZOS), (2 * pad + tw, y))
        d.text((pad, y + th + 4), name, font=font, fill=(220, 225, 230))
    sheet.save(out)


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("dumps", nargs="*")
    ap.add_argument("--all", action="store_true", help="render every build/vp/*.json")
    ap.add_argument("--compare", action="store_true", help="also write build/sim/compare.png")
    a = ap.parse_args()
    panel = Panel()
    dumps = [Path(p) for p in a.dumps] or sorted((BUILD / "vp").glob("*.json")) if a.all or not a.dumps else [Path(p) for p in a.dumps]
    (BUILD / "sim").mkdir(parents=True, exist_ok=True)
    names = []
    for path in dumps:
        img = render_dump(panel, path)
        out = BUILD / "sim" / f"{path.stem}.png"
        img.save(out)
        names.append(path.stem)
        print(out.relative_to(ROOT))
    if a.compare:
        order = ["home", "co2-rising", "agent-thinking", "action-toast", "sensors", "rooms", "alert", "offline",
                 "night", "away", "status-boot", "setup-wifi", "test-pattern"]
        compare_sheet([n for n in order if n in names], BUILD / "sim" / "compare.png")
        print((BUILD / "sim" / "compare.png").relative_to(ROOT))


if __name__ == "__main__":
    main()
