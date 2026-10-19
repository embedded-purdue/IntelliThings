# Home Assistant entities and MCP tools

> **Status: draft.** This lists what is **built and running on the project HA instance as of
> Oct 18**, as the starting point for the contract. To be agreed with the AI lead
> (@ProgrammingJohn); node entity names also need the Software lead (@LukeTuthill).
> HA runs as **HA Container** on the Pi (`192.168.1.2:8123`).

## Sensor entities (one device per node)

Created by MQTT discovery (`ai/deployment/home_assistant/register_emulator.py`):
- **One HA device per node,** named "IntelliThings Kitchen" / "Bedroom" / "Living Room", with a
  suggested area of the same name.
- **Ten entities per device,** one per sensor field.

| Pattern | Example |
|---|---|
| `sensor.intellithings_<node>_<field>` | `sensor.intellithings_emu_kitchen_temperature_c` |
| `binary_sensor.intellithings_<node>_presence` | `binary_sensor.intellithings_emu_kitchen_presence` |

- **`<node>`** is the node ID with `-` replaced by `_`.
- **`<field>`** is the MQTT field name ([`mqtt.md`](mqtt.md)): `temperature_c`, `humidity_pct`,
  `voc_index`, `co2_ppm`, `lux`, `distance_mm`, `pm1_0_ugm3`, `pm2_5_ugm3`, `pm10_ugm3`,
  `presence`.
- **Proposal for real nodes:** the same pattern with the MAC-derived node ID.

## AI output entities

| Entity | Source | Content |
|---|---|---|
| `sensor.kitchen_ai_summary` | MQTT `intellithings/emu-kitchen/display` | Room message (≤60 characters); `ts` attribute |
| `sensor.bedroom_ai_summary` | MQTT `intellithings/emu-bedroom/display` | Room message; `ts` attribute |
| `sensor.living_room_ai_summary` | MQTT `intellithings/emu-living-room/display` | Room message; `ts` attribute |
| `sensor.ai_summary_overall` | MQTT `intellithings/ai/summary` | Overall summary (≤255 characters); `ts` attribute |
| `sensor.ai_inference_status` | MQTT `intellithings/ai/inference_status` | `start thinking` / `stop thinking`; `ts` attribute |
| `input_text.ai_summary_overall` | Written by the pipeline (HA REST) | Overall summary, max length 255. Kept for older dashboard cards |

Setup and dashboard examples: `ai/deployment/home_assistant/README.md`.

## Entities the AI may control

Only entities in `ai/config/allowed_actuators.json` **and** exposed to Assist can be controlled.

| Entity | Friendly name | Notes |
|---|---|---|
| `input_boolean.cooling_actuator` | Turn Cooling On | Virtual flag only; doesn't affect the emulated temperature |

More virtual devices are planned (the AI team's to-do).

## MCP tools the agent uses

HA's **Model Context Protocol Server** integration is set up with the Assist API ("Control Home
Assistant"). The pipeline connects over HTTP at `/api/mcp` with a long-lived access token.

| Tool | Use |
|---|---|
| `homeassistant__GetLiveContext` | Read: state of every Assist-exposed entity, once per run |
| `intent__HassTurnOn` | Write: **narrowed by the pipeline** to the allowlisted names and domains above; at most **one call per run**; never area-wide |

The other tools HA advertises are discovered but not offered to the model. **Turning devices off
isn't possible yet** (no `HassTurnOff`).

## Open points

- Names for the extra virtual devices (lights, fan, purifier, blinds, doors, …) that the agent
  and the 3D floor plan will use.
- Discovery for the real nodes on their per-node topics ([`mqtt.md`](mqtt.md#home-assistant-discovery-homeassistant)).
- Whether to add a turn-off tool.
- **Demo mode** ([`demo/README.md`](../../demo/README.md)): entities for the **whole demo home**,
  inside and outside (e.g. kitchen, living room, master and second bedroom, garage, patio, garden,
  weather), covering **everything that can change or be controlled** (readings, lights, doors,
  the car, presence), each with a **low/high cap** for the demo master's random rolls. **In demo
  mode the agent sees only these demo entities**; how HA switches the agent over is still open.
