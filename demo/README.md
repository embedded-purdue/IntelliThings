# Demo mode (ECE SPARK booth)

*Status: planned — agreed with the team on Oct 18. Details marked **TBD** are still open.*

## Why a demo mode

At the ECE SPARK Challenge the room's environment will barely change, and people will walk past
the booth all the time, so presence is always on. Real sensor data would therefore show visitors
almost nothing of what the agent can do. In **demo mode**, the system feeds **pre-set scenario
data** instead: a hot kitchen, a stuffy bedroom, a PM2.5 spike and so on. Visitors watch the LLM
reason over each scenario and act on the smart home.

## The booth (left to right)

| # | On the table | Role in demo mode |
|---|---|---|
| 1 | **Three sensor nodes** (kitchen, bedroom, living room) | Show the scenario data and their room's AI message on their DWIN displays |
| 2 | **Wi-Fi router** | Booth network |
| 3 | **Raspberry Pi 5** | Home Assistant, the Mosquitto broker and the AI agent |
| 4 | **Game buzzer with an ESP32-C3 Core** — the **demo master** | Demo control ([`demo-master/`](demo-master/)) |
| 5 | **Monitor** | The HA 3D dashboard and the game buzzer's web interface |

## How it works

1. **Scenario library:** **50–200 pre-set scenario groups** (format TBD).
2. **Randomizer:** picks one scenario group at random.
3. **The scenario's data is sent** to Home Assistant **and** to the related sensor nodes. The
   nodes show it on their DWIN displays **instead of** their real readings.
4. **The LLM reasons** over the scenario and takes smart-home actions through HA, as it does on
   live data.
5. **Each related node** shows its room's AI message; the **HA dashboard** shows the overall AI
   message.
6. **After a time interval** (TBD), the randomizer picks the next scenario, and the loop repeats.

**One of the three nodes shows both the demo data and its real sensor readings.** Its AI message
still comes from the demo data. Which node, and how the screen shows both, is TBD.

## Who builds what

| Part | Where | Owner |
|---|---|---|
| Demo master firmware (ESP32-C3 + game buzzer) | [`demo/demo-master/`](demo-master/) | TBD |
| Scenario library (50–200 groups) | TBD | TBD |
| Demo mode in the node firmware (show scenario data; the one node showing demo + real data) | `software/firmware/` | Software (@LukeTuthill) |
| Demo screens on the DWIN UI, if any are needed | VP map ([`docs/interfaces/display-vp-map.md`](../docs/interfaces/display-vp-map.md)) | @rakkicow (PM) |
| HA side + agent run per scenario | `ai/` | AI (@ProgrammingJohn) |
| 3D dashboard + the buzzer's web interface on the monitor | TBD | TBD |

**Contracts first:** any new MQTT topic or payload for demo mode (scenario data, demo on/off,
current scenario) goes into [`docs/interfaces/mqtt.md`](../docs/interfaces/mqtt.md) **before**
anyone builds against it. The scenario data format also needs its own contract.

## Open questions

- **What the game buzzer does:** start/stop demo mode, skip to the next scenario, or something
  else?
- **Where the randomizer and the scenario library live:** on the demo master, or on the Pi?
- **What a "scenario group" contains:** readings for all three rooms, or only the rooms a
  scenario affects? Plus a name or description to show on the monitor?
- **How demo data reaches HA and the nodes,** and how HA and the agent tell it apart from real
  data (topics, entities).
- **The interval between scenarios,** and whether the agent runs once per scenario instead of on
  its 5-minute timer.
- **The hybrid node:** which node it is, how its display shows both data sets, and whether its
  real readings still go to HA.
- **The buzzer's web interface:** what it shows and who serves it (the demo master, the Pi or
  HA).
- **Demo master toolchain:** C on ESP-IDF like the emulator, or Rust like the nodes?
- **How it relates to the emulator ESP32** (`ai/emulator/`), which already has scenarios and a
  control channel.
