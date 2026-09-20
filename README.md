----

# **Placeholder**

----

# IntelliThings

**An LLM-powered smart home companion system — from IoT to AIoT.**

Embedded Systems @ Purdue (ES@P) · Fall 2026 · ECE SPARK Challenge

---

## What we're building

Traditional smart homes make you write every automation rule by hand. IntelliThings
replaces the rulebook with a cloud AI agent that reasons over live environmental
sensor data and decides how the home should respond.

The centerpiece is a desktop companion unit: an ESP32-based device in a 3D-printed
chassis with a 7" display showing live sensor metrics and proactive, AI-generated
status statements. Distributed sensor nodes extend it to multi-room spatial
awareness.

**Sensors → AI → Action.**

No cameras. No local compute box. No ecosystem lock-in. Built on open protocols.

## Architecture

```
┌──────────────────────┐
│  Desktop Companion   │  ESP32-C5 + 7" display + custom PCB
│  + Sensor Nodes      │  temp · humidity · PM · TVOC · CO2
│                      │  ambient light · presence · ultrasonic · IR blaster
└──────────┬───────────┘
           │  Wi-Fi / MQTT / Matter
           ▼
┌──────────────────────┐
│  Home Assistant Hub  │  Raspberry Pi 5 running HAOS
│                      │  HA AI Task · MCP Server · virtual + real devices
└──────────┬───────────┘
           │  Model Context Protocol
           ▼
┌──────────────────────┐
│   Cloud AI Agent     │  Reasons over sensor state, issues device commands,
│      (LLM)           │  generates proactive status statements
└──────────┬───────────┘
           │
           ▼
┌──────────────────────┐
│  Actuation + UI      │  WS2812B strips · 5V relays · servo/fan
│                      │  Real-time dashboard incl. 3D floor-plan view
└──────────────────────┘
```

See [`docs/architecture.md`](docs/architecture.md) for the full breakdown.

## Repository layout

| Path | Subteam | Scope |
|---|---|---|
| [`software/`](software/) | Software | ESP-IDF + FreeRTOS firmware, sensor drivers, MQTT client, display UI, OTA, data processing |
| [`hardware/`](hardware/) | Hardware | Schematics, PCB layout, enclosure CAD, BOM, wiring |
| [`ai/`](ai/) | AI | Home Assistant + Pi 5 hub, MQTT broker, cloud agent, prompts, MCP, dashboard |
| [`docs/`](docs/) | Everyone | Architecture, timeline, onboarding |

## Getting started

```bash
git clone https://github.com/embedded-purdue/IntelliThings.git
cd IntelliThings
git checkout dev
```

Then read [`docs/onboarding.md`](docs/onboarding.md) for your subteam's toolchain setup.

## Branching

- **`main`** — protected. Stable, demo-ready. Changes only arrive via pull request.
- **`dev`** — integration branch. Day-to-day work lands here.
- **feature branches** — cut from `dev`, named `<team>/<short-description>`.

Full workflow in [`CONTRIBUTING.md`](CONTRIBUTING.md).

## Tech stack

**Embedded** Embedded C · ESP-IDF · FreeRTOS · ESP32-C5
**Protocols** Wi-Fi · MQTT · Matter · MCP
**Platform** Home Assistant OS on Raspberry Pi 5
**AI** Cloud LLM agent via Home Assistant AI Task + MCP Server
**Hardware** KiCad / Altium · Fusion 360 / SolidWorks · 3D printing

## Timeline

13 weeks. See [`docs/timeline.md`](docs/timeline.md).

| Weeks | Phase |
|---|---|
| 1–3 | Planning & Research |
| 4–5 | Hardware & Firmware Prototyping |
| 6–7 | AI & System Integration |
| 8–10 | PCB & Full Pipeline Development |
| 11–13 | Assembly & Final Integration |

## Team

Three subteams:

| Subteam | Owns |
|---|---|
| **Software** | ESP-32 firmware, sensor drivers, MQTT publish path, data validation |
| **Hardware** | Sensor wiring, PCB design, display + power integration, enclosure |
| **AI** | Home Assistant + AI Task, cloud agent, MCP server, virtual devices, dashboard |

Subteam assignments are advisory, not walls. If a subteam stalls, reinforce it.

**Meetings:** every Sunday, 1–4 PM. Recaps posted to Discord after each session.

## Collaboration

This is a collaborative project, so research is encouraged. **The design is not
locked in.** Have an idea for the technical path we should take? Bring it up.
