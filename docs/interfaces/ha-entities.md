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

More virtual devices are planned — see [Planned virtual devices](#planned-virtual-devices-for-the-3d-floor-plan--spark-demo) below.

## MCP tools the agent uses

HA's **Model Context Protocol Server** integration is set up with the Assist API ("Control Home
Assistant"). The pipeline connects over HTTP at `/api/mcp` with a long-lived access token.

| Tool | Use |
|---|---|
| `homeassistant__GetLiveContext` | Read: state of every Assist-exposed entity, once per run |
| `intent__HassTurnOn` | Write: **narrowed by the pipeline** to the allowlisted names and domains above; at most **one call per run**; never area-wide |

The other tools HA advertises are discovered but not offered to the model. **Turning devices off
isn't possible yet** (no `HassTurnOff`).

## Planned virtual devices (for the 3D floor plan & SPARK demo)

> **Status: proposal, not built.** Nothing in this section exists in HA yet. This resolves the
> "names for the extra virtual devices" open point above with a concrete list and a general
> recipe for adding any of them. Needs the AI lead's sign-off before anyone builds against it,
> same as the rest of this file.

**Why these specifically:** for the **ECE SPARK Challenge demo** we want a full house on the 3D
floor plan with the agent visibly acting on it, but we only have 3 real/emulated sensor nodes —
no real light, door, fan, vacuum, or car. Virtual devices fill the gap, and because the AI agent
controls the *same* entities through MCP that the 3D card displays, turning on the living-room fan
and seeing it animate in 3D is the same action, not two systems that happen to agree.

**Open dependency:** room/node placement is still an open 🗳 team decision (`docs/Project_Guideline.md`
§10). This table follows the proposal in `ai/docs/3d-floorplan-plan.md`: kitchen, living room,
bedroom (the 3 node rooms) + garage + entry. Rename if the decision lands differently — the
entities and process below don't change.

**Naming:** these are one-off room devices, not per-node sensor fields, so they follow the
simpler `<domain>.<room>_<thing>` pattern already used for the AI summary sensors above (not the
`intellithings_<node>_<field>` pattern used for raw per-node sensor data).

**Reconciled with `demo_home.json`.** An earlier version of this table was drawn up independently
of the emulator's demo-mode draft (`ai/emulator/esp32_c3_emulator/demo/demo_home.json`, 61 fields
across 9 areas) and disagreed with it in places (e.g. `car` as a boolean here vs. a 3-state enum
there). **`demo_home.json` wins** — it's the more developed, SPARK-specific draft (fill rules,
derive rules, scenario presets already worked out) — so the table below maps its existing fields
to concrete HA entities first, then adds the appliance types it doesn't have at all (fan, AC,
purifier, humidifier, TV, plug, vacuum — confirmed absent from every area in that file).

*Already defined in `demo_home.json` — this is the HA-entity mapping for it:*

| Area | Field in `demo_home.json` | Shown in 3D | Proposed entity | HA domain |
|---|---|---|---|---|
| Kitchen / Living room / Master bedroom | `light_on` | Light on/off | `light.kitchen_main`, `light.living_room_main`, `light.bedroom_main` | `light` |
| Kitchen / Living room / Master bedroom | `window_open` | Window open/closed | `binary_sensor.kitchen_window`, `binary_sensor.living_room_window`, `binary_sensor.bedroom_window` | `binary_sensor` (`device_class: window`) |
| Second bedroom | `light_on`, `window_open` | Light, window | `light.second_bedroom_main`, `binary_sensor.second_bedroom_window` | `light`, `binary_sensor` |
| Garage | `door_open` | Garage door open/closed | `cover.garage_door` | `cover` (`device_class: garage`) |
| Garage | `car` (enum: away / parked / running) | Car in/out, engine state | `sensor.garage_car` (not `binary_sensor` — it's 3-state) | `sensor` |
| Garage | `light_on` | Light on/off | `light.garage_main` | `light` |
| Patio | `door_open`, `light_on` | Door, light | `cover.patio_door`, `light.patio_main` | `cover`, `light` |
| Home | `front_door_locked` | Front door lock state | `lock.entry_front_door` | `lock` (not a door sensor — it's a lock) |
| Home | `anyone_home` | Whole-house occupancy | `binary_sensor.anyone_home` | `binary_sensor` (`device_class: occupancy`) |
| Garden | `sprinkler_on` | Sprinkler running | `switch.garden_sprinkler` | `switch` |
| Outside | `condition`, `temperature_c`, `wind_kph` | Weather, for sun/sky rendering | *(use HA's own `weather`/`sun.sun` if available; otherwise a `sensor` per field)* | `sensor` / `weather` |

*Not in `demo_home.json` — new, needs adding there too if it should appear in the booth demo:*

| Room/area | Shown in 3D | Proposed entity | HA domain | Backing helper | Agent-controllable today? |
|---|---|---|---|---|---|
| Kitchen / Bedroom / Living room | Curtains / blinds position | `cover.kitchen_blinds`, `cover.bedroom_blinds`, `cover.living_room_blinds` | `cover` | Template cover backed by `input_number` (0–100) | ❌ needs new harness code (position, not on/off) |
| Bedroom / Living room | Fan | `fan.bedroom_fan`, `fan.living_room_fan` | `fan` | Template fan backed by `input_boolean` (+ speed) | ✅ on/off only |
| Living room | AC / thermostat | `climate.living_room_ac` | `climate` | Virtual `climate` thermostat via helpers | ❌ needs new harness code (setpoint, not on/off) |
| Kitchen | Purifier | `switch.kitchen_purifier` | `switch` | Template switch backed by `input_boolean` | ✅ on/off only |
| Bedroom | Humidifier | `humidifier.bedroom_humidifier` | `humidifier` | Virtual `humidifier` via helpers | ❌ needs new harness code |
| Living room | TV | `switch.living_room_tv` | `switch` | Template switch backed by `input_boolean` | ✅ on/off only |
| Kitchen | Smart plug | `switch.kitchen_plug` | `switch` | Template switch backed by `input_boolean` | ✅ on/off only |
| Living room | Robot vacuum | `vacuum.living_room_vacuum` | `vacuum` | Template vacuum backed by `input_select` (docked / cleaning / returning) + a script that "cleans" for N minutes then returns | ❌ needs new harness code (state machine, not on/off) |
| *(built-in)* | Sun position, day/night | `sun.sun` | `sun` | Already in HA | n/a — read-only |

**Open question for the demo-mode owner (@spicybutter) and AI lead:** should the 7 new appliance
types in the second table be added to `demo_home.json` too (so they roll randomly like everything
else at the booth), or do they stay manually/script-driven since they're not really "sensed" data?
Either way, the entity IDs and domains above are the contract either mode reads from.

### How to add any virtual device (the general recipe)

Same sequence regardless of which row above you're building:

1. **Create the entity in Home Assistant itself** — Settings → Devices & services → Helpers for the
   backing `input_boolean`/`input_number`/`input_select`, then a template entity (UI or YAML under
   `ai/deployment/home_assistant/`) wrapping it with the right `device_class` and friendly name.
   Nothing in this repo creates HA entities automatically.
2. **Expose it to Assist** (Settings → Voice assistants → Expose, or the entity's own toggle). This
   is required for two independent reasons: it's what makes the entity show up in
   `GetLiveContext`, *and* what makes the MCP server accept tool calls against it.
3. **If the agent should act on it**, add it to `ai/config/allowed_actuators.json`: `name` must
   exactly match one of the entity's exposed friendly names as HA reports them, `domain` must
   match exactly. A mismatch doesn't error — it silently lands in `missing_actuators` instead of
   `visible_actuators` (`ai/agent/controls.py`).
4. **Deploy to the Pi** — `main.py` runs from a git checkout on the Pi via a systemd timer (every
   5 minutes) and re-reads `allowed_actuators.json` fresh each run, so a `git pull` on the Pi is
   enough. No restart needed for a config-only change.
5. **Check the harness supports the action type.** `controls.py` currently wires up only
   `intent__HassTurnOn` (binary on/off) — the ✅ rows above. A ❌ row (cover position, climate
   setpoint, vacuum state) needs new code in `ai/agent/controls.py` (a new tool schema plus an
   `execute()` branch) before the agent can act on it. Until then it can still be **shown** in 3D
   and driven by automations/scripts/manual toggles during the demo.
6. **Remember the one-action-per-cycle limit** — the harness executes at most one actuator call
   per 5-minute run. Plan which devices are genuinely agent-controlled for the demo vs. visually
   driven by the emulator/demo master instead.

## Open points

- Discovery for the real nodes on their per-node topics ([`mqtt.md`](mqtt.md#home-assistant-discovery-homeassistant)).
- Whether to add a turn-off tool.
- **Demo mode** ([`demo/README.md`](../../demo/README.md)): entities for the **whole demo home**,
  inside and outside (e.g. kitchen, living room, master and second bedroom, garage, patio, garden,
  weather), covering **everything that can change or be controlled** (readings, lights, doors,
  the car, presence), each with a **low/high cap** for the demo master's random rolls. **In demo
  mode the agent sees only these demo entities**; how HA switches the agent over is still open.
  - **Draft to start from:** the emulator's demo-mode stand-in (v2.1.0) uses 61 entities in 9
    areas. They are listed with their caps, normal ranges and options in
    `ai/emulator/esp32_c3_emulator/demo/demo_home.json`, and published on the experimental
    `intellithings/emulator/demo/home` topic ([`mqtt.md`](mqtt.md#reserved-intellithingsemulator)).
  - **Still needs the AI lead's review.**
