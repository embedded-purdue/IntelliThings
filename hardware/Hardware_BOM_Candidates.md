# IntelliThings — Hardware Design Decisions & Trade-offs
*Hardware subteam (lead @LiamWatson-Purdue) | Last updated Sept 26, 2026*

> The **reasoning** behind the node's hardware choices — why each part was picked and what was considered. The current specs are in `hardware/Full_Parts_List.md`, and spending / build plan / schedule are in `docs/Project_Guideline.md` §3 and §7. If anything here disagrees with the Project Guideline, the guideline wins.

---

## 1. Current Decisions at a Glance

| Area | Decision | Status |
|---|---|---|
| Firmware platform | Rust `std` (nightly) on **ESP-IDF v6.1.0** via `esp-idf-sys` 0.38 / `-hal` 0.47 / `-svc` 0.53 | Confirmed |
| MCU chip | **ESP32-C6** | Confirmed |
| Dev boards | **ESP32-C6-DevKitC-1-N8** ×3 (Adafruit PID 5672) — 2 rigs + 1 spare | Bought |
| Final-node MCU | **Bare ESP32-C6 chip** on the PCB if time allows; **fallback: DevKit plugged into female headers** (or soldered to male headers) | Decided at Oct 25 layout session |
| Sensors | 7 parts: SHT41, SGP40, SCD41, BH1750, VL53L0X (I2C) · PMS5003 (UART) · LD2410C (GPIO OUT pin) | Bought (dev); SMT versions for final PCB |
| Display | **DWIN DMG80480T050_09WN** — 5", 800×480, no touch, UART (TTL mode), DGUS, **12 V** | Bought ×3 |
| Light output | **WS2812B LED bar** — two BTF strips bought to compare; what it shows is a 🗳 team decision | Strip chosen after rig testing |
| Node power | 12 V or 5 V wall adapter; 5 V → 12 V boost if 5 V in | Chosen during PCB design |
| PCB | One board, or split into sensor / ESP / connector-power boards; JLCPCB, ~7 days to DHL delivery (+~1 day with stencil) | Order Nov 1 |
| Protocols | **MQTT only** — Matter rejected | Confirmed |

---

## 2. MCU: why ESP32-C6

**Rust decided the architecture.** Espressif's Rust support splits by CPU:
- **RISC-V chips (C3, C5, C6, C61, H2)** are upstream Rust/LLVM targets — no forked compiler.
- **Xtensa chips (ESP32, S2, S3)** need Espressif's patched toolchain via `espup` — workable, but an extra toolchain to install and maintain.

That ruled out the S3: its strengths (3 UARTs, big PSRAM, dual core, RGB LCD peripheral) mattered less once the node needed only ~2 UARTs, while the Xtensa toolchain was a real added cost.

**Among RISC-V options:**

| Chip | Why / why not |
|---|---|
| **ESP32-C6 — chosen** | Mature Rust-on-ESP-IDF community; most GPIO headroom of the RISC-V options (~12 "safe" pins on the DevKitC-1 vs ~7 on a C3); 802.15.4 radio unused but free |
| ESP32-C3 | Largest Rust tutorial base, cheapest — but tightest GPIO margin |
| ESP32-C5 / C61 | Stable ESP-IDF support only since v6.0 (March 2026) — would stack "new chip" risk on top of the team's first Rust project. C5's 5 GHz Wi-Fi and PSRAM weren't needed |
| ESP32-P4 / H2 / classic ESP32 / S2 | No Wi-Fi (P4, H2), or Xtensa toolchain with no upside (ESP32, S2) |

The C6 has **no RGB-parallel LCD peripheral** — which is why the display is a smart UART panel that renders on its own (§4).

---

## 3. Boards: dev rigs and final nodes

**Dev: ESP32-C6-DevKitC-1-N8.** Two USB-C ports — USB-UART bridge (GPIO16/17) for flashing/console and native USB/JTAG (GPIO12/13) for debugging at the same time. The 8 MB flash leaves room for OTA (`ota_0`/`ota_1`). Onboard RGB LED on GPIO8 (a strapping pin). Considered and not used: Waveshare's pin-compatible clone (was the plan for a board-based fleet before the custom PCB) and the Seeed XIAO ESP32C6 (only ~11 pins).

**Final: bare chip vs. DevKit on headers.**

| | Bare ESP32-C6 chip | DevKit on headers (fallback) |
|---|---|---|
| What's on our PCB | Chip, 40 MHz crystal, SPI flash, RF matching network, PCB antenna | Header footprint matching the DevKitC-1 |
| Pros | Smallest, most "real product"; strong learning outcome | Certified module + antenna already done; no RF design risk; reuses the 3 DevKits |
| Cons | RF/antenna layout and tuning risk; more design time | Bigger; DevKit sticks out; antenna end must clear the board edge and any metal |
| Mounting | — | **Female headers (preferred):** plug-in, removable, easy to swap/reflash. **Male headers, soldered:** sturdier, permanent |

Decision point is the **Oct 25** layout session — switch to the fallback rather than slip the Nov 1 order.

---

## 4. Display: why the DWIN 5"

| Option | Verdict |
|---|---|
| **DWIN DMG80480T050_09WN (5", 800×480, DGUS, 12 V) — chosen** | Bigger, sharper screen for readings + a dedicated AI message section; the display renders everything itself, so the ESP32 only writes values to VP addresses over UART; 3 units bought for dev and reused in the final nodes |
| TJC4827T143_011N (4.3", 480×272, Nextion-style text commands, 5 V) | Earlier baseline — cheaper and simpler protocol, but smaller and lower resolution |
| Raw SPI color LCD (ST7789/ILI9341) | Needs firmware-side rendering (LVGL/`embedded-graphics`), 5 pins; large panels are too slow over SPI |
| SPI e-ink | Holds image unpowered, but slow refresh — poor fit for AI feedback |
| Large RGB-parallel panels | Not possible — the C6 has no RGB LCD peripheral |

**Trade-offs we accepted with the DWIN:**
- **12 V supply** → the node needs a 12 V rail (or a 5 → 12 V boost).
- **DGUS register protocol** is more involved than Nextion/TJC text commands — mitigated by a shared VP address map.
- **UART2 is TTL or RS232, selected on the board** — must be set to **TTL** before wiring to the ESP32.

---

## 5. LED bar

Two BTF-Lighting WS2812B strips (5 V, one data line, driven by the C6's RMT peripheral) were bought to compare:
- **Ultra-narrow 3535, 144/m, 7.2 mm** — individual bright dots.
- **FCOB, 160/m, 5 mm** — continuous diffused line.

Each rig gets one reel of each so the teams can compare look, diffusion, current and fit. Design notes: brightness cap in firmware (~60 mA/LED at full white), 3.3 → 5 V level shifter on the PCB, series resistor, bulk capacitor, power fed to the strip directly. What the bar *shows* (status / AI feedback / air-quality gauge / mix) is a **🗳 team decision**.

---

## 6. Sensors: why these seven

| Sensor | Why this part | Alternatives considered |
|---|---|---|
| **SHT41** temp/RH | Accurate (±0.2 °C, ±1.8 %RH), cheap, I2C; feeds SGP40 humidity compensation | SHT40 (less accurate) |
| **SGP40** TVOC | VOC Index 0–500, more reliable than CCS811; pairs with SHT41 | BME680/688 (gas resistance + pressure in one part — a "fewer parts" option if ppb-level TVOC isn't needed) |
| **SCD41** CO₂ | True NDIR ppm (400–5000) — the standard trigger for ventilation decisions; distinct from SGP40's VOC Index | SCD40 ($5 cheaper, 400–2000 ppm range) |
| **BH1750** light | Lux lets the agent reason about "is it dark" instead of time of day | — |
| **VL53L0X** distance | I2C ToF, rides the shared bus, precise; ~3 cm–1.2 m (desk-range, not room-range) | HC-SR04 ultrasonic (2 GPIO, bulkier, ~4 m range) |
| **PMS5003** PM1.0/2.5/10 | Industry standard, direct µg/m³ output | — |
| **LD2410C** presence | mmWave detects **stationary** people — PIR can't ("cool the room only if someone's there") | ST STHS34PF80 I2C presence sensor |

**Dropped from the earlier 9-sensor draft:** LTR390 (UV) and BMP280 (pressure) — not useful enough for indoor decisions.

**LD2410C wiring — Option A (chosen): OUT pin only.** VCC/GND/OUT gives a boolean presence signal on one GPIO and uses no UART. Option B (full UART: distance, moving vs. stationary, per-zone energy) would need a third UART with the DWIN display — revisit only if richer presence data becomes important.

---

## 7. Pin & peripheral budget

| Block | Pins | Peripheral |
|---|---|---|
| 5 × I2C sensors | 2 (SDA, SCL) — addresses 0x44, 0x59, 0x62, 0x23, 0x29 don't conflict | 1 I2C bus |
| PMS5003 | 1–2 (RX + optional SET pin) | UART0 |
| LD2410C OUT | 1 | GPIO |
| DWIN display | 2 (TX, RX) | UART1 |
| WS2812B LED bar | 1 (data) | RMT |
| **Total** | **7–8 pins, 2 UARTs** | fits the C6's ~12 safe pins |

Avoid on the DevKitC-1: GPIO12/13 (native USB), GPIO16/17 (UART bridge); strapping pins GPIO4, 5, 8, 9, 15 need care — don't put the LED data or LD2410C OUT on them. ESP-IDF defaults the console to the UART0 bridge; switching console to USB Serial/JTAG is a menuconfig option.

---

## 8. Power

| Load | Voltage | Notes |
|---|---|---|
| DWIN display | **12 V** (9–36 V) | ~210 mA max backlight; ~0.5–0.6 A from 5 V if boosted |
| PMS5003 | 5 V power, 3.3 V logic | No level shifter on data lines |
| LD2410C | 5 V (**verify the purchased module's datasheet** — sources disagree, some say 3.3 V only), 3.3 V logic | Check before first power-on |
| WS2812B LED bar | 5 V | Dominates the 5 V budget (e.g. 30 LEDs full white ≈ 1.8 A); needs a data-line level shifter |
| ESP32-C6 + I2C sensors | 3.3 V | Adafruit breakouts accept 3–5 V |

**Input options (picked during PCB design):** 12 V adapter + buck to 5 V, or 5 V adapter (USB-C or barrel jack) + boost to 12 V for the display. A 5 V / 3 A USB-C adapter gives 15 W — check it covers the LED bar + display boost. **Common ground** everywhere. Measure real current on the rigs before finalizing.

---

## 9. PCB structure

**Option: split into several PCBs** (ordered together at JLCPCB — no extra production time):
- **Sensor PCB** — the five I2C sensors near vents/windows, away from ESP32 and converter heat (better temperature/CO₂ readings).
- **ESP PCB** — bare chip, or header sockets for the DevKit fallback (the chip decision only affects this board).
- **Connector / power PCB** — power input and conversion, LED level shifter, connectors for the display, PMS5003, LD2410C and LED bar.

Each board is its own KiCad project; board-to-board connector pinouts become shared interfaces. **Placement rules:** SHT41/SCD41 away from heat; PMS5003 airflow path; VL53L0X and BH1750 windows; no metal or ground pour in front of the LD2410C; antenna keep-out for the bare chip or DevKit.

**Ordering:** JLCPCB order → DHL delivery ~7 days; a stencil for SMT soldering adds ~1 day. Nov 1 order → boards by the Nov 8 session, with room for one re-spin before Dec 6.

---

## 10. Rejected: Matter (historical)

Matter was listed as a target skill in the proposal and is technically possible on the C6 (`esp-matter` supports it, and Matter has air-quality clusters). It was **dropped entirely** because:
- `esp-matter` is a separate C/C++ SDK with no first-class Rust bindings — it would mean FFI or C code, against the Rust decision.
- Added concepts (commissioning, fabrics, attestation) on top of an already full semester.
- Air-quality support is still rough (open `esp-matter` issue; HA TVOC reporting gap), and the distance sensor has no standard cluster.
- Matter's value (cross-ecosystem interop without a shared hub) doesn't apply — this design is single-hub (Home Assistant) with MQTT end to end.

---

## Sources

- [Espressif – Rust on ESP](https://docs.espressif.com/projects/rust/) · [The Rust on ESP Book](https://docs.espressif.com/projects/rust/book/) · [esp-rs/esp-idf](https://github.com/esp-rs/esp-idf)
- [Espressif – ESP32-C5 product page](https://www.espressif.com/en/products/socs/esp32-c5) · [espboards.dev – ESP32 SoC options](https://www.espboards.dev/blog/esp32-soc-options/)
- [DWIN – DMG80480T050_09W specifications](https://www.dwin-global.com/5-0-inch-hmi-tft-lcd-modeldmg80480t050_09windustrial-product/) · [TJC 4.3" HMI (techonicsltd.com)](https://www.techonicsltd.com/tjc-4-3-inch-hmi-lcd-display-basic-series-hmi-without-touch/)
- [BTF-Lighting – Ultra Narrow WS2812B](https://www.btf-lighting.com/products/ultra-narrow-ws2812b-addressable-rgbic-led-strip?variant=46101774827746) · [BTF-Lighting – FCOB strip](https://www.btf-lighting.com/products/fcob-rgb-addressable-led-strip-dc5v-160pixels)
- [Adafruit – ESP32-C6-DevKitC-1-N8 (5672)](https://www.adafruit.com/product/5672) · [PMS5003 kit (3686)](https://www.adafruit.com/product/3686) · [SCD40 (5187)](https://www.adafruit.com/product/5187) · [espboards.dev – LD2410](https://www.espboards.dev/sensors/ld2410/)
