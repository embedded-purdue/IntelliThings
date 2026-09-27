# IntelliThings — Subteam Collaboration Guidelines
*ES@P | Fall '26 | Last updated Sept 26, 2026*

> How the three subteams split the work, share files and hand off to each other. What we're building (parts, pins, topics, schedule) lives in `docs/Project_Guideline.md`; this doc covers **how we work together**. If the two disagree on a technical detail, the Project Guideline wins.

---

## 0. Shared Rules for Every Subteam

### Team at a glance

| Team | People | Lead(s) — GitHub | Example groups (reference only) |
|---|---|---|---|
| **Hardware** | 7 | Hardware Lead — **@LiamWatson-Purdue** | Power (1) · MCU (2) · I/O (3) |
| **Software** | 9 | Software Lead — **@LukeTuthill** | Sensors & I/O (2) · Networking & MQTT (2) · Display & LED (2) · System Services (2) |
| **AI** | 6 + PM | Cloud AI Lead — **@ProgrammingJohn** · Local AI Lead — **Undecided** | Local / Home Assistant (3) · Cloud AI Agent (3) · Emulator ESP32 (@spicybutter, PM) |
| **PM** | 2 | **@spicybutter** · **@rakkicow** | Cross-team coordination; code owners on every path. @spicybutter also owns the AI subteam's **emulator ESP32** (3 simulated nodes) |

> **About the groups:** the groups inside each subteam (and their head counts) are a **reference for how the work and collaboration can be divided** — not a fixed assignment. We may not split people into groups at all; that depends on the work at the time. In practice, **tasks are assigned** (through GitHub Issues) by the subteam lead, and anyone can pick up work outside "their" area. The collaboration rules below — who reviews what, who to talk to before changing an interface — apply whether or not the groups are used.

### GitHub workflow (all teams)

**Issue → Branch → Change → Test → Pull Request → Review → Merge**

- **Branches:** `main` is protected and demo-ready; `dev` is the integration branch. **Every change goes through a Pull Request into `dev`** — nobody pushes directly to `main` or `dev`. `dev` is merged into `main` only at milestones, coordinated by the leads.
- **Before starting:** check Issues so nobody else is on the same task, assign yourself, pull the latest `dev`, then branch from it.
- **Branch names** start with the team prefix: `software/…`, `hardware/…`, `ai/…`, `docs/…`
- **Commits:** small and descriptive, formatted `<area>: <what changed>` — "software: add SHT41 driver", "hardware: fix I²C pull-up values", not "update".
- **Link the Issue** in the PR ("Closes #12") and say how you tested it.
- Full details in `CONTRIBUTING.md` at the repo root.

### Repository layout

One repo for the whole project keeps interfaces and code in the same place:

```
IntelliThings/
├── hardware/          KiCad project(s), project-local symbol + footprint libraries, datasheets,
│                      Hardware_BOM_Candidates.md, Full_Parts_List.md
├── software/          Firmware_Architecture.md
│   └── firmware/      Rust (ESP-IDF) node firmware
├── ai/                AI_Agent_Notes.md
│   ├── emulator/      Emulator ESP32 — 3 simulated nodes for early AI testing
│   ├── ha/            Home Assistant config: MQTT entities, automations, dashboards (no secrets)
│   └── cloud/         Lambda agent harness, chat bot, deployment config
└── docs/              Project_Guideline.md, this doc
    └── interfaces/    The shared contracts below — one file each
```

CODEOWNERS requests a review from the subteam lead when a PR touches `software/`, `hardware/` or `ai/`, and from every lead when it touches `docs/interfaces/`. The two PMs, **@spicybutter** and **@rakkicow**, are code owners on every path and can approve any PR.

### Shared interfaces (contracts between teams)

A contract is **written in `docs/interfaces/` before anyone builds against it**, and **changed only after the teams on both sides agree**. A PR that changes a contract needs a review from **each affected team's lead**.

| Contract | Between | Due |
|---|---|---|
| Rig wiring diagram + pin map + power rails | Hardware ↔ Software | Oct 4 |
| MQTT topics + JSON payload schema (emulator follows it too) | Software ↔ AI | Oct 4 |
| `/ha-event` snapshot format (HA → cloud agent) | Local AI ↔ Cloud AI | Oct 4 |
| AI message format (`text`, `ts`) + max length for the display's AI section | AI ↔ Software | Oct 18 |
| HA entity names + MCP tools exposed to the agent | Local AI ↔ Cloud AI (shared with all) | Oct 18 |
| `/chat` request format (after the Discord/Telegram team decision) | Cloud AI | Oct 25 |
| Final PCB pin map | Hardware → Software | Oct 25 |
| Final GUI layout ↔ enclosure bezel | Software ↔ Hardware | Nov 1 |

### Communication

- **Sunday work sessions (1:00–4:00 PM)** are for integration, hand-offs and decisions; build work continues during the week.
- Each team has its own Discord channel. Questions that affect another team go in **that team's channel**, not DMs, so the answer is visible to everyone.
- **When unsure, ask — don't assume.** A wrong assumption about a pin, voltage or JSON field costs far more than a question.
- **🗳 Team decisions** (listed in the Project Guideline §10) are made together at a work session.

### Secrets

Never commit Wi-Fi passwords, HA tokens, OpenRouter / AWS keys, chat bot tokens or webhook secrets. Use `.gitignore`d local config files, NVS on the ESP32, and AWS Secrets Manager / Lambda environment config in the cloud.

---

## 1. Hardware Team (7 people)

The goal is not for everyone to edit everything — it's for everyone to **own one part of the hardware** while building one complete system.

### Team structure

*Reference split — see the note in §0. Tasks are assigned by the lead; the groups show how the work divides, not fixed teams.*

**Hardware Lead — Integration & Review (1) — @LiamWatson-Purdue**
- Coordinates hardware work and tracks interfaces between sections
- Owns the rig wiring diagram and the pin map shared with Software
- Reviews PRs, integrates sheets, coordinates the final layout and the JLCPCB order

**Power (1)**
- Power input: **12 V** (barrel jack) or **5 V** (USB-C or barrel jack) — option chosen during PCB design
- Conversion: 12 V → 5 V buck, or 5 V → 12 V boost for the display; 5 V → 3.3 V regulator
- **5 V budget sized for the LED bar**, which dominates current draw
- Protection and filtering (reverse polarity, fuse, input capacitors)
- Measures real current draw on Rig H before the design is finalized

**MCU (2)**
- **Plan: bare ESP32-C6 chip** — 40 MHz crystal, external SPI flash, RF matching and PCB antenna, following Espressif's hardware design guidelines
- **Fallback: DevKit-on-headers** — female headers matching the ESP32-C6-DevKitC-1 pin layout (preferred) or soldered male headers. Decided at the Oct 25 layout session.
- GPIO assignment (with Software), strapping pins (GPIO4, 5, 8, 9, 15), boot/reset
- Programming and debug: native USB-JTAG (GPIO12/13)
- Antenna keep-out: no copper, parts or enclosure metal near it

**I/O (3)**
- Sensors on the **I²C bus**: SHT41, SGP40, SCD41, BH1750, VL53L0X — pull-ups and decoupling copied from the Adafruit breakout reference designs
- **UART** connectors: PMS5003 and the DWIN display (8-pin 2.0 mm; display set to **TTL mode**)
- **GPIO:** LD2410C presence output
- **LED bar:** WS2812B data line through a 3.3 V → 5 V level shifter, series resistor, bulk capacitor, 3-pin connector
- Sensor placement rules: SHT41/SCD41 away from heat, PMS5003 airflow, VL53L0X and BH1750 windows, no metal in front of the LD2410C

### Breadboard phase (Rig H)

Before any KiCad work, the team builds **Rig H and Rig S from one wiring diagram**, so both rigs are identical. Any wiring change goes into the diagram first, then onto **both** rigs. Rig S belongs to the Software team: tell them before changing it.

### KiCad collaboration

**One board, one KiCad project, split into hierarchical sheets:**

```
Main system
├── Power
├── MCU
└── I/O
```

- Each sheet has an assigned owner at any given time (assigned through GitHub Issues). **Never have two people editing the same sheet at the same time** — KiCad schematic files don't merge well.
- **Project-local libraries:** keep every symbol and footprint the project uses in `hardware/` in the repo, so nobody's personal library breaks the build.
- **If the team splits into several PCBs** (sensor / ESP / connector-power board — an option in the Project Guideline), each board is its **own KiCad project**. Then the board-to-board connector pinout becomes a shared interface: write it in `docs/interfaces/` first.

**Agree before changing:** GPIO assignments, I²C addresses, UART use, voltage levels, power requirements, connector types and pinouts, physical locations on the board.

### PCB layout

The whole team can join layout decisions, but **not all 7 people route the PCB**.

1. Review the full schematic together.
2. Confirm footprints and part choices (and that parts are in stock).
3. Decide board outline, connector positions and sensor/antenna placement with the enclosure in mind.
4. **2–3 people** do placement and routing.
5. Everyone else reviews: datasheets, footprints, DRC results, placement rules.
6. **Final team design review before ordering.**

**Key dates:** schematic by **Oct 18** · layout, and bare chip vs. DevKit fallback decided, by **Oct 25** · design review and **JLCPCB order on Nov 1**. Order-to-delivery is about 7 days by DHL (+~1 day with a stencil). Splitting into several boards adds no time.

**Branch examples:** `hardware/power-input`, `hardware/mcu-bare-chip`, `hardware/io-sensor-bus`, `hardware/led-bar`

---

## 2. Software Team (9 people)

The main platform is **ESP32-C6 + Rust (`std`, nightly) on ESP-IDF v6.1.0**, using `esp-idf-sys` 0.38 / `esp-idf-hal` 0.47 / `esp-idf-svc` 0.53. The goal is for 9 people to contribute in parallel while building **one clean, maintainable firmware**.

### Team structure

*Reference split — see the note in §0. Tasks are assigned by the lead; the groups show how the work divides, not fixed teams.*

**Software Lead — Architecture & Integration (1) — @LukeTuthill**
- Maintains the firmware architecture and module interfaces
- Coordinates integration; reviews cross-module and architecture changes
- Owns the MQTT topic/schema contract with the AI team

**Sensors & I/O (2)**
- I²C drivers: SHT41, SGP40 (VOC Index, using SHT41 data for compensation), SCD41, BH1750, VL53L0X
- PMS5003 UART frame parsing; LD2410C presence via GPIO
- Bus setup and hardware abstraction; retries and averaging

**Networking & MQTT (2)**
- Wi-Fi connection and reconnection
- MQTT: publish `intellithings/<node_id>/sensors`, retained `status` with last-will, subscribe to the retained `display` topic
- `<node_id>` from the chip's MAC, so one firmware image fits every node

**Display & LED (2)**
- **DWIN DGUS driver over UART**: the ESP32 writes values into display variables (VP addresses). The screens themselves are designed in DWIN's DGUS tool and loaded from an SD card, so there's no framebuffer or pixel drawing on the ESP32.
- Screen design: a **simple dev GUI** first, then the **polished final GUI**, plus the VP address map
- Layout rule: all sensor readings always on screen; the AI section keeps the latest message until a new one arrives
- **WS2812B LED bar** through the RMT peripheral, with a brightness cap. Patterns follow the LED bar team decision.

**System Services (2)**
- Shared system state (latest readings, connection status, latest AI message)
- Configuration stored in NVS (Wi-Fi credentials, broker address — never hardcoded)
- Logging, error handling and fault reporting; supervisor/reconnect loop; OTA if in scope

### Firmware structure

Keep functions in separate modules instead of one large program. The module groups below match the example groups above, but modules are assigned as tasks, not owned by fixed groups.

```
software/firmware/src/
├── main.rs            boot, start threads, supervisor loop
├── hardware/
│   ├── bus.rs         I²C / UART setup
│   └── sensors/       sht41.rs  sgp40.rs  scd41.rs  bh1750.rs  vl53l0x.rs  pms5003.rs  ld2410c.rs
├── network/
│   ├── wifi.rs
│   └── mqtt.rs
├── ui/
│   ├── display.rs     DWIN DGUS driver
│   └── led.rs         WS2812B LED bar (RMT)
└── system/
    ├── state.rs       shared state
    ├── config.rs      NVS settings
    ├── logging.rs
    └── errors.rs
```

Modules talk through the **shared system state**, not directly to each other:

```
Sensors ──► System State ──┬──► MQTT publish
                           ├──► Display
                           └──► LED bar
MQTT (display topic) ──► System State (latest AI message) ──► Display / LED bar
```

### Baseline firmware first

Before anyone builds a separate feature, the team builds a **known-good baseline** together (started Sep 27, target merged by **Oct 4**):

```
Boot → init ESP32-C6 → init basic hardware → connect Wi-Fi → connect MQTT
     → publish a placeholder snapshot in the agreed JSON schema → main loop
```

This also fixes the Rust project structure, module interfaces and toolchain (pinned nightly, crate versions, MQTT managed component). Once the baseline is merged into `dev`, module tasks are assigned and worked on in parallel.

### Learning resources

- [Espressif – Rust on ESP (official docs)](https://docs.espressif.com/projects/rust/) — Espressif's official Rust documentation hub
- [The Rust on ESP Book](https://docs.espressif.com/projects/rust/book/) — the basics of Rust on ESP chips. **Start here.**
- [esp-rs/esp-idf](https://github.com/esp-rs/esp-idf) — the repo for Rust on ESP-IDF (`esp-idf-sys`, `esp-idf-hal`, `esp-idf-svc`): source, examples, changelogs and CI

### Practices

- **Toolchain versions** (confirmed against [esp-rs/esp-idf](https://github.com/esp-rs/esp-idf), where the three crates now live):
  - **ESP-IDF v6.1.0**, written as `ESP_IDF_VERSION = "v6.1"` (ESP-IDF's tag for 6.1.0).
  - **Crates:** `esp-idf-sys = "0.38.1"`, `esp-idf-hal = "0.47.0"`, `esp-idf-svc = "0.53.0"` — the first releases with ESP-IDF 6.1 fixes; esp-rs CI builds the ESP32-C6 target against v6.1.
  - **Rust: nightly + `rust-src`** (required to build `std` for `riscv32imac-esp-espidf`); minimum Rust 1.82. On Sep 27, whoever gets the first clean build pins that **dated nightly** in `rust-toolchain.toml`, and everyone uses it from then on.
  - **MQTT** was moved out of ESP-IDF in v6 — add `remote_component = { name = "espressif/mqtt", version = "1.*" }` under `[[package.metadata.esp-idf-sys.extra_components]]` in `Cargo.toml`.
  - Before adding any third-party crate (WS2812 driver, sensor drivers), check it works with `esp-idf-hal` 0.47 / `embedded-hal` 1.0.

  Starter config:
  ```toml
  # .cargo/config.toml
  [build]
  target = "riscv32imac-esp-espidf"   # ESP32-C6
  [target.'cfg(target_os = "espidf")']
  linker = "ldproxy"
  [unstable]
  build-std = ["std", "panic_abort"]
  [env]
  ESP_IDF_VERSION = "v6.1"

  # rust-toolchain.toml
  [toolchain]
  channel = "nightly"                 # replace with the pinned nightly-YYYY-MM-DD
  components = ["rust-src"]
  ```
- Run `cargo fmt` and `cargo clippy` before opening a PR.
- **Test on Rig S** before a PR. Wiring changes to the rig go through the Hardware team.
- The AI team isn't waiting on you: its emulator ESP32 covers them until the real nodes publish.

### Code review

Not every PR has to wait for the lead:
- **Sensor change** → a teammate who has worked on sensors reviews
- **Display/LED change** → a teammate who has worked on the display or LED bar reviews
- **Architecture, cross-module or MQTT schema change** → the Software Lead reviews

**Talk to whoever owns that module or task first** before changing: GPIO assignments, sensor data structures, MQTT topics/messages, system state, module APIs, display VP map, shared dependencies.

**Branch examples:** `software/baseline`, `software/sht41-driver`, `software/mqtt-reconnect`, `software/dwin-dev-gui`, `software/led-bar`

**General rule: own your module, but understand the whole system.** You're welcome to help with other modules, review others' code and learn other parts of the firmware.

---

## 3. AI Team (6 people)

**Local AI owns the smart-home environment. Cloud AI owns the intelligence. The interface between them belongs to both.**

### Team structure

*Reference split — see the note in §0. Tasks are assigned by the leads; the groups show how the work divides, not fixed teams.*

**Local AI / Home Assistant (3)**

| Role | Responsibilities |
|---|---|
| **Local AI Lead — HA Integration** (**Undecided**) | Raspberry Pi 5 / Home Assistant environment; Mosquitto broker; MQTT → HA integration; **which entities and services are exposed to the agent through HA's MCP Server**; coordination with Cloud AI |
| **HA Devices & Automation** | MQTT entities per node with availability (`status` topic); virtual devices (thermostat, lights, fan, purifier, humidifier…); hand-off automations and triggers; **throttling so the LLM isn't called on every reading**; publishing AI messages to each node's retained `display` topic |
| **HA Interface & Dashboard** | Dashboard: node/room views, device states, AI decision history, floor-plan / 3D view (community card or custom); **Home Assistant Cloud (Nabu Casa)** remote access and MCP Server setup |

**Cloud AI Agent (3)**

| Role | Responsibilities |
|---|---|
| **Cloud AI Lead — Agent Architecture** (**@ProgrammingJohn**) | Agent loop and tool interface; the **MCP client** (fetch HA's tools → give them to the model → run its tool calls against HA → send results back); cloud ↔ HA integration; reviews major agent changes |
| **LLM / Agent Logic** | System prompt; model testing through **OpenRouter**; tool selection and decision logic; knowing when **not** to act; latency and cost per decision |
| **AWS & Chat Integration** | AWS Lambda + API Gateway (`/ha-event`, `/chat`); chat bot (**Discord or Telegram — team decision**; Discord needs the 3-second deferred-reply flow and Ed25519 signature check, Telegram a webhook secret token); secrets; request verification and access control; logging and deployment |

### System flow

```
ESP32 nodes (or emulator)
   │ MQTT
   ▼
Home Assistant ── automation ──HTTPS POST /ha-event──► Lambda agent harness
                                                          │
                                                          ▼
                                               LLM (via OpenRouter)
                                                          │ tool calls
                                                          ▼
Home Assistant MCP Server ◄──── MCP (via Nabu Casa) ── harness runs them
   │
   ▼
Real / virtual devices  →  dashboard  +  AI message → node display
```

Chat enters through the cloud side:

```
Discord or Telegram → API Gateway (/chat) → Lambda → agent → HA MCP Server → Home Assistant
```

Everyone should understand this whole flow, even if you own only one part of it.

### Define the interface first

Before the two groups build independently, write down in `docs/interfaces/`:
- `/ha-event` request format — the snapshot HA sends (per node: `node_id`, all sensor fields and units, presence, time)
- `/chat` request format (once the platform is chosen)
- HA entity naming
- MCP tools exposed to the agent
- AI response format — the display message (`text`, `ts`) and what's logged to the dashboard
- Error handling (HA unreachable, model timeout, bad tool call)
- Authentication and security

**Ownership:** Local AI owns *Home Assistant → snapshot/event format*. Cloud AI owns *snapshot → AI decision → MCP calls*. **Both leads approve any change to the interface between them.**

### Parallel development

**Local team — start from the emulator ESP32** (3 simulated nodes, all sensor fields, same MQTT schema):

```
MQTT → HA devices → virtual devices → automations → MCP Server + Nabu Casa → dashboard
```

**Cloud team — don't wait for Home Assistant.** Start from mock snapshots in the agreed schema. Illustrative only: field names follow the MQTT schema fixed Oct 4, and room names follow the node-placement team decision.

```json
{
  "node_id": "emu-kitchen",
  "temperature_c": 27.2,
  "humidity_pct": 68,
  "co2_ppm": 1250,
  "voc_index": 180,
  "pm2_5_ugm3": 12,
  "lux": 240,
  "presence": true,
  "ts": "2026-10-04T14:05:00-04:00"
}
```

```
Mock snapshot → agent harness → OpenRouter → tool decision → mock MCP action
```

Once both sides work on their own, swap the mocks for the real interface.

### AI testing

Test behavior **systematically**, not just whether a reply "looks good". Keep a shared set of **standard test scenarios** (e.g. hot room + presence, rising CO₂, PM2.5 spike, empty room, nothing wrong). Score each model and prompt on:
- Correct tool, device and arguments
- **Not acting** when nothing is needed
- Handling missing or bad sensor data, and unavailable devices
- Clear, short explanations (they must fit the display's AI section)
- Latency and cost per decision

The emulator's scripted events are a ready source of these scenarios.

### Code review

- **Local HA change** → a Local AI member or the Local AI Lead (while the Local AI Lead is undecided, the Cloud AI Lead or a PM reviews)
- **Cloud agent change** → a Cloud AI member or the Cloud AI Lead
- **Cloud ↔ HA interface change** → **both leads**

**Branch examples:** `ai/ha-mqtt-entities`, `ai/ha-dashboard`, `ai/mcp-config`, `ai/lambda-agent`, `ai/openrouter-tests`, `ai/chat-bot`

### Integration progression

Integrate continuously, not only at the end of the semester:

```
Mock HA + mock agent
  → Real HA (emulator data) + mock agent
  → Real HA + real cloud agent            (by Oct 18)
  → Virtual devices driven by the agent
  → Real ESP32 rigs replace the emulator  (from Oct 18)
  → 3 final nodes — full IntelliThings system (Nov 15 → Dec 6)
```

### Security

- Nothing secret in the repo (see §0).
- The agent gets **only** the HA entities and services it needs, set through HA's MCP exposure settings.
- Verify every incoming chat request (Discord signature or Telegram secret token) and limit the bot to known channels/users.

---

## 4. Cross-Team Hand-offs at a Glance

| When | From → To | What |
|---|---|---|
| Sep 27 | @spicybutter (PM, AI) → Local AI | Emulator ESP32 publishing 3 simulated nodes |
| Oct 4 | Hardware → Software | Rig wiring diagram, pin map, power rails; Rig S identical to Rig H |
| Oct 4 | Software ↔ AI | MQTT topic + JSON schema frozen; baseline firmware and the emulator publishing it |
| Oct 18 | AI ↔ Software | Display message format + AI section length; real rig data starts replacing emulator data |
| Oct 25 | Hardware → Software | Final PCB pin map (bare chip or DevKit fallback) |
| Nov 1 | Software ↔ Hardware | Final GUI layout ↔ enclosure bezel; PCB order placed |
| Nov 15 | Hardware → Software → AI | Assembled nodes → firmware flashed → nodes added to HA |
| Dec 6 | Everyone | Full-system verification, ECE SPARK prep |
