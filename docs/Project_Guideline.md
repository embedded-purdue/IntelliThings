# IntelliThings — Project Guideline
*ES@P | Fall '26 | Last updated Sept 26, 2026*

> The single reference for what we're building, what we've bought, and who owns what. It pulls together every decision made since the May 2026 proposal. Deeper design reasoning lives in the companion docs listed at the end; if anything there disagrees with this guideline, this guideline is current.
>
> Items marked **🗳 TEAM DECISION** are open and will be decided by the whole team at a work session — the options are listed where they come up, and all of them are collected in §10.

---

## 0. At a Glance — What Changed Since the Proposal

| Area | Proposal (May 2026) | Current decision |
|---|---|---|
| Microcontroller | ESP32-C5 | **ESP32-C6** — DevKitC-1-N8 for dev; bare ESP32-C6 chip on a custom SMT PCB for the final nodes if time allows, otherwise the DevKit plugged into headers on the PCB |
| Firmware | Embedded C + FreeRTOS, ESP-IDF | **Rust ("std") on ESP-IDF v6.1.0** via `esp-idf-sys` 0.38 / `esp-idf-hal` 0.47 / `esp-idf-svc` 0.53 (FreeRTOS still runs underneath); **nightly Rust**, exact nightly date pinned after the first successful build |
| Node protocols | MQTT + Matter | **MQTT only** — Matter dropped |
| Sensors | Temp, humidity, PM2.5/PM10, TVOC, PIR, ultrasonic | **7 parts:** SHT41, SGP40, SCD41, BH1750, VL53L0X, PMS5003, LD2410C (adds CO₂ + light; ToF replaces ultrasonic; mmWave replaces PIR) |
| Display | 7" color screen | **DWIN DMG80480T050_09WN** — 5", 800×480 IPS, no touch, UART, **12 V** |
| Light output | — | **WS2812B addressable LED bar** (strip type chosen after testing) |
| Node power | Not specified | 12 V or 5 V wall adapter (USB-C or barrel jack); 5 V input needs a 5 V → 12 V boost for the display |
| LLM | OpenAI GPT-4.1 called from the Pi | **Custom agent harness on AWS Lambda + API Gateway**; models tested through **OpenRouter** first, final model chosen from the tests |
| How the AI acts | MQTT / Matter commands | **Home Assistant MCP Server** — agent calls HA as tools |
| User interfaces | Display; optional buttons / mic / speaker | Display + LED bar on the node + a **chat bot** — **Discord or Telegram (🗳 TEAM DECISION)** |
| Build plan | Not specified | **Emulator ESP32** (3 simulated nodes, for early AI testing) + **2 identical breadboard dev rigs** → **3 final sensor nodes** (SMT PCB + 3D-printed enclosure) |
| Team structure | Software, Hardware, LLM, Smart Home roles | **3 subteams: Hardware, Software, AI** with confirmed leads (§8); the AI subteam is overseen by a Local AI lead (undecided) and a Cloud AI lead |
| Schedule | 13 weekly steps | **11 Sunday work sessions**, Sept 6 – Dec 6 (§9) |

---

## 1. Project Vision & Motivation

**From IoT to AIoT.** Classic smart homes stop at IoT: sensors report data, but a person still has to hand-write "if this, then that" automations for every situation. IntelliThings closes the loop — **sensors → AI → action** — so an LLM agent reasons over live sensor data and decides what to do, adapting to how a household actually lives instead of running a fixed rulebook.

*Example:* the room is warmer than comfortable **and** someone is present → the agent decides to cool the room → Home Assistant turns on the fan/thermostat → the node's display shows "Room was 28 °C, turned on the fan."

**Similar product — Xiaomi Miloco 2.0** (open-sourced June 2026; JARVIS-like whole-home AI with memory):
- Senses mainly through **Mi Home camera video/audio**.
- Needs a **dedicated local compute unit** running 24/7 (Xiaomi recommends ≥4 GB RAM / ≥256 GB storage) *and* calls Xiaomi's cloud model (MiMo) — a hybrid architecture.
- Only works with devices bound to a **Xiaomi Mi Home account**.
- Always-on cameras plus cloud video processing raise **privacy** concerns and add deployment complexity.

**Where IntelliThings differs:** low-cost, privacy-preserving **environmental sensors** instead of cameras; no local compute box (reasoning runs in a **cloud agent**); open protocols end to end (**MQTT, Home Assistant, MCP**) instead of a single-vendor ecosystem. Miloco wins on perception richness (vision) and maturity (shipped product) — we trade those for cost, privacy and openness.

**Showcase target:** ECE SPARK Challenge.

---

## 2. System Architecture

### 2.1 Block diagram

```
┌──────────── Desktop Sensor Node (×3 final, ×2 dev rigs) ─────────┐
│ 12 V in (or 5 V in + 5→12 V boost)                               │
│   ├─► DWIN 5" display (12 V)                                     │
│   └─► 5 V ─┬─► PMS5003 · LD2410C · WS2812B LED bar               │
│            └─► 3.3 V ─► ESP32-C6 · I2C sensors                   │
│                                                                  │
│ ESP32-C6 (Rust + ESP-IDF)                                        │
│  I2C bus: SHT41 · SGP40 · SCD41 · BH1750 · VL53L0X               │
│  UART0:   PMS5003 (PM1.0/2.5/10)                                 │
│  UART1:   DWIN DMG80480T050_09WN display (TTL mode, DGUS)        │
│  GPIO:    LD2410C OUT (presence)                                 │
│  RMT:     WS2812B LED bar (via 5 V level shifter)                │
└──────────────┬─────────────────────────────▲─────────────────────┘
   publish     │ Wi-Fi / MQTT                │ subscribe
   .../sensors │                             │ .../display
┌──────────────▼─────────────────────────────┴───────┐
│ Raspberry Pi 5 — Home Assistant OS                  │
│  Mosquitto MQTT broker (add-on)                     │
│  MQTT sensor entities · virtual devices · dashboard │
│  Automation ──(HTTPS POST: state snapshot)──────────┼──┐
│  MCP Server integration ◄──(MCP tool calls)─────────┼──┼──┐
└──────────────▲──────────────────────────────────────┘  │  │
               │ Nabu Casa remote access (HA Cloud)      │  │
┌──────────────┴──────────────────────────────────────┐  │  │
│ AWS: API Gateway (HTTP API) → Lambda agent harness  │◄─┘  │
│   MCP client ─────────────────────────────────────────────┘
│   LLM via OpenRouter (testing) → final provider     │
│   Chat webhook ◄──► Discord or Telegram bot         │
└─────────────────────────────────────────────────────┘
```

### 2.2 Components

1. **Sensor nodes** — ESP32-C6 + 7 sensors + 5" DWIN display + WS2812B LED bar. Two breadboard rigs during development, three finished nodes on custom PCBs in 3D-printed enclosures for the final product. Room placement: **🗳 TEAM DECISION** (§10). Early on, an **emulator ESP32** stands in for all three nodes with simulated data (§3.1a).
2. **Wi-Fi router** — home network and internet uplink (already owned).
3. **Raspberry Pi 5 running Home Assistant OS** (already owned) — hosts the **Mosquitto MQTT broker add-on**, turns MQTT payloads into HA entities, runs automations, hosts virtual devices and the dashboard.
4. **Hand-off to the agent** — an HA automation sends a state snapshot to the cloud agent's HTTPS endpoint (e.g. via `rest_command`) when something relevant changes. HA's *AI Task* integration only runs against LLM integrations configured inside HA, so it can't target our Lambda harness; it stays useful for quick in-HA experiments.
5. **Cloud AI agent** — a small custom harness on **AWS Lambda behind API Gateway (HTTP API)**. It calls the LLM with the snapshot, a system prompt and the tools exposed by HA.
6. **Action via MCP** — the LLM *decides*; MCP is *how* the decision is executed. The harness acts as an **MCP client** to **HA's Model Context Protocol Server** integration: it lists HA's tools, hands them to the model, and runs the tool calls the model makes. HA's entity-exposure settings limit exactly which devices/services the agent can touch.
7. **Real and virtual devices** — HA service calls hit real devices where we have them, and **virtual devices** (template/helper entities) for thermostat, lights, fan, purifier, humidifier, etc. in the prototype.
8. **Feedback to the node** — HA publishes the AI's message to the node's `.../display` topic. The screen always shows all environmental readings, plus a **dedicated AI feedback section** that keeps showing the latest AI message until a new one arrives. The LED bar's role is a **🗳 TEAM DECISION** (§3.5).
9. **Chat control** — a **chat bot** (Discord or Telegram — **🗳 TEAM DECISION**, §6) sends commands such as "turn off all the lights" to a second API Gateway route on the *same* Lambda, which runs the same LLM + MCP logic and replies in the chat.
10. **Dashboard** — real-time HA dashboard including a 3D/floor-plan view. HA has no native 3D floor plan, so this needs a community Lovelace card or a custom one.

### 2.3 Flows

**Autonomous (sensor-driven):**
`sensor → ESP32-C6 → MQTT (.../sensors) → Mosquitto → HA entities → HA automation → API Gateway → Lambda harness → LLM decides → MCP call → HA MCP Server → HA service call → real/virtual device → dashboard update + message to node display`

**Chat-driven:**
`Chat message or command (Discord or Telegram) → API Gateway → Lambda → LLM → MCP call → HA → device → reply in the chat` (Discord adds a 3-second acknowledge step — §6)

---

## 3. End-Node Hardware

### 3.1 Build plan: dev rigs → final nodes

| Phase | What | Parts |
|---|---|---|
| **Development** | **Two identical breadboard rigs** — **Rig H** for the Hardware subteam, **Rig S** for the Software subteam. Identical wiring so code and measurements transfer 1:1. Stored in project boxes between sessions. | Dev-only: Adafruit breakouts, ESP32-C6-DevKitC-1-N8, breadboards. Shared: displays, LD2410C, PMS5003, LED strip, connectors |
| **Final product** | **Three sensor nodes** — custom PCB with **SMT parts**, each in a **3D-printed enclosure**. | SMT sensors + ESP32-C6 on the PCB (or the DevKit plugged into headers); PMS5003 and LD2410C as modules; displays, LED strip, connectors, LD2410C and both PMS5003s reused from dev |

**Unit allocation:**

| Part | Qty owned | Rig H | Rig S | Spare | Final nodes |
|---|---|---|---|---|---|
| ESP32-C6-DevKitC-1-N8 | 3 | 1 | 1 | 1 | Only if the DevKit-on-headers fallback is used (all 3 move into the final nodes) |
| SHT41, SGP40, SCD41, BH1750, VL53L0X breakouts | 2 each | 1 | 1 | — | — (replaced by SMT parts) |
| PMS5003 kit | 2 | 1 | 1 | — | Both move to final nodes; **buy 1 more** for node 3 |
| LD2410C | 3 | 1 | 1 | 1 | All 3 move to final nodes |
| DWIN 5" display | 3 | 1 | 1 | 1 | All 3 move to final nodes |
| Display cable combo (8-pin 2.0 mm) | 4 | 1 | 1 | 2 | 3 used, 1 spare |
| WS2812B 3535 144/m strip (1 m) | 2 | 1 | 1 | — | Cut into bars if chosen |
| WS2812B FCOB 160/m strip (1 m) | 2 | 1 | 1 | — | Cut into bars if chosen |
| LED pigtail connectors (3-pin SM, 15 cm) | 20 | as needed | as needed | — | 1+ per node |
| ELEGOO 830-pt breadboard | 3 | 1 | 1 | 1 | — (dev only) |
| IRIS 6 Qt storage box | 4 | 1 | 1 | 2 (parts) | — (dev storage) |

*Each rig gets one reel of each strip type so the strips can be compared side by side.*

### 3.1a Emulator ESP32 — early AI testing (AI subteam; owner: @spicybutter, PM)

So the AI subteam can build and test the Home Assistant + cloud AI pipeline **before the Software subteam's firmware is ready**, one spare ESP32 board from @spicybutter's own stock runs as a **sensor-data emulator**. It has no sensors attached. The emulator belongs to the **AI subteam** (code in `ai/emulator/`); @spicybutter (PM) builds and maintains it.

- **Board:** a LuatOS ESP32-C3 Core, running C on ESP-IDF v6.1 (the emulator shares no code with the node firmware, only the MQTT contract). Full guide: `ai/emulator/README.md`.
- **Three emulated nodes on one board:** it publishes as `emu-kitchen`, `emu-bedroom` and `emu-living-room`, each with its own topics and its own MQTT connection and Last Will, so HA sees three independent devices — the same count as the final product.
- **Publish interval:** every 3 s, even when values haven't changed (emulator only; the real-node interval is still open).
- **Every reading in the system:** each emulated node publishes the full sensor set from §3.3 — temperature, humidity, VOC Index, CO₂, lux, distance, PM1.0/PM2.5/PM10 and presence — as fake but plausible values.
- **Same MQTT contract as the real nodes (§4):** same topic pattern, same JSON fields and units, plus `.../status` (online/offline) — so nothing in HA or the agent changes when real nodes replace the emulated ones. It also subscribes to each emulated node's `.../display` topic and prints the AI messages over serial, which tests the feedback path end to end.
- **Useful test behavior:** values drift slowly like a real room, with scripted events the agent should react to (e.g. CO₂ climbing, a hot room with presence, a PM2.5 spike, presence turning off).
- **Control panel:** the emulator serves a web page (`http://intellithings-emu.local/`) where the AI team triggers those events with one click, forces any value into a range, sees system status and reads the AI messages each node received. The same commands work over MQTT on `intellithings/emulator/cmd` (a reserved namespace real nodes never use).
- **Schema coordination:** until the MQTT schema is fixed on Oct 4, the emulator's JSON is the working draft (`docs/interfaces/mqtt.md`); the Software subteam's firmware and the AI subteam's emulator both follow the agreed schema after that.
- **Retirement:** once the rigs publish real data, the emulated nodes are disabled in HA (the emulator stays available for testing the agent and dashboard without hardware).

### 3.2 Microcontroller

**Chip: ESP32-C6** — Wi-Fi 6 (2.4 GHz) + BLE 5 + 802.15.4 (unused), single RISC-V core @160 MHz, 30 GPIO nominal, 2 full UARTs + 1 LP-UART, RMT peripheral, no PSRAM.

Why C6: RISC-V is an official upstream Rust target (no forked Xtensa toolchain like S3), it has a mature Rust-on-ESP-IDF community, and it has enough GPIO headroom (~12 "safe" pins on the DevKitC-1) for this node. C5/C61 were set aside as newer chips (stable ESP-IDF support only since v6.0, March 2026), which would stack new-chip risk on top of the team's first Rust project.

- **Dev:** Espressif **ESP32-C6-DevKitC-1-N8** (Adafruit PID 5672) ×3. Two USB-C ports: USB-UART bridge (GPIO16/17) + native USB/JTAG (GPIO12/13) — flash and debug at the same time. 8 MB flash leaves room for OTA.
- **Final — plan: bare ESP32-C6 chip if time allows.** A bare chip means designing the 40 MHz crystal, external SPI flash, RF matching network and PCB antenna (following Espressif's hardware design guidelines and reference layout) and tuning the antenna.
- **Final — fallback: plug the ESP32-C6 dev board into the PCB.** The PCB gets headers matching the DevKitC-1's pin layout, and the dev board (with its certified WROOM-1 module, antenna, USB and regulator) plugs in:
  - **Female headers (preferred):** the DevKit plugs in and can be pulled out — easy to swap, reflash or reuse. More flexible.
  - **Male headers, soldered:** the DevKit is pushed onto male pins and soldered down — sturdier, but permanent.
  - Switch to this fallback if the bare-chip layout isn't ready by the PCB layout session (Oct 25). The three DevKits from the dev rigs can move into the final nodes when the rigs are retired. Keep the DevKit's antenna end past the board edge, clear of copper and metal.

### 3.3 Sensors

| Measures | Dev part (bought) | Interface / addr | Supply | Output | Final-PCB form |
|---|---|---|---|---|---|
| Temp + humidity | Adafruit **SHT41** (PID 5776) | I2C 0x44 | 3–5 V | °C, %RH (±0.2 °C, ±1.8 %RH) | SHT41 chip (SMT) |
| TVOC | Adafruit **SGP40** (PID 4829) | I2C 0x59 | 3–5 V | VOC Index 0–500 (uses SHT41 data for compensation) | SGP40 chip (SMT) |
| CO₂ | Adafruit **SCD41** (PID 5190) | I2C 0x62 | 3–5 V | CO₂ 400–5000 ppm (+ temp/RH) | SCD41 module (SMT) |
| Ambient light | Adafruit **BH1750** (PID 4681) | I2C 0x23 | 3–5 V | lux | BH1750 chip (SMT) |
| Distance | Adafruit **VL53L0X** ToF (PID 3317) | I2C 0x29 | 3–5 V | mm, ~30–1000 mm (≈1.2 m max) | VL53L0X module (SMT) + cover window |
| PM1.0 / 2.5 / 10 | **PMS5003** + breadboard adapter kit (Adafruit PID 3686) | UART 9600 baud | **5 V**, 3.3 V logic | Binary frames, µg/m³ | Same module (fan unit) on a board connector |
| Human presence | Hi-Link **LD2410C** mmWave (Qoroos 3-pack) | GPIO (OUT pin) | 5 V (check module datasheet), 3.3 V logic | HIGH/LOW presence | Same module on header/connector |

- All five I2C parts share **one bus (2 pins)**; default addresses don't conflict.
- **LD2410C wiring:** VCC/GND/OUT only (boolean presence, no UART used). Its UART output (distance, moving vs. stationary) is available later if a UART frees up.
- **VL53L0X range:** ~3 cm–1.2 m — good for "someone at the desk / something in front of the unit," not for measuring across a room.
- **Before first power-on:** confirm the LD2410C's supply voltage from the Qoroos listing/datasheet — sources disagree between modules.

### 3.4 Display — DWIN DMG80480T050_09WN

| Spec | Value |
|---|---|
| Size / resolution | 5.0", 800×480, IPS, 900 nit, no touch |
| Controller | DWIN T5L (DGUS II), 16 MB flash, SD card (FAT32) for loading UI files |
| Power | **9–36 V, 12 V typical**; ~210 mA @ 12 V at max backlight, ~100 mA backlight off; DWIN recommends a 12 V / 1 A supply |
| Interface | UART2, **TTL/CMOS or RS232 — selected on the board**; baud 3150–3225600 (typ. 115200), 8N1 |
| Connector | 8-pin 2.0 mm socket (power + serial) — using the SD003 8-pin 2.0 mm cable combos |
| Operating temp | −20 to 70 °C |
| Qty | 3 — used on the dev rigs, then moved into the three final nodes |

- **⚠ Set UART2 to TTL mode before wiring it to the ESP32.** In RS232 mode the display drives RS232 voltage levels, which the ESP32's 3.3 V pins can't take.
- **Working model:** screens are designed in DWIN's **DGUS** tool and loaded onto the display from an SD card. The ESP32 then only writes values into display variables ("VP addresses") with short UART frames (`5A A5 <len> 82 <VP addr> <data>`), so all the graphics work happens on the display itself.
- **Screen layout rule (dev and final):** all environmental readings are **always on screen**, and a **dedicated AI feedback section** shows the latest AI message. That message **stays until a new one replaces it** — no timeout.
- **GUI plan (Software subteam):**
  - **Dev GUI** — simple: sensor readouts, connection status and the AI feedback box. Just enough to test the driver and the VP address map.
  - **Final GUI** — designed properly for the product: layout, icons/gauges, AI section styling (with the message's time), and matching the enclosure's bezel.
- Verify the 8-pin pinout against the DWIN datasheet before connecting — it carries both 12 V and serial lines.

### 3.5 LED bar — WS2812B

Two BTF-Lighting WS2812B strips were bought to compare. Both are 5 V, individually addressable, and use a single data line. **The strip for the final bar and the LED count per bar are chosen after testing on the rigs.**

| | Ultra-narrow 3535 | FCOB |
|---|---|---|
| Product | BTF-Lighting Ultra Narrow WS2812B RGBIC, 3535 LEDs | BTF-Lighting FCOB RGB IC, WS2812B |
| Density / width | 144 LEDs/m, 7.2 mm | 160 pixels/m, 5 mm |
| Look | Separate bright dots | Continuous, diffused line (COB) |
| Qty | 2 × 1 m | 2 × 1 m |

**🗳 TEAM DECISION — what the LED bar shows:**

| Option | Behavior |
|---|---|
| A. Status only | Wi-Fi / MQTT / sensor-fault states (replaces the DevKit's onboard LED) |
| B. AI feedback | Lights up or animates when the agent acts or sends a message |
| C. Air-quality gauge | Color/length shows a reading (e.g. CO₂, PM2.5 or overall air quality) |
| D. Mix | e.g. air-quality gauge by default, with status alerts and AI-action animations layered on top |

- **Connectors:** 3-pin SM pigtails, 15 cm, for quick connect/disconnect on the rigs and in the final nodes.
- **Driver:** ESP32-C6 **RMT** peripheral (e.g. `ws2812-esp32-rmt-driver` / `smart-leds` in Rust), one GPIO.
- **Signal level:** WS2812B powered at 5 V expects ≥ 0.7 × VDD (≈3.5 V) for a logic high, and the ESP32 outputs 3.3 V. It often works on a short wire, but the **final PCB includes a 3.3 V → 5 V level shifter** (e.g. 74AHCT1G125) on the data line. On the dev rigs, try direct drive first and add a shifter if the strip flickers.
- **Good practice:** ~330 Ω series resistor on the data line, a bulk capacitor (~470–1000 µF) across the strip's 5 V/GND, and power fed to the strip directly, not through the ESP32 board.
- **Current:** up to ~60 mA per LED at full white. Cap brightness in firmware and size the 5 V rail from the final LED count.

### 3.6 Pin & peripheral budget

| Block | Pins | Peripheral |
|---|---|---|
| 5 × I2C sensors | 2 (SDA, SCL) | 1 I2C bus |
| PMS5003 | 1–2 (RX, + optional SET pin to sleep the fan) | UART0 |
| LD2410C OUT | 1 | GPIO |
| DWIN display | 2 (TX, RX) | UART1 |
| WS2812B LED bar | 1 (data) | RMT |
| **Total** | **7–8 pins, 2 UARTs** | fits in the C6's ~12 safe pins |

Reserved on the DevKitC-1: GPIO12/13 (native USB), GPIO16/17 (UART bridge); strapping pins GPIO4, 5, 8, 9, 15 need care — don't put the LED data or LD2410C OUT on them.

### 3.7 Power

The display needs 12 V; everything else runs from 5 V or 3.3 V.

**Input options (not final — picked during PCB design):**

| Option | Adapter | Connector | On-board conversion |
|---|---|---|---|
| **12 V in** | 12 V wall adapter | Barrel jack | 12 V → display directly; **buck 12 V → 5 V**; 5 V → 3.3 V |
| **5 V in** | 5 V wall adapter | USB-C or barrel jack | **boost 5 V → 12 V** for the display; 5 V → loads directly; 5 V → 3.3 V |

| Rail | Loads |
|---|---|
| **12 V** | DWIN display (~0.2 A max at 12 V; ~0.5–0.6 A drawn from 5 V if boosted) |
| **5 V** | PMS5003, LD2410C, WS2812B LED bar |
| **3.3 V** | ESP32-C6, the five I2C sensors (on the dev rigs, the DevKit's own regulator) |

- **The LED bar dominates the 5 V budget** (e.g. 30 LEDs at full white ≈ 1.8 A). Brightness caps in firmware keep real draw far below that. With 5 V input, check the adapter can cover the LED bar + display boost + everything else (a 5 V / 3 A USB-C adapter gives 15 W).
- **Common ground** between the supply, all rails, the display and the ESP32 — the UART and LED data lines depend on it.
- No level shifters needed for PMS5003 / LD2410C (3.3 V logic); one needed for the LED data line (§3.5).
- **Dev rigs:** 12 V adapter for the display + a 5 V supply/buck for the PMS5003, LD2410C and LED strip; the DevKit can stay on USB as long as grounds are tied together.
- Measure actual current on the rigs before finalizing the PCB power stage.

### 3.8 Final PCB & enclosure

- **PCB (×3 nodes + spares):** ESP32-C6 (§3.2), the five I2C sensors as SMT parts, power input and conversion (§3.7), LED level shifter, and connectors for the display (8-pin 2.0 mm), PMS5003, LD2410C and LED bar (3-pin). The breakout boards on the dev rigs are the reference circuits — copy their pull-ups, decoupling and regulators.
- **Option — split into several PCBs** (ordered together, no extra production time):

| Board | Contents | Why separate |
|---|---|---|
| **Sensor PCB** | SHT41, SGP40, SCD41, BH1750, VL53L0X | Sits near the vents and windows, away from heat from the ESP32 and power parts — better temperature/CO₂ readings |
| **ESP PCB** | ESP32-C6 (bare chip) — or header sockets for the plug-in DevKit as fallback | The bare-chip vs. DevKit choice only affects this board |
| **Connector / power PCB** | Power input and conversion, LED level shifter, connectors for the display, PMS5003, LD2410C, LED bar | Heavy connectors and power parts stay off the sensor board |

  Boards connect with board-to-board headers or short cables (e.g. JST); the I2C bus and power run between them.
- **Placement rules:**
  - Keep **SHT41 and SCD41 away from heat** (ESP32, power converters, display backlight) or temperature/CO₂ will read high — put them at the board edge near a vent, with a slot or cutout for thermal isolation.
  - **PMS5003 needs a clear air inlet and outlet** in the enclosure.
  - **VL53L0X needs an unobstructed window** (IR-transparent cover or open hole).
  - **BH1750 needs a light window** facing the room.
  - **LD2410C** radar sees through thin plastic but not metal — no metal or ground pour in front of it.
  - Bare-chip ESP32-C6: keep the antenna area clear of copper, components and enclosure metal.
- **Enclosure (×3):** 3D-printed, designed around the PCB(s), display bezel, LED bar slot, PMS5003 airflow path and sensor windows.
- **Ordering (JLCPCB):** from order placed to delivery by DHL air is usually **~7 days**. A **stencil** for SMT soldering can be added if needed — it may add **~1 day**. Splitting into several PCBs doesn't add production time.

---

## 4. Firmware (ESP32-C6, Rust + ESP-IDF) — Software subteam

Full design: `software/Firmware_Architecture.md`. Summary:

- **Toolchain (confirmed Sept 26 against the [esp-rs/esp-idf](https://github.com/esp-rs/esp-idf) repo):**
  - **ESP-IDF v6.1.0** — set `ESP_IDF_VERSION = "v6.1"` (ESP-IDF's tag for 6.1.0; the esp-rs CI uses the same value).
  - **Rust crates:** `esp-idf-sys` **0.38.1**, `esp-idf-hal` **0.47.0**, `esp-idf-svc` **0.53.0** — the first releases with ESP-IDF 6.1 compatibility fixes; the esp-rs CI builds the ESP32-C6 target (`riscv32imac-esp-espidf`) against v6.1 in its nightly CI runs.
  - **Rust: nightly** (with `rust-src`), because `std` on this target is built from source with `-Zbuild-std=std,panic_abort`. Minimum Rust 1.82. The exact nightly date is pinned in `rust-toolchain.toml` after the first successful build on the rigs (Sep 27).
  - **MQTT is no longer built into ESP-IDF v6** — add it as a managed component in `Cargo.toml` (`[[package.metadata.esp-idf-sys.extra_components]]` → `remote_component = { name = "espressif/mqtt", version = "1.*" }`). The esp-rs CI tests this on v6.0 / ESP32-C3 only, so verify it on the C6 rig.
  - Third-party crates (WS2812 RMT driver, sensor drivers) must be checked against `esp-idf-hal` 0.47 / `embedded-hal` 1.0. `esp-idf-hal`'s timer drivers aren't available on ESP-IDF 6+ (not needed by this design).
- **Runtime model:** `std::thread` maps to FreeRTOS tasks; `Arc<Mutex<…>>` / `mpsc` cover shared state. Sensor crates: `sht4x`, `sgp40`, `vl53l0x`, `bh1750`, `scd4x` (check each against the `embedded-hal` version `esp-idf-hal` uses; hand-rolled register reads are the fallback). `serde_json` for payloads.
- **Threads:** `main` (boot + supervisor/reconnect), `sensor_i2c` (2–5 s), `pms5003_uart` (10–30 s), `presence_gpio` (edge/poll), `mqtt_publish` (interval or on-change), `dwin_display` (0.5–1 s), `led_bar` (animation tick, ~30–50 ms); MQTT receive handled in a callback.
- **Display driver:** writes sensor values and the AI message to DGUS VP addresses over UART1 at 115200 baud, using the VP address map from the GUI design. Sensor fields refresh continuously; the AI section is only rewritten when a new message arrives.
- **LED bar driver:** RMT-based WS2812B output with a global brightness cap; patterns follow the team's LED bar decision (§3.5).
- **Shared state:** one `Arc<SharedState>` holding the latest `SensorSnapshot`, `SystemStatus` and the latest AI message.
- **MQTT topics** (`<node_id>` derived from the chip's MAC, so one firmware image fits every node):

| Topic | Direction | Content |
|---|---|---|
| `intellithings/<node_id>/sensors` | publish | One JSON snapshot of all readings |
| `intellithings/<node_id>/status` | publish (retained, LWT) | `online` / `offline` |
| `intellithings/<node_id>/display` | subscribe (HA publishes **retained**) | `{"text": "...", "ts": "2026-10-18T14:05:00-04:00"}` — latest AI message; retained so a node that reboots re-shows it |

- **Provisioning:** Wi-Fi credentials and broker address in NVS, not hardcoded.
- **Robustness:** tolerate N consecutive sensor failures before flagging a fault; supervisor loop drives Wi-Fi/MQTT reconnect.
- **Code layout:** `main.rs` plus four module groups — `hardware/` (bus setup, one file per sensor), `network/` (Wi-Fi, MQTT), `ui/` (DWIN display, LED bar), `system/` (state, NVS config, logging, errors). Modules talk through the shared system state. A **baseline firmware** (boot → Wi-Fi → MQTT → placeholder snapshot) is built together first, then the groups work in parallel — see `docs/Collaboration_Guidelines.md` §2.

---

## 5. Home Assistant Hub (Raspberry Pi 5) — AI subteam

- **HAOS** on the Pi 5 with the **Mosquitto broker add-on**.
- **MQTT entities** built from each node's `.../sensors` JSON topic (one entity per field), with `.../status` as availability so a dropped node shows as unavailable immediately. Three final nodes → three devices in HA. The emulator's three nodes (§3.1a) are the first devices set up; they're disabled once real nodes come online.
- **Virtual devices** for everything we don't physically own (thermostat, lights, fan, purifier, humidifier, dehumidifier, smart plug).
- **Hand-off automation:** triggers on meaningful changes (threshold crossings, presence changes, periodic check), throttled so we don't call the LLM on every reading, and POSTs a compact snapshot to the agent endpoint.
- **MCP Server integration** enabled and secured, exposing only the entities/services the agent should control.
- **Remote access for the agent: Home Assistant Cloud (Nabu Casa)** — gives HA a secure public URL so the cloud agent can reach HA's MCP server from outside the home network, with no router port-forwarding. $6.50/month or $65/year.
- **Dashboard:** live readings per node/room, device states, recent AI decisions, and a 3D/floor-plan view.

---

## 6. Cloud AI Agent — AI subteam

- **Custom harness, not a third-party agent framework.** OpenClaw (general autonomous daemon) and Hermes Agent (hosted, memory-centric) were evaluated and not adopted: our loop is narrow (snapshot in → LLM + tools → MCP actions out), a small harness is easier for students to debug, and depending on a hosted product would reintroduce the lock-in we criticize in Miloco.
- **Hosting: AWS Lambda + API Gateway (HTTP API).** Two routes into the same function: `/ha-event` (from HA) and `/chat` (the chat bot's webhook). Effectively free at our volume.
- **🗳 TEAM DECISION — chat bot: Discord or Telegram.** Both work on Lambda with no always-on server:

| | **Discord** | **Telegram** |
|---|---|---|
| How messages reach Lambda | *Interactions Endpoint URL*: Discord POSTs each **slash command** (e.g. `/home turn off the lights`) to `/chat` | *Webhook* (`setWebhook`): Telegram POSTs **every message** sent to the bot to `/chat` |
| What users type | Slash commands only — reading ordinary chat messages needs Discord's always-on Gateway connection, which doesn't fit Lambda | Plain messages in a private chat or group with the bot — no command syntax needed |
| Response timing | **Must acknowledge within 3 seconds.** Lambda returns a deferred "thinking…" reply, runs the agent asynchronously (self-invoke or second Lambda), then edits the message via Discord's follow-up webhook | No 3-second rule. Lambda runs the agent, replies with the `sendMessage` API, then returns HTTP 200 (Telegram redelivers updates that fail or time out) |
| Request verification | Ed25519 signature on every request (`X-Signature-Ed25519`, `X-Signature-Timestamp`) — required, or Discord won't register the endpoint | Secret token set in `setWebhook`, checked on each request (`X-Telegram-Bot-Api-Secret-Token`) |
| Access control | Restrict commands to a private channel or role | Allow only known chat/user IDs |
| Fit for the team | Team already works in Discord — the bot sits in the same server as the meeting recaps | Separate app; simplest code path and natural phone-first chat |
| Effort | Higher (async deferred-reply flow + signature crypto) | Lower (one synchronous handler) |

- **Model testing through OpenRouter.** OpenRouter gives one OpenAI-compatible API and one credit balance for many models (Claude, GPT, Gemini, open-weight models), so the team can compare models on the same prompts before committing. Credits purchased Sept 24 (§7.4).
  - **What that means for MCP:** OpenRouter doesn't connect to MCP servers itself. The harness is the MCP client — it fetches HA's tool list from the MCP Server, converts the tools to OpenAI-style function definitions, sends them with the prompt, runs any tool calls the model returns against HA, and sends the results back until the model gives a final answer. This loop works with any provider, so it carries over whichever model wins.
  - **Compare models on:** correct tool choice, not acting when nothing needs doing, latency, and cost per decision.
- **Prompt design:** system prompt with household comfort targets, the snapshot (all sensor values + presence + time, per node), and rules for when *not* to act. Ask the model for a short human-readable explanation to send to the node's display.
- **Security:** keep HA tokens and API keys (OpenRouter, chat bot token) in AWS Secrets Manager/Lambda environment config, never in code; verify every chat request (Discord signature or Telegram secret token); restrict the bot to known channels/users; expose the minimum set of HA entities via MCP.
- **Latency:** Lambda cold starts add ~1–2 s — fine for automations and chat.

---

## 7. Bill of Materials & Spending

*The overall project budget hasn't been set yet. This section tracks what has been spent and what's still to buy. CNY prices converted at ≈ 6.72 CNY/USD (Sept 25, 2026) — approximate.*

### 7.1 Adafruit — order #3744273 (shipped)

Dev-only breakouts (replaced by SMT parts on the final PCB), except the PMS5003s, which move to the final nodes.

| Item | PID | List price | Qty | Discount | Total |
|---|---|---|---|---|---|
| SGP40 VOC Index sensor | 4829 | $14.95 | 2 | — | $29.90 |
| PMS5003 + breadboard adapter kit | 3686 | $39.95 | 2 | — | $79.90 |
| BH1750 light sensor | 4681 | $4.50 | 2 | — | $9.00 |
| SCD41 CO₂ / temp / RH sensor | 5190 | $49.95 | 2 | — | $99.90 |
| SHT41 temp & humidity sensor | 5776 | $5.95 | 2 | — | $11.90 |
| VL53L0X ToF distance sensor | 3317 | $14.95 | 2 | — | $29.90 |
| ESP32-C6-DevKitC-1-N8 | 5672 | $9.95 | 3 | — | $29.85 |
| KB2040 RP2040 board (promo, not used) | 5302 | $8.95 | 1 | −$8.95 (free) | $0.00 |
| PCB coaster (promo) | 5719 | $2.50 | 1 | −$2.50 (free) | $0.00 |
| **Subtotal** | | | | | **$290.35** |
| UPS Ground | | | | | $0.00 |
| Tax | | | | | $20.32 |
| **Order total** | | | | | **$310.67** |

### 7.2 Amazon

**Order #114-0143941-0754636** — placed Sept 14, delivered Sept 16

| Item | Seller | List price | Qty |
|---|---|---|---|
| Qoroos LD2410C (HLK-LD2410) 24 GHz mmWave presence module, 3-pack | HomarTech | $20.48 | 1 pack (3 modules) |
| ELEGOO 830-point solderless breadboard, 3-pack | ELEGOO Official US | $8.99 | 1 pack (3 boards) |
| **Items subtotal** | | **$29.47** | |
| Coupon savings | | −$1.02 | |
| Shipping | | $0.00 | |
| Total before tax | | $28.45 | |
| Tax | | $1.99 | |
| **Order total** | | **$30.44** | |

**Order #114-6199246-2757065** — placed Sept 24, delivered Sept 25 (dev-rig storage)

| Item | Seller | List price | Qty |
|---|---|---|---|
| IRIS USA 6 Qt storage bins with lids, clear, 4-pack | Amazon.com | $29.99 | 1 pack (4 boxes) |
| Shipping | | $2.99 − $2.99 free shipping = $0.00 | |
| Total before tax | | $29.99 | |
| Tax | | $2.10 | |
| **Order total** | | **$32.09** | |

### 7.3 Displays, LED strips & connectors (CNY) — used in dev, reused in the final nodes

All items below ship together in one air-freight shipment (in transit as of Sept 26); the shipping line includes duty and handling.

| Item | Unit | Qty | Total (CNY) | ≈ USD |
|---|---|---|---|---|
| DWIN DMG80480T050_09WN 5" display (12 V) | CNY 175 | 3 | 525.0 | $78.13 |
| 8-pin 2.0 mm display cable combo (SD003) | CNY 26.5 | 4 | 106.0 | $15.77 |
| BTF-Lighting ultra-narrow WS2812B, 3535, 144/m, 7.2 mm, 1 m | CNY 80 | 2 | 160.0 | $23.81 |
| BTF-Lighting FCOB WS2812B, 160/m, 5 mm, 1 m | CNY 25 | 2 | 50.0 | $7.44 |
| 3-pin SM LED connector pigtails, 15 cm (10/pack) | CNY 9 | 2 | 18.0 | $2.68 |
| **Parts subtotal** | | | **CNY 859.0** | **≈ $127.83** |
| Air freight (one shipment, duty + handling included) | | | 491.0 | $73.07 |
| **Total** | | | **CNY 1,350.0** | **≈ $200.89** |

### 7.4 Software & cloud services

| Item | Cost |
|---|---|
| **OpenRouter credits** (paid Sept 24) | **$21.19** |
| AWS Lambda | Free tier: 1M requests + 400,000 GB-s/month |
| API Gateway (HTTP API) | ~$1.00 per million requests; 1M/month free for 12 months on a new account |
| Discord or Telegram bot | Free |
| Home Assistant Cloud (Nabu Casa) — remote access | $6.50/month or $65/year (not yet purchased) |
| Raspberry Pi 5, Wi-Fi router | Already owned |

### 7.5 Spending to date

| | Amount |
|---|---|
| Adafruit #3744273 | $310.67 |
| Amazon #114-0143941-0754636 (LD2410C, breadboards) | $30.44 |
| Amazon #114-6199246-2757065 (storage boxes) | $32.09 |
| Displays, LED strips, connectors + air freight (CNY 1,350) | ≈ $200.89 |
| OpenRouter credits | $21.19 |
| **Total spent** | **≈ $595.28** |
| Upcoming recurring | Nabu Casa, $6.50/month or $65/year |

### 7.6 Still to buy — final product (3 nodes)

| Item | Notes |
|---|---|
| Custom PCB fabrication (JLCPCB) | 3 nodes + spares, one board or several (sensor / ESP / connector); optional SMT stencil; priced once the layout is done |
| SMT components | ESP32-C6 (bare chip + crystal + flash + RF parts; for the fallback, just header sockets — the DevKits are reused), SHT41, SGP40, SCD41, BH1750, VL53L0X, power conversion (buck or boost per §3.7), 3.3 V regulator, LED level shifter, passives, connectors |
| PMS5003 ×1 | For node 3 (nodes 1–2 reuse the dev units) |
| Wall adapters ×3 | 12 V or 5 V per §3.7, sized after measuring the LED bar draw |
| Enclosure filament | Free (printed by @spicybutter, PM) |

---

## 8. Subteams & Responsibilities

Three subteams: **Hardware (7)**, **Software (9)** and **AI (6)**. Planned member mix: **~28% freshmen, ~44% sophomores, ~28% juniors**.

How each subteam divides its work — roles inside each team, KiCad sheet ownership, firmware module groups, the GitHub workflow (Issue → Branch → PR into `dev` → Review → Merge, nothing pushed straight to `main` or `dev`) and code review rules — is in **`docs/Collaboration_Guidelines.md`**.

| Team | Lead — GitHub | Example groups (reference only) |
|---|---|---|
| Hardware | **@LiamWatson-Purdue** | Power (1) · MCU (2) · I/O (3) |
| Software | **@LukeTuthill** | Sensors & I/O (2) · Networking & MQTT (2) · Display & LED (2) · System Services (2) |
| AI — Cloud | **@ProgrammingJohn** (Cloud AI Lead) | LLM / agent logic · AWS & chat integration |
| AI — Local | **Undecided** (Local AI Lead) | HA devices & automation · HA interface & dashboard |

*The groups show how each subteam's work can be divided; they are not fixed assignments. We may not split people into groups at all — it depends on the work, and tasks are assigned by the leads.*

**PMs — @spicybutter and @rakkicow:** coordinate the subteams and are code owners on every path, so either can approve any PR. @spicybutter also builds and maintains the AI subteam's **emulator ESP32** (§3.1a), which feeds simulated data from three nodes into Home Assistant during early development.

### Hardware subteam (owns **Rig H**) — lead @LiamWatson-Purdue
- Build both breadboard rigs to one shared wiring diagram so Rig H and Rig S stay identical; own the **pin map** (shared with Software).
- Pre-power checks: set the DWIN display to **TTL mode**, confirm its 8-pin pinout, confirm LD2410C supply voltage.
- Power design: input option (§3.7), 5 V/3.3 V rails, LED bar power and brightness budget; measure current draw on the rig.
- Test both WS2812B strips (look, diffusion, current, fit) and pick the one for the final bar.
- Final PCB: schematic from the breakout reference designs, ESP32-C6 bare chip (or DevKit-on-headers fallback), single board or sensor/ESP/connector split, layout with placement rules (§3.8), JLCPCB order (+ stencil if needed), SMT soldering and assembly of **3 nodes**.
- Enclosure design and 3D printing ×3.
- *Skills:* circuits, datasheets, PCB CAD, soldering, CAD.

### Software subteam (owns **Rig S**) — lead @LukeTuthill
- ESP32-C6 firmware in **Rust on ESP-IDF v6.1.0**: firmware structure and shared state/MQTT scaffolding first, then driver tasks assigned across the team — I2C sensors, PMS5003 frame parsing, LD2410C GPIO, **DWIN DGUS display driver**, **WS2812B LED bar (RMT)**.
- **Display GUI:** a simple dev GUI for testing, then the polished final GUI (§3.4), plus the VP address map.
- Own the **MQTT topic contract** (§4); get clean data showing as HA entities, together with the AI subteam.
- Data quality: averaging, retries, dropped-reading recovery, fault reporting; Wi-Fi/MQTT reconnect; NVS provisioning; OTA if in scope.
- Bring the firmware up on the final PCBs (pin map changes only).
- *Skills:* Rust (or C with willingness to learn Rust), embedded basics, Git/GitHub. Recruitment listed Embedded C, so members should work through the Rust-on-ESP material below alongside the first sessions.
- **Learning resources:**
  - [Espressif – Rust on ESP (official docs)](https://docs.espressif.com/projects/rust/) — Espressif's official Rust documentation hub
  - [The Rust on ESP Book](https://docs.espressif.com/projects/rust/book/) — the basics of Rust on ESP chips. **Start here.**
  - [esp-rs/esp-idf](https://github.com/esp-rs/esp-idf) — the repo for Rust on ESP-IDF (`esp-idf-sys`, `esp-idf-hal`, `esp-idf-svc`): source, examples, changelogs and CI

### AI subteam
Led by a **Local AI Team Lead** (**undecided**) and a **Cloud AI Team Lead** (**@ProgrammingJohn**), who oversee the AI team together. The work splits into a local side and a cloud side:
- **Local (Raspberry Pi / Home Assistant):** MQTT broker, MQTT entities for all nodes, virtual devices, hand-off automations, HA's MCP Server (secured, minimal exposed entities), Nabu Casa remote access, real-time dashboard including the 3D/floor-plan view.
- **Cloud (AWS):** Lambda + API Gateway harness with the MCP client loop, OpenRouter model testing, prompt design, chat bot (Discord or Telegram, per team decision).
- **Emulator ESP32 (owner: @spicybutter, PM):** three simulated nodes on the real MQTT schema, with scripted events for agent testing (§3.1a).
- *Skills:* Python (Lambda), YAML/HA config, API usage; no data-science background needed.

### Interfaces between subteams

Each contract is written in `docs/interfaces/` in the repo before anyone builds against it, and changed only with the agreement of the teams on both sides.

| Contract | Between | Due |
|---|---|---|
| Rig wiring diagram + pin map + power rails | Hardware ↔ Software | Oct 4 |
| MQTT topic + JSON payload schema (emulator follows it too) | Software ↔ AI | Oct 4 |
| `/ha-event` snapshot format (HA → cloud agent) | Local AI ↔ Cloud AI | Oct 4 |
| AI message format (`text`, `ts`) + max length for the AI section | AI ↔ Software | Oct 18 |
| HA entity names + exposed MCP tools | AI, shared with all | Oct 18 |
| `/chat` request format (after the Discord/Telegram decision) | Cloud AI | Oct 25 |
| Final PCB pin map | Hardware → Software | Oct 25 |
| Final GUI layout ↔ enclosure bezel | Software ↔ Hardware | Nov 1 |

---

## 9. Timeline — Sunday Work Sessions

Work sessions run every Sunday, 1:00–4:00 PM, except holidays and breaks — **11 sessions** this semester. No sessions on **Oct 11**, **Nov 22** or **Nov 29**. Work continues between sessions; each session is where subteams integrate, test and hand off.

| # | Date | Hardware | Software | AI |
|---|---|---|---|---|
| 1–3 | Sep 6, 13, 20 ✓ | Kickoff phase: project introduction, brainstorming, BOM finalized, dev parts ordered (Adafruit, Amazon, displays/LED strips) | same | same; OpenRouter credits bought Sep 24 |
| 4 | **Sep 27** | **Put dev parts on the breadboards** (Rig H + Rig S); power checks. Displays and LED strips join the rigs when the air shipment arrives (set displays to TTL mode first) | **Set up the firmware structure** (ESP-IDF v6.1 project on the confirmed crates, first build → pin the nightly date, module layout, `system/state.rs`, `network/mqtt.rs`) and **start driver development** | **Set up MQTT** (Mosquitto) and **test MQTT data into Home Assistant** using the **emulator ESP32** (3 simulated nodes, PM); **start the cloud AI pipeline** (Lambda + API Gateway + OpenRouter) |
| 5 | Oct 4 | Finish both rigs; wiring diagram + pin map; measure current | I2C sensor drivers; MQTT publish from the rig; topic contract fixed | MQTT entities from the agreed schema (emulator data); virtual devices; first OpenRouter prompt tests on emulated scenarios |
| — | Oct 11 | *No session (break)* | | |
| 6 | Oct 18 | Compare LED strips and pick one; PCB schematic (bare-chip ESP32-C6 attempt) | PMS5003, LD2410C, DWIN driver + **dev GUI**, LED bar; MQTT subscribe | HA → Lambda hand-off; MCP client loop against HA's MCP Server; Nabu Casa remote access; start switching from emulated to real rig data |
| 7 | Oct 25 | PCB layout — **bare chip vs. DevKit-on-headers decided here**; final pin map to Software | Feature-complete on the rigs; AI messages on the display; robustness | Chat bot (Discord or Telegram); model comparison on OpenRouter; decision logic |
| 8 | Nov 1 | PCB review; **order PCBs (+ stencil if needed) from JLCPCB, SMT parts, 1 PMS5003, adapters**; start enclosure CAD | **Final GUI** design; hardening (reconnects, fault handling) | Dashboard incl. 3D/floor-plan view; full-pipeline tests on the rigs |
| 9 | Nov 8 | PCBs arrive (~7–8 days after ordering); **start SMT assembly**; enclosure CAD + test prints | Final GUI on the display; support integration tests | Pick final model; dashboard polish |
| 10 | Nov 15 | **Finish assembly of 3 nodes**; print enclosures; order a PCB re-spin now if a board needs a fix | Bring-up on the 3 PCBs; flash final firmware | Add all 3 nodes to HA; test on assembled nodes |
| — | Nov 22, Nov 29 | *No sessions (Thanksgiving break)* — any re-spin boards arrive; finish assembly or enclosure prints that slipped | | |
| 11 | Dec 6 | Final verification of the 3 nodes and the complete pipeline; enclosures fitted — **project complete, prep for ECE SPARK** | same | same |

**PCB lead time:** JLCPCB order → DHL delivery is usually ~7 days (+~1 day with a stencil). Ordering on Nov 1 puts boards in hand by the Nov 8 session and leaves room for one re-spin (ordered by Nov 15, delivered before Dec 6). If the bare-chip layout isn't ready by Oct 25, switch to the DevKit-on-headers fallback rather than slip the order.

---

## 10. Team Decisions (🗳 open — decided together at a work session)

| Decision | Options | Details |
|---|---|---|
| **What the LED bar shows** | A. Status only · B. AI feedback · C. Air-quality gauge · D. Mix of these | §3.5 |
| **Where the 3 final nodes go** | Rooms such as bedroom, living room, kitchen, study/desk, entryway — or one large space split into zones for the demo | Affects the dashboard floor plan, HA areas, the agent prompt (which room each reading describes) and which virtual devices each node relates to |
| **Chat bot platform** | Discord (slash commands, in the team's server) · Telegram (plain messages, simpler code) | §6 |

---

## 11. Collaboration & Meetings

- This is a collaborative project — research is encouraged, and the design isn't locked in. If you find a better sensor, agent design or dashboard approach, bring it up at a meeting or in Discord.
- **Work sessions:** Sundays, 1:00–4:00 PM — Sep 6, Sep 13, Sep 20, Sep 27, Oct 4, Oct 18, Oct 25, Nov 1, Nov 8, Nov 15, Dec 6.
- A recap is posted in the Discord channel after every meeting — check it if you miss one.

---

## Companion Documents

| Doc | Contents | Status |
|---|---|---|
| `docs/Collaboration_Guidelines.md` | Subteam roles, GitHub workflow, KiCad / firmware / AI collaboration rules, code review, cross-team hand-offs | Current |
| `software/Firmware_Architecture.md` | Toolchain, threads, shared state, DWIN display driver, LED bar, MQTT topics, boot sequence, module layout | Current (updated Sept 26) |
| `ai/AI_Agent_Notes.md` | Agent harness, HA hand-off, OpenRouter + MCP client loop, hosting, chat bot options, Nabu Casa, security | Current (updated Sept 26) |
| `hardware/Hardware_BOM_Candidates.md` | Hardware design decisions and trade-offs (chip, boards, display, LED bar, sensors, power, PCB structure) | Current (updated Sept 26) |
| `hardware/Full_Parts_List.md` | Per-part specs reference (interface, voltage, output, quantity, source) | Current (updated Sept 26). The old `IntelliThings_BOM.xlsx` is outdated |
| `IntelliThings_Slides.md` | Kickoff presentation (Sept 6) | **Archived** — kept as a record, no further updates |

---

## Sources

- [GitHub – XiaoMi/xiaomi-miloco](https://github.com/XiaoMi/xiaomi-miloco)
- [Pandaily – Xiaomi Open-Sources Miloco 2.0 Smart Home AI](https://pandaily.com/xiaomi-miloco-2-smart-home-ai-jun2026)
- [Home Assistant – AI Task integration](https://www.home-assistant.io/integrations/ai_task/) · [Model Context Protocol Server integration](https://www.home-assistant.io/integrations/mcp_server/)
- [Apidog – How to use MCP servers with OpenRouter](https://apidog.com/blog/use-mcp-servers-with-openrouter/)
- [DWIN – DMG80480T050_09W (industrial grade) specifications](https://www.dwin-global.com/5-0-inch-hmi-tft-lcd-modeldmg80480t050_09windustrial-product/) · [Evelta – DMG80480T050_09WN](https://evelta.com/5-800x480-non-touch-ttl-232-lcd-display/)
- [BTF-Lighting – Ultra Narrow WS2812B strip](https://www.btf-lighting.com/products/ultra-narrow-ws2812b-addressable-rgbic-led-strip?variant=46101774827746) · [BTF-Lighting – FCOB addressable strip](https://www.btf-lighting.com/products/fcob-rgb-addressable-led-strip-dc5v-160pixels)
- [Adafruit – SHT41 (5776)](https://www.adafruit.com/product/5776) · [SGP40 (4829)](https://www.adafruit.com/product/4829) · [SCD41 (5190)](https://www.adafruit.com/product/5190) · [BH1750 (4681)](https://www.adafruit.com/product/4681) · [VL53L0X (3317)](https://www.adafruit.com/product/3317) · [PMS5003 kit (3686)](https://www.adafruit.com/product/3686) · [ESP32-C6-DevKitC-1-N8 (5672)](https://www.adafruit.com/product/5672)
- [espboards.dev – LD2410](https://www.espboards.dev/sensors/ld2410/)
- [Espressif – Rust on ESP (official docs)](https://docs.espressif.com/projects/rust/) · [The Rust on ESP Book](https://docs.espressif.com/projects/rust/book/)
- [esp-rs/esp-idf – esp-idf-sys / -hal / -svc (repo, CI, changelogs)](https://github.com/esp-rs/esp-idf) · [esp-idf-sys](https://crates.io/crates/esp-idf-sys) · [esp-idf-hal](https://crates.io/crates/esp-idf-hal) · [esp-idf-svc](https://crates.io/crates/esp-idf-svc) on crates.io
- [Trading Economics – USD/CNY](https://tradingeconomics.com/china/currency)
- [Nabu Casa – Home Assistant Cloud pricing](https://www.nabucasa.com/pricing/)
- [AWS Lambda pricing](https://aws.amazon.com/lambda/pricing) · [API Gateway pricing](https://aws.amazon.com/api-gateway/pricing)
- [Discord – Interactions / receiving and responding](https://discord.com/developers/docs/interactions/receiving-and-responding) · [OneUptime – Serverless Discord bot on AWS](https://oneuptime.com/blog/post/2026-02-12-build-a-serverless-discord-bot-on-aws/view) · [Telegram Bot API](https://core.telegram.org/bots/api)
