# Demo mode (ECE SPARK booth)

*Status: planned — agreed with the team on Oct 18. Details marked **TBD** are still open.*

## Why a demo mode

At the ECE SPARK Challenge the room's environment will barely change, and people will walk past
the booth all the time, so presence is always on. Real sensor data would therefore show visitors
almost nothing of what the agent can do. In **demo mode**, the system feeds **pre-set and
randomized home data** instead: a hot kitchen, a stuffy bedroom, a smoky garage, a cold night
outside. Visitors watch the LLM reason over it and act on the smart home.

## The booth (left to right)

| # | On the table | Role in demo mode |
|---|---|---|
| 1 | **Three sensor nodes** (kitchen, bedroom, living room) | Each shows its own room's demo data and AI message on its DWIN display |
| 2 | **Wi-Fi router** | Booth network |
| 3 | **Raspberry Pi 5** | Home Assistant, the Mosquitto broker and the AI agent |
| 4 | **Game buzzer with a LuatOS ESP32-C3 Core** — the **demo master** | Holds the scenario library and the randomizer, serves the web interface ([`demo-master/`](demo-master/)) |
| 5 | **Monitor** | The HA 3D dashboard and the demo master's web interface |

## How it works

### The demo home

The demo covers **the whole home, inside and outside**, not just the three node rooms: for
example the kitchen, living room, master bedroom, second bedroom, garage, patio, garden and the
weather outside. It includes **everything that can change or be controlled**: sensor readings,
**lights, doors, the car in the garage, human presence** and so on. Every value for every entity
comes from the demo master. **Only the three real nodes show data on a DWIN display**, each for
its own room. The rest appears in HA (dashboard, 3D floor plan) and to the agent.

**In demo mode the agent sees only the demo master's entities.** The nodes may keep sending
their real readings to HA, but that isn't needed during the demo.

The entity list (areas, entities and their caps) is **TBD**, to be agreed with the AI team and
added to [`docs/interfaces/ha-entities.md`](../docs/interfaces/ha-entities.md).

### Scenario loop

1. **The scenario library** on the demo master holds **up to ~200 pre-set scenario groups**, as
   many as possible. They mix **normal homes** with **abnormal ones** (a smoky kitchen, a door
   left open at night, a hot empty room…), so visitors see the agent both stay calm and react.
   Each group sets a value for **every** entity in the demo home, including device states.
2. **The randomizer** picks one group and the demo master publishes it, to HA and to each node
   for its room.
3. **Home Assistant automations/helpers decide when to call the LLM.** The demo master never calls
   it directly.
4. **The agent reasons and acts** through HA as it does on live data.
5. **Each node** shows its room's AI message; the **HA dashboard** shows the overall message.
6. **After a time interval,** the randomizer picks the next group. The interval still needs
   experiments: **open discussion**.

### Game buzzer: "roll the house"

Visitors are invited to **press the buzzer**:

1. **The scenario loop stops.**
2. The web interface plays a **rolling animation** on every value for about **4 seconds**.
3. **Every value lands on a new random value** and is published to HA and the nodes. Visitors
   then watch how the LLM reacts.
4. **After a fixed time, the scenario loop resumes.**

The roll length and the fixed time still need experiments: **open discussion**.

**Random, but realistic.** Each entity has a **low and high cap**, and values never go outside
it, because anything outside isn't a normal home. Example: indoor temperature between **60 °F and
100 °F**; a roll near 100 °F reads as a possible fire hazard, which is exactly the kind of thing
the agent should catch. Device states (lights on/off, doors open/closed, car in/out, someone
home or not) are rolled too. MQTT still carries **°C** (60–100 °F ≈ 15.6–37.8 °C); °F is only
for what people see. The caps for each entity are **TBD**, along with whether values are spread
evenly across the range or lean toward normal.

### The web interface (on the monitor)

Served by the demo master:
- **The home's entities in rows and columns,** with their current values.
- **A number or text rolling animation** whenever a value changes (new scenario or buzzer roll).
- **A buzzer press randomizes every value immediately.**

### The hybrid node — TODO

One of the three nodes will show **both the demo data and its real sensor readings**; its AI
message still comes from the demo data. **TODO:** which node, how its screen shows both (the VP
map needs a field for it; @rakkicow), and whether its real readings still go to HA.

## MQTT (proposed)

All demo traffic lives under a new **reserved `intellithings/demo/#`** namespace, next to
`intellithings/ai/#` and `intellithings/emulator/#`. **Proposed, not agreed:** it goes into
[`docs/interfaces/mqtt.md`](../docs/interfaces/mqtt.md) once the Software and AI leads agree.

| Topic (proposed) | Publisher → subscriber | Retained | Payload |
|---|---|---|---|
| `intellithings/demo/status` | demo master → anyone | yes (Last Will) | `online` / `offline` |
| `intellithings/demo/state` | demo master → nodes, HA | yes | Demo mode (off / scenario / random), the current scenario's ID and name, `ts` — fields TBD |
| `intellithings/demo/home` | demo master → HA | yes | Values for **every** demo-home entity, readings and device states — format TBD (depends on the entity list) |
| `intellithings/demo/<node_id>/sensors` | demo master → that node | yes | **The same JSON schema as `intellithings/<node_id>/sensors`**, so the node can reuse its parsing and display code |

The AI messages need nothing new: the agent keeps publishing to `intellithings/<node_id>/display`
and `intellithings/ai/summary`.

## Who builds what

| Part | Where | Owner |
|---|---|---|
| Demo master firmware: scenario library, randomizer, buzzer, web interface, MQTT | [`demo/demo-master/`](demo-master/) | **@spicybutter (PM)** — C on ESP-IDF, LuatOS ESP32-C3 Core (like the emulator) |
| Scenario library drafts (50–200 groups) | `demo/demo-master/` | @spicybutter; drafts to be generated later |
| Demo mode in the node firmware: show demo data from `intellithings/demo/<node_id>/sensors`; the hybrid node | `software/firmware/` | Software (@LukeTuthill) |
| Demo or hybrid fields on the DWIN UI, if needed | VP map ([`docs/interfaces/display-vp-map.md`](../docs/interfaces/display-vp-map.md)) | @rakkicow (PM) |
| HA demo entities for the whole home; automations that trigger the agent; 3D dashboard | `ai/` | AI (@ProgrammingJohn) |

## Open questions

- **Timing** (open discussion, needs experiments): the interval between scenarios, the roll
  length (about 4 s) and the fixed time before the loop resumes after a roll.
- **The entity list for the whole home,** each entity's caps, and how values are spread within
  them.
- **How HA switches the agent to the demo entities** in demo mode (exposure to Assist, a demo
  on/off helper).
- **When the agent runs in demo mode:** which HA automations/helpers trigger it. **Open
  discussion,** pending how the automations perform.
- **The hybrid node** (TODO above).
- **Buzzer signal pin:** to be found by testing the buzzer's PCB (see
  [`demo-master/README.md`](demo-master/README.md#game-buzzer-wiring)).
