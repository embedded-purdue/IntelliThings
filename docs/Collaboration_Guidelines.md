# IntelliThings — Subteam Collaboration Guidelines
*ES@P | Fall '26 | Last updated Oct 18, 2026*

> How the three subteams split the work, share files and hand off to each other. What we're building (parts, pins, topics, schedule) lives in `docs/Project_Guideline.md`; this doc covers **how we work together**. If the two disagree on a technical detail, the Project Guideline wins.

---

## 0. Shared Rules for Every Subteam

### Team at a glance

| Team | People | Lead(s) — GitHub | Example groups (reference only) |
|---|---|---|---|
| **Hardware** | 7 | Hardware Lead — **@LiamWatson-Purdue** | Power (1) · MCU (2) · I/O (3) |
| **Software** | 9 | Software Lead — **@LukeTuthill** | Sensors & I/O (2) · Networking & MQTT (2) · Display & LED (2) · System Services (2) |
| **AI** | 6 + PM | AI Lead — **@ProgrammingJohn** | HA side (3) · Agent side (3) · Emulator ESP32 (@spicybutter, PM) |
| **PM** | 2 | **@spicybutter** · **@rakkicow** | Cross-team coordination; code owners on every path. @spicybutter also owns the AI subteam's **emulator ESP32** (3 simulated nodes); @rakkicow owns the **display UI** and its VP map |

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
│   └── firmware/      no_std Rust node firmware (esp-hal + Embassy)
├── ai/                Python agent (agent/, config/, prompts/), deployment/, tools/, tests/
│   ├── emulator/      Emulator ESP32 — 3 simulated nodes for early AI testing
│   └── docs/          Pipeline details, design notes, roadmap, cloud option
├── demo/              Demo mode for the ECE SPARK booth
│   └── demo-master/   Demo master firmware (ESP32-C3 + game buzzer)
└── docs/              Project_Guideline.md, this doc
    └── interfaces/    The shared contracts below — one file each
```

CODEOWNERS requests a review from the subteam lead when a PR touches `software/`, `hardware/` or `ai/`, and from every lead when it touches `docs/interfaces/`. The two PMs, **@spicybutter** and **@rakkicow**, are code owners on every path and can approve any PR.

### Shared interfaces (contracts between teams)

A contract is **written in `docs/interfaces/` before anyone builds against it**, and **changed only after the teams on both sides agree**. A PR that changes a contract needs a review from **each affected team's lead**.

| Contract | Between | Status |
|---|---|---|
| Rig wiring diagram + pin map + power rails (`pin-map.md`) | Hardware ↔ Software | Stub: I2C pins and power confirmed, rest TBD |
| MQTT topics + JSON payloads — the single list of every topic (`mqtt.md`) | Software ↔ AI | Agreed Oct 4 |
| Display VP map (`display-vp-map.md`) | Software ↔ display UI (@rakkicow) | v1.0 |
| AI message format (`text`, `ts`) + max length for the display's AI section | AI ↔ Software | Format agreed; length still being tested |
| HA entity names + MCP tools exposed to the agent (`ha-entities.md`) | AI (shared with all) | Draft |
| `/ha-event` snapshot format | AI | Only for the future Lambda option |
| `/chat` request format (after the Discord/Telegram team decision) | AI | Due Oct 25 |
| Final PCB pin map | Hardware → Software | Due Oct 25 |
| Final GUI layout ↔ enclosure bezel | Software ↔ Hardware | Due Nov 1 |

### Communication

- **Sunday work sessions (1:00–4:00 PM)** are for integration, hand-offs and decisions; build work continues during the week.
- Each team has its own Discord channel. Questions that affect another team go in **that team's channel**, not DMs, so the answer is visible to everyone.
- **When unsure, ask — don't assume.** A wrong assumption about a pin, voltage or JSON field costs far more than a question.
- **🗳 Team decisions** (listed in the Project Guideline §10) are made together at a work session.

### Secrets

Never commit Wi-Fi passwords, HA tokens, OpenRouter / AWS keys, chat bot tokens or webhook secrets. Use `.gitignore`d local config files: the root `.env` for the AI pipeline, and `software/firmware/.cargo/esp-config.toml` for the firmware's Wi-Fi settings (compiled into the firmware — fine for the prototype). For the future cloud option, use AWS Secrets Manager / Lambda environment config.

**Shared team accounts and passwords** (HA logins, MQTT users, service accounts) go in the team's shared Google Drive folder (e.g. the AI Team folder) so nobody loses them — never in Git.

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
- Power input: **USB-C 5 V** (power only, 5.1 kΩ CC pull-downs) — decided
- Conversion: 5 V → 12 V boost for the display; 3.3 V from the DevKit
- **5 V budget sized for the LED bar**, which dominates current draw
- Protection and filtering (reverse polarity, fuse, input capacitors)
- Measures real current draw on Rig H before the design is finalized

**MCU (2)**
- **Decided: ESP32-C6-DevKitC-1 on sockets** — female headers matching the DevKit's pin layout, so the board can be pulled out (a bare chip was considered and dropped)
- GPIO assignment (with Software), strapping pins (GPIO4, 5, 8, 9, 15), boot/reset
- Programming and debug: native USB-JTAG (GPIO12/13)
- Antenna keep-out: no copper, parts or enclosure metal near the DevKit's antenna end

**I/O (3)**
- Sensors on the **I²C bus** (SDA GPIO6, SCL GPIO7): SHT41, SGP40, SCD41, BH1750, VL53L0X — the **breakout boards plug into sockets** on the PCB; make a **custom footprint for each breakout**
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

**Key dates:** schematic by **Oct 18** (DevKit + I2C sensors + USB-C power done) · layout by **Oct 25** · design review and **JLCPCB order on Nov 1**. Order-to-delivery is about 7 days by DHL (+~1 day with a stencil). Splitting into several boards adds no time.

**Branch examples:** `hardware/power-input`, `hardware/sensor-footprints`, `hardware/io-sensor-bus`, `hardware/led-bar`

---

## 2. Software Team (9 people)

The main platform is **ESP32-C6 + bare-metal `no_std` Rust** with `esp-hal`, `esp-rtos` and **Embassy** (stable Rust; versions in `software/README.md`). The goal is for 9 people to contribute in parallel while building **one clean, maintainable firmware**.

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
- **DWIN display driver over UART**: the ESP32 writes values into display variables (VP addresses) as defined in [`docs/interfaces/display-vp-map.md`](interfaces/display-vp-map.md). The screens are custom DGUS structures built by **@rakkicow (PM)**, so there's no framebuffer or pixel drawing on the ESP32. Which pages to drive is an open discussion with @rakkicow.
- Layout rule: all sensor readings always on screen; the AI section keeps the latest message until a new one arrives
- **WS2812B LED bar** driven by the ESP32, with a brightness cap (how to drive the strip is still being worked out with Hardware). Patterns follow the LED bar team decision.

**System Services (2)**
- Shared system state (latest readings, connection status, latest AI message)
- Configuration: Wi-Fi settings compiled in from the gitignored `.cargo/esp-config.toml` (accepted for the prototype)
- Logging, error handling and fault reporting; supervisor/reconnect loop; OTA if in scope

### Firmware structure

Keep functions in separate modules instead of one large program; modules are assigned as tasks, not owned by fixed groups. The current layout (`main.rs`, `wifi.rs`, `sensors/` with one file per sensor) and the planned modules are in `software/README.md`.

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

This also fixes the Rust project structure, module interfaces and crate versions. Wi-Fi and the I2C sensors already work (Oct 18); MQTT is next.

### Learning resources

- [Espressif – Rust on ESP (official docs)](https://docs.espressif.com/projects/rust/) — Espressif's official Rust documentation hub
- [The Rust on ESP Book](https://docs.espressif.com/projects/rust/book/) — the basics of Rust on ESP chips. **Start here.**
- [esp-rs/esp-hal](https://github.com/esp-rs/esp-hal) — the repo for bare-metal Rust on ESP chips: source, examples and changelogs

### Practices

- **Toolchain:** stable Rust + `rust-src`, target `riscv32imac-unknown-none-elf`; the exact crate versions and setup steps are in `software/README.md` and `software/firmware/README.md`. `cargo run` builds, flashes and opens the serial monitor.
- Before adding any crate, check it supports `no_std`, the ESP32-C6 and the current `esp-hal`/Embassy versions.
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

**One lead, @ProgrammingJohn, oversees the whole AI team.** The work divides into an HA side (the smart-home environment) and an agent side (the intelligence); everyone should understand both.

### Team structure

*Reference split — see the note in §0. Tasks are assigned by the lead; the groups show how the work divides, not fixed teams.*

| Group | Responsibilities |
|---|---|
| **AI Lead — @ProgrammingJohn** | Agent architecture and the MCP client loop; the HA ↔ agent interface; which entities and services are exposed to the agent through HA's MCP Server; reviews major changes |
| **HA side (3)** | HA Container + Mosquitto on the Pi; MQTT entities per node with availability (`status` topic); virtual devices (thermostat, lights, fan, purifier, humidifier…); automations/helpers that trigger the agent on big changes; dashboard incl. the AI summaries and the 3D floor plan |
| **Agent side (3)** | System prompt; model testing through **OpenRouter**; tool selection and knowing when **not** to act; latency and cost per decision; chat bot (**Discord or Telegram — team decision**); the future Lambda option |
| **Emulator ESP32 — @spicybutter (PM)** | 3 simulated nodes on the real MQTT schema, with scripted events for agent testing |

### System flow (as built)

```
ESP32 nodes (or emulator)
   │ MQTT
   ▼
Home Assistant (Container) + Mosquitto on the Pi
   │                         ▲ MCP tool call (allowlisted)
   │ MCP GetLiveContext      │
   ▼                         │
Python agent on the Pi (every 5 min) ──► LLM via OpenRouter
   │
   ▼
Room messages → each node's display topic · summaries + thinking status → HA dashboard
```

How the agent is triggered beyond the 5-minute timer (HA automations/helpers on big changes) is an open discussion. A cloud version (Lambda + API Gateway with `/ha-event` and `/chat`) is kept as a future option. Everyone should understand this whole flow, even if you own only one part of it.

### Define the interface first

Before building against it, write down in `docs/interfaces/`:
- HA entity naming and the MCP tools exposed to the agent (`ha-entities.md`)
- MQTT topics the agent publishes (`mqtt.md`)
- AI response format — the display message (`text`, `ts`) and what goes to the dashboard
- `/chat` request format (once the platform is chosen); `/ha-event` only if the Lambda option is built
- Error handling (HA unreachable, model timeout, bad tool call)
- Authentication and security

**Ownership:** the AI lead approves changes to the HA ↔ agent interface; contracts shared with other subteams also need their leads.

### AI testing

Test behavior **systematically**, not just whether a reply "looks good". Keep a shared set of **standard test scenarios** (e.g. hot room + presence, rising CO₂, PM2.5 spike, empty room, nothing wrong). Score each model and prompt on:
- Correct tool, device and arguments
- **Not acting** when nothing is needed
- Handling missing or bad sensor data, and unavailable devices
- Clear, short explanations (they must fit the display's AI section)
- Latency and cost per decision

The emulator's scripted events are a ready source of these scenarios.

### Code review

- **HA-side or agent change** → an AI team member who has worked on that part, or the AI lead
- **HA ↔ agent interface or a shared contract** → the AI lead (plus the other affected leads)

**Branch examples:** `ai/ha-mqtt-entities`, `ai/ha-dashboard`, `ai/mcp-config`, `ai/openrouter-tests`, `ai/chat-bot`

### Integration progression

Integrate continuously, not only at the end of the semester:

```
Real HA (emulator data) + agent on the Pi         ✓ (Oct 4)
  → Room messages to the node displays            ✓ (Oct 18)
  → More virtual devices driven by the agent
  → Real ESP32 rigs replace the emulator
  → 3 final nodes — full IntelliThings system (Nov 15 → Dec 6)
```

### Security

- Nothing secret in the repo (see §0).
- The agent gets **only** the HA entities and services it needs, set through HA's MCP exposure settings and the agent's allowlist.
- Verify every incoming chat request (Discord signature or Telegram secret token) and limit the bot to known channels/users.

---

## 4. Cross-Team Hand-offs at a Glance

| When | From → To | What |
|---|---|---|
| Sep 27 ✓ | @spicybutter (PM) → AI team | Emulator ESP32 publishing 3 simulated nodes |
| Oct 4 | Hardware → Software | Rig wiring diagram, pin map, power rails; Rig S identical to Rig H |
| Oct 4 ✓ | Software ↔ AI | MQTT topic + JSON schema agreed; the emulator publishing it |
| Oct 18 | AI ↔ Software | Display message format ✓ (AI section length still being tested); real rig data starts replacing emulator data |
| Oct 25 | Hardware → Software | Final PCB pin map (DevKit + breakouts on sockets) |
| Nov 1 | Software ↔ Hardware | Final GUI layout ↔ enclosure bezel; PCB order placed |
| Nov 15 | Hardware → Software → AI | Assembled nodes → firmware flashed → nodes added to HA |
| Dec 6 | Everyone | Full-system verification, ECE SPARK prep |
