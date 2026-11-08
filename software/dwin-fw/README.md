# IntelliThings companion display

Software for the node's **DWIN DMG80480T050_09WN** (5", 800×480, T5L0 / DGUS II, no touch) and its
**24-LED light bar**, matching the approved concepts in
`repo/software/display-concepts` (home, CO₂ rising, thinking, action, sensors, rooms, alert,
offline, night, away, boot, setup, test).

It replaces DWIN's Windows-only DGUS tool: a Python build turns the designs into the `DWIN_SET`
folder the panel loads from microSD, plus a Rust table the firmware uses to draw on it.

```
dgus/                 DWIN_SET builder (Python)
  design.py           palette, fonts, icons (ported from the concepts)
  pages.py            static page backgrounds
  sprites.py          icons, tile/room skins, orb frames, chip parts
  glyphs.py           anti-aliased text, one sprite per character per text style
  ui.py               the contract: pages, layers, text styles, VP map
  formats.py          ICL / 14ShowFile / CFG / 22_Config writers (+ readers)
  build.py            -> build/DWIN_SET, firmware/companion-ui/src/generated.rs
  sim.py              software T5L: renders pages from the DWIN_SET + VP memory
firmware/
  companion-ui/       no_std Rust crate: DGUS driver, screens, light-bar patterns (host-tested)
  companion-node/     ESP32-C6 bench firmware (esp-hal 1.2, like software/firmware): demo mode
  sensors/            no_std drivers for the node's five I2C sensors + readout (host-tested)
  sensor-node/        ESP32-C6-DevKitC-1 firmware: prints the five sensors every second
  vpmap.py            the VP map the node firmware writes -> handoff/
handoff/              for the software team: VP_MAP.html (+ csv, json), dwin_vp.rs, HANDOFF.md
  sensor-node/        the sensor firmware to flash (README, .bin, .elf); `make sensors` rebuilds
                      it and handoff/sensor-node.zip
tools/
  companion.py        write the display's VPs by name from a laptop; `play` = scripted live demo
  dwin.py             talk to the panel from a laptop (USB-UART): info, page, read/write VPs
  sdcard.sh           format a microSD card for the panel (macOS) and copy DWIN_SET
tests/                pytest for the builder
```

## Bring-up

### 1. Parts

- The display, its **8-pin 2.0 mm cable** (SD003) and a **12 V / ≥1 A** DC adapter (the panel
  takes 6–36 V; 5 V is too little).
- An **ESP32-C6** board: the Waveshare ESP32-C6-LCD-1.47 or an ESP32-C6-DevKitC-1. Any C6 works
  if it has free GPIOs; the classic ESP32 (ELEGOO kit) and the S3 would need other toolchains.
- A **microSD card** for the display (not for the ESP32). DWIN supports SD/SDHC and recommends
  1–16 GB; `tools/sdcard.sh` makes an 8 GB partition on bigger cards.
- Optional: the WS2812B strip (24 LEDs), a 5 V supply for it, 330 Ω on its data line.

### 2. Make sure the display's serial port is TTL

On the back of the display, next to the white connector marked **6-36V IN**, there are two
tiny pads labelled **ON=TTL OFF=232**.

- **Bridged** (a tiny 0 Ω part or a solder blob across them): TTL. This is the factory
  default per DWIN's guide.
- **Two bare pads**: RS232. Bridge them with a little solder.

Then check it: power the display from 12 V and measure **pin 4 (TX2) against pin 7 (GND)**.
A positive voltage means TTL; a negative one means RS232: don't connect it to an ESP32. Ours idles
at **+5 V**, not the 3.3 V the datasheet suggests, which is why TX2 goes through a divider below.

### 3. Wire it

Display connector (DWIN datasheet; the pin names are also printed next to the connector):

| Pin | Signal | Connect to |
|---|---|---|
| 1, 2 | VCC (6–36 V) | 12 V adapter **+** |
| 3 | TX4 | — |
| 4 | **TX2** (display → ESP32) | **divider** → ESP32 **GPIO19** |
| 5 | **RX2** (ESP32 → display) | ESP32 **GPIO18** |
| 6 | RX4 | — |
| 7, 8 | GND | adapter **−** and an ESP32 **GND** |

The divider: pin 4 → **1 kΩ** → GPIO19, and GPIO19 → **2 kΩ** → GND (5 V × 2/3 ≈ 3.3 V; the C6's
pins are not 5 V tolerant). Any pair near 1:2 works, e.g. 2 × 560 Ω in series over 2.2 kΩ. The
display's RX2 reads the C6's 3.3 V fine, so GPIO18 connects directly.

Optional LED strip: three wires, changed only with the USB unplugged. Strip 5V to the board's
5V, strip GND to GND, and DIN from **GPIO20** through 330 Ω near the strip. The GND is
**required**: without it the strip's current goes through GPIO20, which flickers at random and
can destroy the ESP32. On the Waveshare board, held **screen side up with USB at the top**, GP18/GP19/GP20 are on the
**left** header (TXD, RXD, GP13, GP12, GP23, GP20, GP19, GP18, GP9 from the top) and GND is on
the right one. `docs/wiring.png` shows the whole thing. They are free on the DevKitC-1 too.
The ESP32 is powered from USB-C. Never connect the display's 12 V to the ESP32.

115200 baud, 8N1.

### 4. Load the screens (microSD, once per design change)

```bash
make setup          # first time: Python venv
make dwin           # -> build/DWIN_SET (≈4.5 MB), verified by parsing it back
make sd DISK=/dev/diskN    # ERASES that card, formats FAT32/4 KB clusters, copies DWIN_SET
```

`tools/sdcard.sh` with no argument lists external disks so you can find the card. It refuses
non-removable disks and asks before erasing.

Then: **display off → insert card → power on**. A blue download screen counts the files; wait
for **"SD card process end"**, power off, remove the card, power on. You should now see the orb,
**IntelliThings** and **"Waiting for the controller…"**. That screen alone exercises every
generated file (CFG, backgrounds, the icon library, the controls and the preloaded memory).
The panel keeps the files in its flash; the card only goes back in after `make dwin` changes
them.

### 5. Flash the ESP32-C6

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh   # once
rustup target add riscv32imac-unknown-none-elf && rustup component add rust-src
cargo install espflash --locked
make flash          # = cd firmware/companion-node && cargo run --release
```

No Rust on the computer? `flash.txt` flashes the prebuilt `build/companion-node.elf` with a
downloaded espflash. `make firmware` refreshes that file.

The serial monitor prints:

```
DWIN panel answered at 115200 baud (CRC off): DGUS version 0x65, OS version 0x22
Companion DWIN_SET found - drawing the demo
> co2-rising      page 1 (drawing on the panel) - 89 writes, 89 acks, 0 rx errors
> agent-thinking  page 1 (drawing on the panel) - 355 writes, 355 acks, 0 rx errors
```

The demo cycles through all 13 screen states with animated demo data (typing, CO₂ rising, the
action toast, boot checklist, night, away…) and the light bar follows each one: on the strip if
fitted, and on the board's RGB LED and its 1.47" screen, which also shows the link state and the
current scene. Writes and acks should stay equal; a gap means frames got lost on the way. If the
panel answers but doesn't have our DWIN_SET yet, the firmware says so and holds one page.

### Troubleshooting

| Symptom | Check |
|---|---|
| "No answer from the panel" | TTL pads (step 2), TX/RX crossed (ESP TX → pin 5), shared GND, 12 V present |
| Blue screen says 0 files / no download screen | FAT32 with 4 KB clusters (`tools/sdcard.sh`), `DWIN_SET` at the card root, card ≤ 16 GB partition, power off before inserting |
| Black screen with weak backlight after loading | 32.ICL missing; reload the card |
| Answers, but "doesn't have the companion DWIN_SET" | Load the card (step 4); the firmware looks for a value the DWIN_SET preloads |
| Garbled or partial screens | Rebuild both: `make dwin` then `make flash` (the firmware's tables must match the DWIN_SET) |

`tools/dwin.py --port /dev/tty.usbserial-… info` talks to the panel straight from a laptop
through any 3.3 V USB-UART adapter, for checking the panel without an ESP32.

## How it works

DGUS II panels draw their own screens; the host only writes 16-bit **VP** variables over UART
(`5A A5 <len> 82 <VP> <data>`). DGUS's built-in fonts are 1-bit and can't follow the concepts, so
this project uses one DGUS II control in a particular way:

- Every **page** is a background image (`32.ICL`, image *n* = page *n*) holding everything static.
- Every page has three **sprite layers** (skin, main, overlay). Each is a *basic graphics*
  control (type `0x21`) whose VP block holds an **icon-display command list**:
  `0x2207 n  x y id  x y id … 0xFF00` (high byte = icon library). The firmware rewrites these lists
  to place any sprite anywhere.
- All text is sprites: every character of every text style is pre-rendered anti-aliased in
  Avenir Next over the exact background colour it sits on (DGUS can't blend). The firmware has
  each glyph's advance (generated) and lays strings out itself: wrapping, alignment, the typing
  animation, the caret.
- Trend lines are basic-graphics polylines; the setup QR is the QR control (`0x25`); the toast's
  backdrop is a brightness control (`0x26`).
- The firmware keeps a shadow of the VP memory and sends only the words that changed, so an
  animation frame is a few bytes. Before a page switch it empties the shared layers.
- `22_Config.bin` preloads VP memory at power-up: empty layers, the boot screen's "Waiting for the
  controller…", and two marker words. `SIGNATURE_VP` (0xC0DE) tells the firmware our DWIN_SET is
  loaded; `SESSION_VP` starts at 0, the firmware writes its own value on connecting, and a 0
  later means the panel restarted. Both sit outside the VP range the firmware's shadow covers,
  so frame updates never touch them (a compile-time check in `ui.rs` keeps it that way).

Generated files (sector = 256 KB of the panel's 16 MB flash):

| File | Contents |
|---|---|
| `T5LCFG_800480.CFG` | 115200 baud, CRC off, backgrounds in 32.ICL, load 22_Config; factory LCD timing untouched |
| `13TouchFile.bin` | `FF FF` (no touch controls) |
| `14ShowFile.bin` | controls per page |
| `22_Config.bin` | VP memory at power-up |
| `32.ICL` | 9 page backgrounds |
| `34.ICL` | 3,894 sprites and glyphs |

## Changing the design

Edit `dgus/pages.py` (backgrounds), `dgus/sprites.py` (icons, skins), `dgus/ui.py` (text styles,
layers) or the screen layout in `firmware/companion-ui/src/screens.rs`, then:

```bash
make sim     # rebuild, render all 13 screens -> build/sim/*.png and build/sim/compare.png
make test    # Python + Rust tests
```

Reload the SD card **and** reflash: `generated.rs` (sprite ids, glyph metrics, VP map) changes
with the DWIN_SET. `build/sim/compare.png` puts each concept next to the simulated panel.

## Using it from the team firmware

The node firmware feeds the display through **VPs**: 16-bit words at `0x1000`–`0x17FF`, written
with ordinary DGUS write frames (`5A A5 <len> 82 <VP> <data>`), as the firmware architecture
plans for the DWIN display. `handoff/VP_MAP.html` is the table; `handoff/HANDOFF.md` walks
through it. The companion turns the values into the screens and the light bar.

- **Bench (two chips):** UART from the node to the companion C6's node link, 115200 8N1: node TX
  → **GPIO2**, **GPIO3** → node RX, GND. The first write ends the demo. `handoff/dwin_vp.rs` is a
  drop-in Rust file with every address and the frame encoders. `tools/companion.py` writes the
  same VPs from a laptop over the companion's USB (`play` runs a scripted live demo).
- **One chip, later:** `software/firmware` (esp-hal 1.2 + embassy) depends on `companion-ui` by
  path, keeps one `vp::Inputs`, calls `inputs.write(vp, words, now)`, and runs the frame loop of
  `firmware/companion-node/src/bin/main.rs` (`inputs.apply`, `director.decide`, `ui.frame`).

`dgus/vpmap.py` is the VP map's single source: `make dwin` regenerates the HTML, CSV, JSON, the
drop-in file and the decoder's constants (`firmware/companion-ui/src/vp_map.rs`).

Optional extras the screens already support, if the AI message contract grows to carry them:
action chips, a "thinking" line, toasts and alerts (all in the VP map). With a plain
`{"text","ts"}` payload the home screen works as is; the AI section shows about 110 characters
(80 with action chips), measured by `cargo run --example text_fit` in `firmware/companion-ui`.

Notes against the team docs: the concepts' sensors/rooms pages hide the AI message, which the
layout rule forbids, so page rotation is **off** by default (`Settings::rotate_pages`). The
SGP40 reports a VOC *index*, not ppb, so the tile says "VOC index" rather than "TVOC ppb". On the
Waveshare board GPIO6/7 (the I2C pins on the rigs) belong to its LCD/SD; pick other I2C pins there.

## What's verified, and what only the panel can confirm

Verified here:
- Every file layout was checked byte for byte against DGUS-generated DWIN_SETs from four public
  projects (DGUS 7.641–7.651) and DWIN's *T5L DGUS II Application Development Guide* V2.9;
  `make dwin` parses everything back (CRCs, JPEG decode, record layout).
- `dgus/sim.py` renders every screen from the generated binaries plus the firmware's VP memory
  (`build/sim/compare.png`); all 13 match the concepts. Rust and Python tests pass; the bench
  firmware builds for the C6.

Confirmed on the DMG80480T050_09WN (DGUS 0x65, OS 0x22):
- It loads the generated DWIN_SET and draws the boot page, sprites and glyphs from a single
  3,894-icon library.
- Icon lists redraw cleanly: when a list changes, the panel restores the background under the
  old icons and leaves the other layers alone (`cargo run --release --bin paneltest` in
  `firmware/companion-node` steps through this on the boot page).
- A full 104 s demo loop at 115200 baud: every write frame acknowledged, no receive errors.

Still to confirm on the panel (none affect wiring safety):
- **Icon transparency.** The icon-display command filters near-black pixels. Sprites are drawn
  over their exact background colour, so it shouldn't show; the toast card relies on it for its
  rounded corners.
- **Brightness control behind the toast.** If it misbehaves, the toast still works; the scrim
  can be removed in `page_controls()`.

## Sources

- DWIN, *T5L_DGUSII Application Development Guide* V2.9:
  https://www.dwin-global.com/development-guide/
- DWIN, *DMG80480T050_09WN datasheet* (pinout, TTL pads, 6–36 V): via
  https://evelta.com/content/datasheets/353-DMG80480T050_09WN.pdf
- Reference DWIN_SETs: CR6Community/CR-6-touchscreen, Not1Fun/HT,
  opeensf/project-4-COUNTER-STRIKE-UNIVERSITY-, Thinkersbluff/Klipper-dgus_CR6 (GitHub)
- Waveshare ESP32-C6-LCD-1.47 wiki (pinout): https://www.waveshare.com/wiki/ESP32-C6-LCD-1.47
