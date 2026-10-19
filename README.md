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

## Current Python prototype

The runnable AI pipeline is in [`ai/main.py`](ai/main.py). It uses Home Assistant
Container and a separate Mosquitto container on the project Pi, fetches
Assist-exposed context through MCP, and calls OpenRouter directly. It saves JSON
recommendations, optionally executes a configured virtual turn-on action, and
updates `input_text.ai_summary_overall`. A systemd timer runs it every five minutes.

See [local pipeline setup](ai/README.md#current-local-python-pipeline) for the root
`.venv`, `.env`, and run commands. The Lambda deployment and retained display-message
publishing described below are the target design, not features of this local
Python pipeline yet.

## Target architecture

```
Desktop sensor nodes (ESP32-C6, no_std Rust with esp-hal + Embassy)
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
| MCU | ESP32-C6-DevKitC-1-N8 — on the dev rigs, then plugged into sockets on the custom PCB in the final nodes |
| I2C sensors | SHT41 (temp/RH) · SGP40 (VOC Index) · SCD41 (CO₂) · BH1750 (light) · VL53L0X (ToF distance) |
| UART / GPIO sensors | PMS5003 (PM1.0/2.5/10) · LD2410C mmWave presence (OUT pin) |
| Display | DWIN DMG80480T050_09WN — 5", 800×480, UART (TTL mode), DGUS, 12 V |
| Light output | WS2812B addressable LED bar (RMT) |
| Power | 12 V or 5 V wall adapter (chosen during PCB design) |

Build plan: an **emulator ESP32** (3 simulated nodes, for early AI testing) → **2 identical
breadboard rigs** (Rig H for Hardware, Rig S for Software) → **3 final nodes** on custom SMT
PCBs in 3D-printed enclosures.

## Tech stack

**Firmware** Rust (`no_std`, stable; minimum 1.95, edition 2024) · `esp-hal` 1.2.2 · `esp-rtos` 0.4 · Embassy async tasks/networking · `esp-radio` 1.0.0-beta.1
**Protocols** Wi-Fi · MQTT · MCP (Matter was dropped)
**Hub** Raspberry Pi 5 · Raspberry Pi OS + Docker · Home Assistant Container · Mosquitto container · Python agent (OpenRouter)
**Cloud** OpenRouter for the LLM · future option: AWS Lambda + API Gateway (HTTP API) · Nabu Casa remote access
**Hardware** KiCad · JLCPCB · 3D printing

## Documentation

Start with the Project Guideline — it's the single current reference, and it wins if any
other doc disagrees.

| Doc | Contents |
|---|---|
| [Project Guideline](docs/Project_Guideline.md) | What we're building, BOM and spending, subteams, timeline, open team decisions |
| [Collaboration Guidelines](docs/Collaboration_Guidelines.md) | Subteam roles, GitHub workflow, KiCad / firmware / AI collaboration rules, code review, hand-offs |
| [Firmware Architecture](software/Firmware_Architecture.md) | Earlier ESP-IDF design: shared state, display and LED drivers, MQTT topics, boot sequence; see the [firmware README](software/firmware/README.md) for the current Rust toolchain and implementation |
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
| **AI** | 6 | @ProgrammingJohn | Home Assistant, broker, entities, virtual devices, MCP Server, dashboard; the agent, prompts, model testing, chat bot |
| **AI — Emulator** | | @spicybutter (PM) | Emulator ESP32 — 3 simulated nodes for early AI testing |
| **PM** | 2 | @spicybutter · @rakkicow | Cross-team coordination; code owners on every path. @rakkicow also owns the display UI |

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
| [`ai/`](ai/) | AI | Local Python pipeline, prompts, MCP tools, broker setup and scheduling · `emulator/` · `ha/` and `cloud/` scaffolding · agent notes |
| [`docs/`](docs/) | Everyone | Project Guideline, Collaboration Guidelines · `interfaces/` holds the shared contracts |

Never commit secrets — current firmware reads Wi-Fi credentials from the local, ignored
`software/firmware/.cargo/esp-config.toml` and embeds them at build time; cloud keys live in
AWS Secrets Manager / Lambda environment config.

## Timeline

11 Sunday work sessions (1:00–4:00 PM), Sept 6 – Dec 6. No sessions on Oct 11, Nov 22 or
Nov 29. Full per-subteam schedule in the
[Project Guideline §9](docs/Project_Guideline.md#9-timeline--sunday-work-sessions).

| Milestone | Date |
|---|---|
| Rigs built, emulator feeding HA ✓ | Sep 27 |
| MQTT schema agreed; LLM pipeline working ✓ | Oct 4 |
| All I2C sensors working; schematic; AI messages to the node displays ✓ | Oct 18 |
| PCB layout (DevKit + breakouts on sockets); final pin map | Oct 25 |
| PCB order (JLCPCB) | Nov 1 |
| 3 final nodes assembled and added to HA | Nov 15 |
| Full-system verification, ECE SPARK prep | Dec 6 |

## Collaboration

Research is encouraged and the design isn't locked in — if you find a better sensor, agent
design or dashboard approach, bring it up at a meeting or in Discord. A recap is posted in
Discord after every meeting.
