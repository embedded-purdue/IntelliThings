"""dgus toolchain tests"""

import json
import struct
from pathlib import Path

import pytest
from PIL import Image

from dgus import build, formats as F, ui

ROOT = Path(__file__).resolve().parent.parent


def test_crc_matches_dwin_guide_examples():
    # guide crc vector
    assert F.crc16_modbus(bytes([0x83, 0x00, 0x0F, 0x01])).to_bytes(2, "little") == b"\xed\x90"
    assert F.crc16_modbus(bytes([0x82, 0x10, 0x00, 0x31, 0x32])).to_bytes(2, "little") == b"\xcc\x9b"


def test_icl_layout_matches_dgus_output():
    imgs = [Image.new("RGB", (800, 480), (11, 15, 20)), Image.new("RGB", (17, 9), (200, 30, 30))]
    data = F.write_icl(imgs)
    assert data[:6] == b"DGUS_3"
    assert struct.unpack_from(">I", data, 8)[0] == len(data) - 8 - 0x7000
    assert data[12] == 0x04 and struct.unpack_from(">H", data, 13)[0] == 1 and data[15] == 0
    first = struct.unpack_from(">I", data, 0x10)[0]
    assert first == 0x8000
    w, h, hl, bl = struct.unpack_from(">HHHI", data, first)
    assert (w, h) == (800, 480)
    assert (10 + hl) % 4 == 0 and bl % 4 == 0  # four byte alignment
    assert data[first + 10:first + 12] == b"\xff\xd8"
    back = [F.decode_icl_image(e) for e in F.read_icl(data)]
    assert [b.size for b in back] == [(800, 480), (17, 9)]


def test_show_file_round_trip():
    pages = [[F.basic_graphics(0x5000), F.qr_code(0x6400, 1, 2, 5)], [], [F.brightness_area(0x6500, (0, 0, 9, 9))]]
    data = F.write_show_file(pages)
    assert data[:8] == b"\x14DGUS_2\x10" and data[-32:] == b"\xff" * 32
    back = F.read_show_file(data)
    assert [len(p) for p in back] == [2, 0, 1]
    assert back[0][0][:6] == b"\x5a\x21\xff\xff\x00\x08"  # descriptor length words
    assert back[0][1][:6] == b"\x5a\x25\xff\xff\x00\x08"
    assert back[2][0][:6] == b"\x5a\x26\xff\xff\x00\x05"


def test_cfg_keeps_factory_lcd_timing():
    cfg = F.write_cfg()
    assert len(cfg) == 46 and cfg[:5] == b"T5LC1"
    assert cfg[6] & 0x08 == 0  # preserve flash bit
    assert cfg[8] == 32 and struct.unpack(">H", cfg[10:12])[0] == 28
    assert cfg[0x10:0x12] == b"\x00\x00"


def test_word_styles_cover_every_firmware_word():
    for word in ui.AIR_WORDS + ui.BOOT_WORDS:
        assert set(word) <= set(ui.WORDS), word


def test_full_build_verifies(capsys):
    build.build(check=True)
    out = capsys.readouterr().out
    assert "all check out" in out
    manifest = json.loads((ROOT / "build/manifest.json").read_text())
    for lib in manifest["libraries"]:
        assert lib["end"] - lib["first"] <= ui.MAX_ICONS_PER_ICL
    for page in F.read_show_file((ROOT / "build/DWIN_SET/14ShowFile.bin").read_bytes()):
        assert len(page) <= F.MAX_CONTROLS_PER_PAGE


def test_power_on_screen_draws_from_preloaded_memory():
    from dgus.sim import Panel

    p = Panel()
    mem = list(p.init) + [0] * (0x10000 - len(p.init))
    img = p.render(0, mem)
    # verify sprite elements
    assert img.getpixel((400, 138)) != p.backgrounds[0].getpixel((400, 138))


@pytest.mark.skipif(not (ROOT / "build/vp/home.json").exists(), reason="run the Rust scenarios example first")
def test_simulated_home_matches_concept_layout():
    from dgus.sim import Panel, render_dump

    img = render_dump(Panel(), ROOT / "build/vp/home.json")
    concept = ROOT.parent / "repo/software/display-concepts/previews/home.png"
    if not concept.exists():
        pytest.skip("concept previews not available")
    ref = Image.open(concept).convert("RGB")
    diff = sum(abs(a - b) for pa, pb in zip(img.getdata(), ref.getdata()) for a, b in zip(pa, pb))
    # layout comparison check
    assert diff / (800 * 480 * 3) < 6.0


def test_vp_map():
    """verify vp map"""
    from dgus import vpmap

    used = vpmap.check()
    assert len(used) < vpmap.WORDS
    names = [e.name for e in vpmap.entries()]
    assert len(names) == len(set(names))
    ex = dict(vpmap.examples())
    assert ex["CO₂ = 812 ppm"].hex(" ").upper() == "5A A5 05 82 10 03 03 2C"
    assert ex["AI message “Hi!”"].hex(" ").upper() == "5A A5 07 82 11 00 48 69 21 00"
    # match mqtt snapshot
    sensors = [e.var.mqtt for e in vpmap.entries() if e.section == "sensors"]
    assert sensors == ["temperature_c", "humidity_pct", "voc_index", "co2_ppm", "lux", "distance_mm",
                       "pm1_0_ugm3", "pm2_5_ugm3", "pm10_ugm3", "presence"]
