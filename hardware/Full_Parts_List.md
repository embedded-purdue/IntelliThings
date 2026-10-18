# IntelliThings — Full Parts List (Specs Reference)
*Last updated Sept 26, 2026*

> Per-part specs for every component in the current design: interface, voltage, output data, key specs, quantity and source. **Spending, order totals and what's still to buy are in `docs/Project_Guideline.md` §7** — this doc is the technical reference. The earlier `IntelliThings_BOM.xlsx` (9-sensor set, ESP32-C5) is **outdated** and shouldn't be used for ordering.

---

## 1. Microcontroller

| Use | Part | Interface / radio | Voltage | Qty | Unit price | Key specs |
|---|---|---|---|---|---|---|
| Dev rigs (+ final-node fallback) | **Espressif ESP32-C6-DevKitC-1-N8** (Adafruit PID 5672) | Two USB-C: USB-UART bridge (GPIO16/17) + native USB/JTAG (GPIO12/13); Wi-Fi 6 2.4 GHz + BLE 5 + 802.15.4 | 5 V via USB, 3.3 V logic | 3 (owned) | $9.95 | RISC-V single core 160 MHz, no PSRAM, 8 MB flash (ESP32-C6-WROOM-1 module), ~23 GPIO exposed / ~12 "safe", onboard RGB LED on GPIO8 |
| Final nodes (plan) | **Bare ESP32-C6 chip** on the custom PCB | — | 3.3 V | 3 + spares | TBD at PCB design | Needs 40 MHz crystal, external SPI flash, RF matching and PCB antenna |
| Final nodes (fallback) | DevKitC-1-N8 plugged into **female headers** (or soldered to male headers) on the PCB | — | — | reuse the 3 dev boards | — | Used if the bare-chip layout isn't ready by Oct 25 |

---

## 2. Sensors

| Measures | Part (dev) | Final-PCB form | Interface / addr | Supply | Output data | Key specs | Qty owned | Unit price |
|---|---|---|---|---|---|---|---|---|
| Temp + humidity | Sensirion **SHT41** — Adafruit PID 5776 | SHT41 chip (SMT) | I2C 0x44 | Board 3–5 V (chip 1.08–3.6 V) | 16-bit temp + RH | ±0.2 °C, ±1.8 %RH | 2 | $5.95 |
| TVOC | Sensirion **SGP40** — Adafruit PID 4829 | SGP40 chip (SMT) | I2C 0x59 | Board 3–5 V (chip 1.7–3.6 V) | Raw signal → **VOC Index 0–500** (uses SHT41 data for compensation) | No ppb / eCO₂ output | 2 | $14.95 |
| CO₂ | Sensirion **SCD41** — Adafruit PID 5190 | SCD41 module (SMT) | I2C 0x62 | Board 3–5 V (chip 2.4–5.5 V) | CO₂ ppm + temp + RH | 400–5000 ppm, ±(40 ppm + 5 %), updates ~every 5 s | 2 | $49.95 |
| Ambient light | **BH1750** — Adafruit PID 4681 | BH1750 chip (SMT) | I2C 0x23 | Board 3–5 V (chip 2.4–3.6 V) | 16-bit lux | ~1–65535 lux | 2 | $4.50 |
| Distance | ST **VL53L0X** ToF — Adafruit PID 3317 | VL53L0X module (SMT) + cover window | I2C 0x29 | Board 3–5 V (chip 2.8 V, regulated on board) | Distance in mm | ~30–1000 mm (≈1.2 m max); 3–12 % ranging accuracy depending on light/surface | 2 | $14.95 |
| PM1.0 / 2.5 / 10 | Plantower **PMS5003** + breadboard adapter kit — Adafruit PID 3686 | Same module on a board connector | UART 9600 baud (+ optional SET pin to sleep the fan) | **5 V** power, 3.3 V logic | Binary frames (`0x42 0x4D`, checksum), µg/m³ | 0–500 µg/m³; fan needs clear inlet/outlet; ~30 s warm-up after sleep | 2 (both move to final nodes; **buy 1 more**) | $39.95 (kit) |
| Human presence | Hi-Link **LD2410C** mmWave — Qoroos 3-pack (Amazon) | Same module on header/connector | GPIO — OUT pin only (UART available but unused) | 5 V (**verify module datasheet** — sources disagree), 3.3 V logic | HIGH/LOW presence | 24 GHz; detects stationary people; sees through thin plastic, not metal | 3 | $20.48 / 3-pack |

All five I2C sensors share one bus (2 pins); default addresses don't conflict.

---

## 3. Display

| Part | Size / res | Interface | Power | Controller | Qty | Unit price |
|---|---|---|---|---|---|---|
| **DWIN DMG80480T050_09WN** | 5.0", 800×480 IPS, 900 nit, **no touch** | UART2 — **TTL/CMOS or RS232, selected on the board (set to TTL!)**; 3150–3225600 baud (typ. 115200), 8N1; 8-pin 2.0 mm socket (power + serial) | **9–36 V, 12 V typ.**; ~210 mA @ 12 V max backlight, ~100 mA backlight off; 12 V / 1 A supply recommended | T5L (DGUS II), 16 MB flash, SD card (FAT32) for UI files; −20 to 70 °C | 3 (dev → final) | CNY 175 |
| 8-pin 2.0 mm display cable combo (SD003) | — | — | — | — | 4 | CNY 26.5 |

---

## 4. LED Bar

| Part | Density / width | Voltage | Control | Look | Qty | Unit price |
|---|---|---|---|---|---|---|
| **BTF-Lighting Ultra Narrow WS2812B RGBIC**, 3535 LEDs | 144 LEDs/m, 7.2 mm | 5 V | Single data line (RMT) | Separate bright dots | 2 × 1 m | CNY 80 |
| **BTF-Lighting FCOB RGB IC** (WS2812B) | 160 pixels/m, 5 mm | 5 V | Single data line (RMT) | Continuous diffused line (COB) | 2 × 1 m | CNY 25 |
| 3-pin SM LED connector pigtails, 15 cm | — | — | — | — | 2 packs × 10 | CNY 9 / pack |

Strip choice and LED count per bar are decided after testing on the rigs. Up to ~60 mA per LED at full white — cap brightness in firmware. Data line needs ≥ ~3.5 V high at 5 V supply → 3.3 V → 5 V level shifter (e.g. 74AHCT1G125) on the final PCB, ~330 Ω series resistor, 470–1000 µF bulk cap.

---

## 5. Power (final nodes — option chosen during PCB design)

| Option | Adapter | On-board conversion |
|---|---|---|
| 12 V in | 12 V wall adapter, barrel jack | 12 V → display; buck 12 V → 5 V; 5 V → 3.3 V |
| 5 V in | 5 V wall adapter, USB-C or barrel jack | Boost 5 V → 12 V for the display; 5 V loads direct; 5 V → 3.3 V |

Rails: **12 V** display · **5 V** PMS5003, LD2410C, LED bar · **3.3 V** ESP32-C6 + I2C sensors. Adapters ×3 still to buy, sized after measuring the rigs.

---

## 6. Dev Supplies

| Item | Qty | Price |
|---|---|---|
| ELEGOO 830-point solderless breadboard, 3-pack | 1 pack (3 boards) | $8.99 |
| IRIS USA 6 Qt storage bins with lids, 4-pack (dev-rig storage) | 1 pack (4 boxes) | $29.99 |

---

## 7. Hub & Cloud Services

| Item | Purpose | Cost |
|---|---|---|
| Raspberry Pi 5 + Home Assistant OS | Hub: Mosquitto broker, entities, automations, MCP Server, dashboard | Already owned |
| Wi-Fi router | Network + internet | Already owned |
| Home Assistant Cloud (Nabu Casa) | Remote access for the cloud agent to HA's MCP Server | $6.50/month or $65/year |
| AWS Lambda | Agent harness | Free tier: 1M requests + 400,000 GB-s/month |
| API Gateway (HTTP API) | `/ha-event` + `/chat` endpoints | ~$1.00 per million requests; 1M/month free for 12 months (new account) |
| OpenRouter | Model testing across providers | Pay-as-you-go credits ($21.19 bought Sept 24) |
| Discord or Telegram bot (team decision) | Chat control | Free |

---

## 8. Final-PCB Parts Still to Select

Chosen during PCB design (schematic by Oct 18, layout by Oct 25, order Nov 1): ESP32-C6 support parts (crystal, SPI flash, RF matching, antenna) or header sockets for the fallback · SMT sensors (SHT41, SGP40, SCD41, BH1750, VL53L0X) · buck or boost converter · 3.3 V regulator · LED level shifter · connectors (display 8-pin 2.0 mm, PMS5003, LD2410C, LED 3-pin, power) · passives · optional JLCPCB stencil.

---

## Sources

- [Adafruit – SHT41 (5776)](https://www.adafruit.com/product/5776) · [SGP40 (4829)](https://www.adafruit.com/product/4829) · [SCD41 (5190)](https://www.adafruit.com/product/5190) · [BH1750 (4681)](https://www.adafruit.com/product/4681) · [VL53L0X (3317)](https://www.adafruit.com/product/3317) · [PMS5003 kit (3686)](https://www.adafruit.com/product/3686) · [ESP32-C6-DevKitC-1-N8 (5672)](https://www.adafruit.com/product/5672)
- [espboards.dev – LD2410](https://www.espboards.dev/sensors/ld2410/)
- [DWIN – DMG80480T050_09W specifications](https://www.dwin-global.com/5-0-inch-hmi-tft-lcd-modeldmg80480t050_09windustrial-product/) · [Evelta – DMG80480T050_09WN](https://evelta.com/5-800x480-non-touch-ttl-232-lcd-display/)
- [BTF-Lighting – Ultra Narrow WS2812B](https://www.btf-lighting.com/products/ultra-narrow-ws2812b-addressable-rgbic-led-strip?variant=46101774827746) · [BTF-Lighting – FCOB addressable strip](https://www.btf-lighting.com/products/fcob-rgb-addressable-led-strip-dc5v-160pixels)
- [Nabu Casa pricing](https://www.nabucasa.com/pricing/) · [AWS Lambda pricing](https://aws.amazon.com/lambda/pricing) · [API Gateway pricing](https://aws.amazon.com/api-gateway/pricing)
