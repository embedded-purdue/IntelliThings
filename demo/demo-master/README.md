# Demo master (ESP32-C3 + game buzzer)

Owner: **@spicybutter (PM)** · *Status: not started — firmware is developed in this folder.*
See [the demo mode plan](../README.md).

The **demo master** sits on the ECE SPARK booth table between the Raspberry Pi 5 and the monitor.
It holds the demo home's **scenario library** and **randomizer**, reads the **game buzzer**,
publishes the demo data over MQTT and serves the **web interface** shown on the booth monitor.

## Hardware

| Part | Notes |
|---|---|
| **LuatOS ESP32-C3 Core** | Same board family as the emulator ESP32; Wi-Fi on the booth network |
| **Game buzzer** (a big push button) | Wiring (GPIO, button LED if any): TBD |

## Firmware

- **C on ESP-IDF v6.1**, like the emulator (`ai/emulator/esp32_c3_emulator/`). Its Wi-Fi
  manager, MQTT connections, time sync and embedded web server are good starting points.
- **Wi-Fi and broker settings** in a gitignored `sdkconfig.local`, as in the emulator. Never
  commit them.

### What it does

| Feature | Behaviour |
|---|---|
| **Scenario library** | 50–200 pre-set scenario groups; each sets a value for **every** entity in the demo home (inside and outside). Drafts to be generated later |
| **Scenario loop** | Picks a random group, publishes it, waits an interval (TBD), picks the next |
| **Buzzer press** | Stops the loop; the web interface rolls every value for about **4 s** (to be tuned); then every entity gets a **random value within its caps** and is published |
| **Realistic caps** | Each entity has a low and high cap it never crosses (e.g. indoor temperature 60–100 °F; near 100 °F reads as a fire hazard). Published in °C |
| **Web interface** | The home's entities in rows and columns; a number/text rolling animation on every change; a buzzer press randomizes every value immediately |
| **MQTT** | Publishes under the proposed `intellithings/demo/#` topics (see [`../README.md`](../README.md#mqtt-proposed)). **Never calls the LLM**: HA automations decide when the agent runs |

## Contracts

Before the firmware publishes anything, the demo topics and payloads go into
[`docs/interfaces/mqtt.md`](../../docs/interfaces/mqtt.md), and the demo-home entity list and caps
go into [`docs/interfaces/ha-entities.md`](../../docs/interfaces/ha-entities.md).

## Layout

```
demo/demo-master/
└── README.md    this file (ESP-IDF project to be added)
```
