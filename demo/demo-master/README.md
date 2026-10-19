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
| **Game buzzer** | An Amazon game buzzer on **2 × AA batteries**; a press makes its LED blink and plays a sound. Two wires go to the ESP32-C3: signal and GND (see below) |

## Firmware

- **C on ESP-IDF v6.1**, like the emulator (`ai/emulator/esp32_c3_emulator/`). Its Wi-Fi
  manager, MQTT connections, time sync and embedded web server are good starting points.
- **Wi-Fi and broker settings** in a gitignored `sdkconfig.local`, as in the emulator. Never
  commit them.

### What it does

| Feature | Behaviour |
|---|---|
| **Scenario library** | Up to ~200 pre-set scenario groups, normal and abnormal; each sets **every** entity in the demo home, inside and outside, including device states (lights, doors, car, presence). Drafts to be generated later |
| **Scenario loop** | Picks a random group, publishes it, waits an interval (open discussion), picks the next |
| **Buzzer press** | Stops the loop; the web interface rolls every value for about **4 s** (to be tuned); every entity, device states included, gets a **random value within its caps** and is published; **after a fixed time** the loop resumes |
| **Realistic caps** | Each entity has a low and high cap it never crosses (e.g. indoor temperature 60–100 °F; near 100 °F reads as a fire hazard). Published in °C |
| **Web interface** | The home's entities in rows and columns; a number/text rolling animation on every change; a buzzer press randomizes every value immediately |
| **MQTT** | Publishes under the proposed `intellithings/demo/#` topics (see [`../README.md`](../README.md#mqtt-proposed)). **Never calls the LLM**: HA automations decide when the agent runs |

## Game buzzer wiring

The buzzer is a self-contained, battery-powered toy, so the plan is to **find a point on its PCB
that carries a voltage when it's pressed** and wire it to the ESP32-C3: **signal → a GPIO,
buzzer GND → ESP32 GND**. Things to check when testing the board:

- **Measure the voltage first** with a multimeter. 2 × AA is about **3.0–3.2 V** with fresh
  cells, just inside the ESP32-C3's 3.3 V logic. **Never feed it more than 3.3 V.** If the point
  you pick is higher, use a resistor divider.
- **Connect the grounds** (buzzer GND to ESP32 GND), or the signal has no reference.
- **Put a ~1 kΩ resistor in series** with the signal wire, to protect both boards if something
  is wired wrong.
- **Don't connect the buzzer's battery + to the ESP32's 3.3 V,** so the two supplies never fight.
- **The press may look like several pulses** if the tapped point follows the blinking LED or the
  sound. The firmware should act on the first edge, then **ignore the input** for a while (at
  least the roll plus the fixed time), which also stops visitors re-triggering a roll mid-roll.
- **The best point is usually the switch contact itself** (a clean high/low), rather than the
  LED or speaker drive, which pulse.

The GPIO is still to be picked. Avoid the C3's strapping pins (GPIO2, 8, 9), the board's
onboard LEDs (GPIO12/13) and, on the native-USB board variant, the USB pins (GPIO18/19).

## Contracts

Before the firmware publishes anything, the demo topics and payloads go into
[`docs/interfaces/mqtt.md`](../../docs/interfaces/mqtt.md), and the demo-home entity list and caps
go into [`docs/interfaces/ha-entities.md`](../../docs/interfaces/ha-entities.md).

## Layout

```
demo/demo-master/
└── README.md    this file (ESP-IDF project to be added)
```
