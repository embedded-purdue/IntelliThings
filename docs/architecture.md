# Architecture

## The idea

Most smart home systems are rule engines. You configure "if motion after sunset, turn
on hallway light," and the system does exactly that, forever, until you reconfigure it.

IntelliThings moves the decision layer to an LLM. Sensors report environmental state,
a cloud AI agent reasons over it, and the agent issues device commands through Home
Assistant. The household's routines get adapted to automatically rather than encoded
by hand.

**Sensors → AI → Action.**

## Why not cameras

The closest shipped comparison is Xiaomi Miloco 2.0, an open-sourced whole-home AI
system. It uses cameras plus a required local compute unit, locked to the Xiaomi
ecosystem.

| | Miloco 2.0 | IntelliThings |
|---|---|---|
| Sensing | Camera | Environmental sensors |
| Local compute unit | Required | None |
| Ecosystem | Xiaomi-locked | Open |
| Privacy | Camera-based | Sensor-based |
| Perception depth | Richer (visual) | Basic |
| Maturity | Shipped product | Prototype |

We trade perception depth for privacy, cost, and openness. Low-cost sensors at the
edge, all heavy reasoning pushed to the cloud, built on MQTT / Home Assistant / MCP.

## Layers

### 1. Desktop companion & sensor nodes

The flagship is a desktop unit: ESP32-C5, 7" color display, custom PCB, 3D-printed
chassis. It is both the primary sensor hub and the system's face — the display shows
live metrics and the agent's proactive status statements.

Additional distributed nodes extend coverage for multi-node spatial awareness.

**Sensors:** temperature, humidity, PM2.5/PM10, TVOC, CO2 (SCD40/41), ambient light
(BH1750), human presence, ultrasonic distance.
**Output:** IR blaster (universal remote), 7" display.

Firmware is Embedded C on ESP-IDF with FreeRTOS. Sensor drivers speak I2C. See
[`../software/`](../software/).

### 2. Transport

Wi-Fi for the network. MQTT for telemetry to the hub. Matter for local device
interop.

Nodes publish sensor readings to the broker; Home Assistant subscribes.

### 3. Home Assistant hub

Raspberry Pi 5 running Home Assistant OS. This is the integration point where
hardware, software, and the reasoning layer all have to land.

Responsibilities:
- MQTT broker, ingesting node telemetry
- Entity + device registry for real and virtual devices
- **HA AI Task** — the newer HA feature that hands context to an AI model
- **HA MCP Server** — exposes Home Assistant as callable tools for the agent
- Real-time dashboard, including a 3D floor-plan view

See [`../ai/`](../ai/).

### 4. Cloud AI agent

A cost-effective cloud LLM acting as the decision engine. It:
- interprets sensor state across nodes (spatial awareness)
- decides which devices to actuate, and calls them via MCP tools
- generates the proactive status statements shown on the companion display

Prompt design, MCP wiring, and evals live in [`../ai/`](../ai/).

### 5. Actuation

A hybrid environment, so the SPARK demo shows both scale and tangibility:

- **Virtual devices** in Home Assistant — represent a full-size home
- **Physical demo kit** — WS2812B LED strips (smart lighting), 5V relays
  (appliances), servo/DC fan (HVAC)

## Data flow

```
sensor read (I2C)
   → ESP32 firmware task
   → MQTT publish
   → Home Assistant entity state
   → AI Task context
   → cloud LLM reasoning
   → MCP tool call back into HA
   → device command (virtual or physical)
   → status statement pushed to companion display
```

## Known risks

**The AI / Home Assistant layer is the structural bottleneck.** Home Assistant, Matter, and the MCP
integration are the least-documented parts of the stack and the thinnest on
experience. Start the Pi 5 hub in week 1, not week 6.

**The display is the most visible failure mode.** The 7" panel is the flagship's
face. Bring-up is deliberately double-covered.

**PCB has one shot.** Five prototype fabs budgeted, ~15 SMT part types. Design review
before ordering matters more than speed.

## Bill of materials

Estimated budget ~$375. Full BOM in [`../hardware/README.md`](../hardware/README.md).
