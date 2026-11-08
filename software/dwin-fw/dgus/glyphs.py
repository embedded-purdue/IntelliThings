"""anti-aliased glyph sprites"""

from dataclasses import dataclass, field

from PIL import Image, ImageDraw

from . import design as d

ASCII = "".join(chr(c) for c in range(0x20, 0x7F))
EXTRA = "°₂µ³·…’‘“”–—−×"
TEXT_CHARS = ASCII + EXTRA
DIGITS = "0123456789"
NUMERIC = DIGITS + ".,:-−+% "


@dataclass(frozen=True)
class TextStyle:
    """glyph style definition"""

    name: str
    size: float
    weight: str
    fg: tuple
    bg: tuple
    chars: str = TEXT_CHARS


@dataclass
class Glyph:
    char: str
    image: Image.Image  # cell sprite image
    advance: float  # fractional pen advance
    x_offset: int  # pen x offset


@dataclass
class GlyphSet:
    style: TextStyle
    ascent: int  # cell baseline ascent
    height: int  # cell pixel height
    glyphs: dict = field(default_factory=dict)  # glyph character map


    def measure(self, s):
        return sum(self.glyphs[c].advance for c in s if c in self.glyphs)


def _supersampled_font(style):
    return d.font_px(round(style.size * d.S), style.weight)


def build_glyph_set(style):
    f = _supersampled_font(style)
    asc, desc = f.getmetrics()
    # cell pixel bounds
    ascent = -(-asc // d.S) + 1
    height = ascent + -(-desc // d.S) + 1
    gs = GlyphSet(style=style, ascent=ascent, height=height)
    for ch in style.chars:
        gs.glyphs[ch] = _render(style, f, ch, ascent, height)
    return gs


def _render(style, f, ch, ascent, height):
    sub = ch == "₂"  # subscript two glyph
    draw_ch, ff, drop = ch, f, 0
    if sub:
        draw_ch = "2"
        ff = d.font_px(round(style.size * 0.62 * d.S), style.weight)
        drop = round(style.size * 0.22 * d.S)
    advance2x = ff.getlength(draw_ch)
    if ch == " ":
        bbox = (0, 0, 0, 0)
    else:
        bbox = ff.getbbox(draw_ch, anchor="ls")  # baseline glyph bbox
    left2x = min(0, bbox[0])
    right2x = max(advance2x, bbox[2])
    # cell bounds snap
    left = int(left2x // d.S) if left2x < 0 else 0
    right = -(-int(right2x) // d.S)
    width = max(1, right - left)
    img = Image.new("RGB", (width * d.S, height * d.S), style.bg)
    dr = ImageDraw.Draw(img)
    if ch != " ":
        dr.text((-left * d.S, ascent * d.S + drop), draw_ch, font=ff, fill=style.fg, anchor="ls")
    img = img.resize((width, height), Image.LANCZOS)
    return Glyph(char=ch, image=d.rgb666_image(img), advance=advance2x / d.S, x_offset=left)


def render_string(gs, s, letter_spacing=0.0):
    """measure layout width"""
    width = max(1, round(gs.measure(s) + 2))
    img = Image.new("RGB", (width, gs.height), gs.style.bg)
    pen = 0.0
    for ch in s:
        g = gs.glyphs.get(ch)
        if g is None:
            continue
        img.paste(g.image, (round(pen) + g.x_offset, 0))
        pen += g.advance + letter_spacing
    return img
