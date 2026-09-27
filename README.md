# IntelliThings

**An LLM-powered smart home companion system — from IoT to AIoT.**

Embedded Systems @ Purdue (ES@P) · Fall 2026 · ECE SPARK Challenge

---

## What we're building

Classic smart homes stop at IoT: sensors report data, but a person still has to hand-write
"if this, then that" automations. IntelliThings closes the loop — **sensors → AI → action** —
so an LLM agent reasons over live sensor data and decides what the home should do.

*Example:* the room is warmer than comfortable **and** someone is present → the agent
decides to cool the room → Home Assistant turns on the fan → the node's display shows
"Room was 28 °C, turned on the fan."

Unlike camera-based systems such as Xiaomi Miloco 2.0, IntelliThings uses low-cost,
privacy-preserving **environmental sensors**, needs **no local compute box** (reasoning runs
in a cloud agent), and is built on **open protocols end to end** — MQTT, Home Assistant, MCP.

## Architecture

```
Desktop sensor nodes (ESP32-C6, Rust on ESP-IDF)
  7 sensors · 5" DWIN display · WS2812B LED bar
        │  Wi-Fi / MQTT                     ▲  retained AI message
        ▼  intellithings/<node_id>/sensors  │  intellithings/<node_id>/display
Raspberry Pi 5 — Home Assistant OS
  Mosquitto broker · MQTT entities · virtual devices · dashboard
        │  HA automation: HTTPS POST /ha-event    ▲  MCP tool calls (via Nabu Casa)
        ▼                                         │
AWS API Gateway → Lambda agent harness (MCP client) ── LLM via OpenRouter
        ▲
        └── /chat ◄── chat bot (Discord or Telegram — team decision)
```

The agent **decides**; Home Assistant's **MCP Server** is how the decision is executed.
Actions hit real devices where we have them and HA virtual devices (thermostat, lights, fan,
purifier, humidifier…) for the rest.

### Sensor node hardware

| Block | Part |
|---|---|
| MCU | ESP32-C6 (DevKitC-1-N8 for dev; bare chip on a custom PCB for the final nodes, DevKit-on-headers as fallback) |
| I2C sensors | SHT41 (temp/RH) · SGP40 (VOC Index) · SCD41 (CO₂) · BH1750 (light) · VL53L0X (ToF distance) |
| UART / GPIO sensors | PMS5003 (PM1.0/2.5/10) · LD2410C mmWave presence (OUT pin) |
| Display | DWIN DMG80480T050_09WN — 5", 800×480, UART (TTL mode), DGUS, 12 V |
| Light output | WS2812B addressable LED bar (RMT) |
| Power | 12 V or 5 V wall adapter (chosen during PCB design) |

Build plan: an **emulator ESP32** (3 simulated nodes, for early AI testing) → **2 identical
breadboard rigs** (Rig H for Hardware, Rig S for Software) → **3 final nodes** on custom SMT
PCBs in 3D-printed enclosures.

## Tech stack

**Firmware** Rust (`std`, nightly) on ESP-IDF v6.1.0 · `esp-idf-sys` 0.38 / `esp-idf-hal` 0.47 / `esp-idf-svc` 0.53 · FreeRTOS underneath
**Protocols** Wi-Fi · MQTT · MCP (Matter was dropped)
**Hub** Home Assistant OS on Raspberry Pi 5 · Mosquitto · Nabu Casa remote access
**Cloud** AWS Lambda + API Gateway (HTTP API) · Python · OpenRouter for model testing
**Hardware** KiCad · JLCPCB · 3D printing

## Documentation

Start with the Project Guideline — it's the single current reference, and it wins if any
other doc disagrees.

| Doc | Contents |
|---|---|
| [Project Guideline](docs/Project_Guideline.md) | What we're building, BOM and spending, subteams, timeline, open team decisions |
| [Collaboration Guidelines](docs/Collaboration_Guidelines.md) | Subteam roles, GitHub workflow, KiCad / firmware / AI collaboration rules, code review, hand-offs |
| [Firmware Architecture](software/Firmware_Architecture.md) | Toolchain, threads, shared state, display and LED drivers, MQTT topics, boot sequence |
| [AI Agent Notes](ai/AI_Agent_Notes.md) | Agent harness, HA hand-off, OpenRouter + MCP client loop, hosting, chat bot, security |
| [Hardware Design Decisions](hardware/Hardware_BOM_Candidates.md) | Why each part was chosen and what was considered |
| [Full Parts List](hardware/Full_Parts_List.md) | Per-part specs: interface, voltage, output, quantity, source |

Shared contracts between subteams (pin map, MQTT schema, `/ha-event` format, AI message
format, …) are written in `docs/interfaces/` **before** anyone builds against them.

## Team

| Subteam | Size | Lead | Owns |
|---|---|---|---|
| **Hardware** | 7 | @LiamWatson-Purdue | Rigs, pin map, power, PCB, enclosure |
| **Software** | 9 | @LukeTuthill | ESP32-C6 firmware, display GUI, MQTT topic contract |
| **AI — Cloud** | 6 (AI total) | @ProgrammingJohn | Lambda harness, MCP client loop, prompts, model testing, chat bot |
| **AI — Local** | | Undecided | Home Assistant, broker, entities, virtual devices, MCP Server, dashboard |
| **AI — Emulator** | | @spicybutter (PM) | Emulator ESP32 — 3 simulated nodes for early AI testing |
| **PM** | 2 | @spicybutter · @rakkicow | Cross-team coordination; code owners on every path |

## Workflow

**Issue → Branch → Change → Test → Pull Request → Review → Merge.**

- **`main`** — protected, demo-ready. Only receives `dev` at milestones.
- **`dev`** — integration branch. Every PR targets `dev`.
- **Feature branches** — cut from `dev`, named `software/…`, `hardware/…`, `ai/…` or `docs/…`.

Link the issue in the PR ("Closes #12") and say how you tested it. Full workflow in
[`CONTRIBUTING.md`](CONTRIBUTING.md).

```bash
git clone https://github.com/embedded-purdue/IntelliThings.git
cd IntelliThings
git checkout dev
```

## Repository layout

| Path | Owner | Contents |
|---|---|---|
| [`hardware/`](hardware/) | Hardware | KiCad project(s) + project-local libraries, enclosure CAD, datasheets · design decisions and parts list |
| [`software/`](software/) | Software | `firmware/` (Rust node firmware) · firmware architecture |
| [`ai/`](ai/) | AI | `emulator/` (emulator ESP32) · `ha/` (Home Assistant config) · `cloud/` (Lambda agent harness, chat bot) · agent notes |
| [`docs/`](docs/) | Everyone | Project Guideline, Collaboration Guidelines · `interfaces/` holds the shared contracts |

Never commit secrets — Wi-Fi credentials live in the ESP32's NVS, cloud keys in AWS
Secrets Manager / Lambda environment config.

## Timeline

11 Sunday work sessions (1:00–4:00 PM), Sept 6 – Dec 6. No sessions on Oct 11, Nov 22 or
Nov 29. Full per-subteam schedule in the
[Project Guideline §9](docs/Project_Guideline.md#9-timeline--sunday-work-sessions).

| Milestone | Date |
|---|---|
| Rigs built, baseline firmware, emulator feeding HA | Sep 27 |
| Pin map, MQTT schema and `/ha-event` format frozen | Oct 4 |
| HA → Lambda → MCP loop working; real rig data starts | Oct 18 |
| PCB layout; bare chip vs. DevKit fallback decided | Oct 25 |
| PCB order (JLCPCB) | Nov 1 |
| 3 final nodes assembled and added to HA | Nov 15 |
| Full-system verification, ECE SPARK prep | Dec 6 |

## Collaboration

Research is encouraged and the design isn't locked in — if you find a better sensor, agent
design or dashboard approach, bring it up at a meeting or in Discord. A recap is posted in
Discord after every meeting.
