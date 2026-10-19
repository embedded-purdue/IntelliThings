# Demo master (ESP32-C3 + game buzzer)

*Status: not started — firmware is developed in this folder. See [the demo mode overview](../README.md).*

The **demo master** sits on the ECE SPARK booth table between the Raspberry Pi 5 and the
monitor. It's an **ESP32-C3 Core** board wired to a **game buzzer** (a big push button), and it
controls **demo mode**: pre-set scenarios, picked at random, that the nodes, Home Assistant and
the AI agent run through one after another.

## Hardware

| Part | Notes |
|---|---|
| ESP32-C3 Core board | Wi-Fi on the booth network. Exact board and USB variant: TBD |
| Game buzzer | Wiring (GPIO, button LED if any): TBD |

## What the firmware does

To be specified. From the team's plan (see [`../README.md`](../README.md)), demo mode needs:
- **a randomizer** that picks one of 50–200 pre-set scenario groups;
- **the scenario data delivered** to Home Assistant and the related sensor nodes;
- **the next scenario picked** after a time interval;
- **a web interface** related to the game buzzer, shown on the booth monitor.

Which of these run on the demo master, and what the buzzer does, are open questions listed in
[`../README.md`](../README.md#open-questions).

## Contracts

The demo master talks to the rest of the system over the booth Wi-Fi. Any MQTT topic or payload
it uses goes into [`docs/interfaces/mqtt.md`](../../docs/interfaces/mqtt.md) first.

## Layout

```
demo/demo-master/
└── README.md    this file (firmware project to be added)
```
