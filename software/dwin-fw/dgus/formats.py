"""dwin file formats"""

import io
import struct

from PIL import Image


def crc16_modbus(data):
    crc = 0xFFFF
    for b in data:
        crc ^= b
        for _ in range(8):
            crc = (crc >> 1) ^ 0xA001 if crc & 1 else crc >> 1
    return crc


# icl format

ICL_INDEX_END = 0x8000
ICL_MAX_IMAGES = (ICL_INDEX_END - 0x10) // 4
JPEG_MAX_BYTES = 248 * 1024  # max image size


def _split_jpeg(data):
    """split jpeg segments"""
    assert data[:2] == b"\xff\xd8", "not a JPEG"
    i, segs = 2, []
    while True:
        assert data[i] == 0xFF, f"bad marker at {i}"
        m = data[i + 1]
        (length,) = struct.unpack_from(">H", data, i + 2)
        seg = data[i:i + 2 + length]
        if m == 0xDA:
            segs.append(seg)
            return segs, data[i + 2 + length:]
        if m in (0xE0, 0xDB, 0xC0, 0xC4):  # standard jpeg markers
            segs.append(seg)
        elif m in (0xC2, 0xDD):
            raise ValueError("progressive JPEG or restart markers are not supported by the T5L")
        i += 2 + length


def jpeg_record(img, quality=92, subsampling=0):
    """encode icl record"""
    img = img.convert("RGB")
    buf = io.BytesIO()
    img.save(buf, "JPEG", quality=quality, subsampling=subsampling, optimize=False, progressive=False)
    segs, scan = _split_jpeg(buf.getvalue())
    assert scan.endswith(b"\xff\xd9")
    header = b"\xff\xd8" + b"".join(segs)
    pad = (4 - (10 + len(header)) % 4) % 4
    if pad:
        sos = segs[-1]
        (ls,) = struct.unpack_from(">H", sos, 2)
        sos = sos[:2] + struct.pack(">H", ls + pad) + sos[4:] + b"\xff" * pad
        header = b"\xff\xd8" + b"".join(segs[:-1]) + sos
    zeros = 8 + (4 - (len(scan) + 8) % 4) % 4
    body = scan + b"\x00" * zeros
    if len(header) + len(scan) > JPEG_MAX_BYTES:
        raise ValueError(f"{img.size} image is {len(header) + len(scan)} bytes; the T5L limit is {JPEG_MAX_BYTES}")
    return struct.pack(">HHHI", img.width, img.height, len(header), len(body)) + header + body


def write_icl(images, quality=92, subsampling=0):
    """write icl file"""
    if len(images) > ICL_MAX_IMAGES:
        raise ValueError(f"{len(images)} images; one ICL holds at most {ICL_MAX_IMAGES}")
    records, offsets, pos = [], [], ICL_INDEX_END
    for img in images:
        rec = jpeg_record(img, quality, subsampling)
        offsets.append(pos)
        records.append(rec)
        pos += len(rec)
    index = b"".join(struct.pack(">I", o) for o in offsets)
    index = index.ljust(ICL_INDEX_END - 0x10, b"\x00")
    total = ICL_INDEX_END + sum(len(r) for r in records)
    body = struct.pack(">I", total - 8 - 0x7000) + bytes([0x04]) + struct.pack(">H", len(images) - 1) + b"\x00"
    body += index + b"".join(records)
    return b"DGUS_3" + struct.pack("<H", crc16_modbus(body)) + body


def read_icl(data):
    """read icl file"""
    assert data[:6] == b"DGUS_3", "not an ICL"
    assert struct.unpack_from("<H", data, 6)[0] == crc16_modbus(data[8:]), "ICL CRC mismatch"
    (maxid,) = struct.unpack_from(">H", data, 13)
    out = []
    for i in range(maxid + 1):
        (off,) = struct.unpack_from(">I", data, 0x10 + 4 * i)
        if off == 0:
            out.append(None)
            continue
        w, h, hl, bl = struct.unpack_from(">HHHI", data, off)
        hdr = data[off + 10:off + 10 + hl]
        body = data[off + 10 + hl:off + 10 + hl + bl]
        # unpad sos byte
        sos = hdr.rfind(b"\xff\xda")
        (ls,) = struct.unpack_from(">H", hdr, sos + 2)
        ns = hdr[sos + 4]
        real = 6 + 2 * ns
        hdr = hdr[:sos + 2] + struct.pack(">H", real) + hdr[sos + 4:sos + 2 + real]
        end = body.rfind(b"\xff\xd9")
        out.append((w, h, hdr + body[:end + 2]))
    return out


def decode_icl_image(entry):
    w, h, jpeg = entry
    img = Image.open(io.BytesIO(jpeg))
    img.load()
    assert img.size == (w, h)
    return img.convert("RGB")


# show file format

SHOW_TABLE = 4092
SHOW_RECORDS = 0x4000
MAX_CONTROLS_PER_PAGE = 64  # max controls per-page


def control(type_code, vp, words, sp=0xFFFF):
    """encode control record"""
    body = struct.pack(">H", vp) + b"".join(struct.pack(">H", w & 0xFFFF) for w in words)
    assert len(body) <= 26, "descriptor too long"
    rec = bytes([0x5A, type_code]) + struct.pack(">HH", sp, len(body) // 2) + body
    return rec.ljust(32, b"\x00")


def basic_graphics(vp, area=(0, 0, 799, 479), line_scale=0):
    """basic graphics record"""
    xs, ys, xe, ye = area
    # basic graphics descriptor
    return control(0x21, vp, [xs, ys, xe, ye, 0x0000, 0x0000, (0x00 << 8) | line_scale])


def qr_code(vp, x, y, unit, colour_bg=None, colour_fg=None):
    """qr code record"""
    mode = 0x00 if colour_bg is None else 0x80
    # qr descriptor words
    return control(0x25, vp, [x, y, unit, (0x00 << 8) | mode, colour_bg or 0xFFFF, colour_fg or 0x0000, 0])


def brightness_area(vp, area):
    """brightness area record"""
    xs, ys, xe, ye = area
    return control(0x26, vp, [xs, ys, xe, ye])


def write_show_file(pages):
    """write show file"""
    if len(pages) > SHOW_TABLE:
        raise ValueError("too many pages")
    table, records, off = [], [], SHOW_RECORDS
    for p, recs in enumerate(pages):
        if len(recs) > MAX_CONTROLS_PER_PAGE:
            raise ValueError(f"page {p} has {len(recs)} controls (max {MAX_CONTROLS_PER_PAGE})")
        table.append((len(recs), off))
        records += recs
        off += 32 * len(recs)
    if off > 0xFFFF:
        raise ValueError("records overflow the 16-bit page offsets")
    table += [(0, off)] * (SHOW_TABLE - len(table))
    head = bytes([0x14]) + b"DGUS_2" + bytes([0x10]) + struct.pack(">H", len(pages) - 1) + bytes(6)
    tab = b"".join(bytes([n, 0]) + struct.pack(">H", o) for n, o in table)
    return head + tab + b"".join(records) + b"\xff" * 32


def read_show_file(data):
    assert data[:8] == bytes([0x14]) + b"DGUS_2" + bytes([0x10])
    (maxpage,) = struct.unpack_from(">H", data, 8)
    pages = []
    for p in range(maxpage + 1):
        n, zero, off = data[16 + 4 * p], data[17 + 4 * p], struct.unpack_from(">H", data, 18 + 4 * p)[0]
        assert zero == 0
        pages.append([data[off + 32 * i:off + 32 * i + 32] for i in range(n)])
    return pages


# config file format


def write_cfg(background_icl=32, baud=115200, brightness=80, load_22=True, crc=False):
    """write config file"""
    flags = (0x80 if crc else 0) | (0x20 if load_22 else 0)  # config flag bits
    divisor = round(3_225_600 / baud)
    cfg = bytearray(46)
    cfg[0:5] = b"T5LC1"
    cfg[5] = flags
    cfg[6] = 0x00  # preserve flash contents
    cfg[7] = 0x00  # no audio file
    cfg[8] = background_icl
    cfg[9] = 0x28  # touch scan rate
    cfg[10:12] = struct.pack(">H", divisor)
    cfg[12] = brightness  # active backlight level
    cfg[13] = brightness  # standby backlight level
    cfg[14:16] = struct.pack(">H", 1000)  # standby delay time
    # factory lcd timing
    return bytes(cfg)


# init memory format


def write_init_file(words):
    """write init file"""
    data = bytearray(0x20004)
    for vp, value in words.items():
        assert 0x1000 <= vp <= 0xFFFF, f"VP {vp:#x} is not initialised from 22_Config.bin"
        struct.pack_into(">H", data, vp * 2, value & 0xFFFF)
    return bytes(data)


TOUCH_FILE_EMPTY = b"\xff\xff"  # empty touch file
