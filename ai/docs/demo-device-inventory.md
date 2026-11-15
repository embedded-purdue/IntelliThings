# Demo device inventory

## Room-topic payloads

Real-node `intellithings/<node_id>/sensors` payloads still include `node_id`. This replaces the earlier whole-house 

below are all of the proposed demo topics with their expected payloads

`Device / household states` are to be considered in the emulator program so that the AI is acutally making changes to the home enviornment

## `intellithings/demo/master_bedroom`

| JSON key | Type | Example value | Meaning / allowed values |
|---|---|---|---|
| **Device / household states** | | | |
| `light_on` | boolean | `false` | `true` / `false` |
| `fan_on` | boolean | `false` | `true` / `false` |
| `ac_mode` | string | `"off"` | `"off"` / `"cool"` |
| `ac_target_temperature_c` | number | `22.0` | °C; target limits to be agreed |
| `blinds_position_pct` | integer | `0` | 0–100; 0 closed, 100 open |
| `humidifier_on` | boolean | `false` | `true` / `false` |
| `humidifier_target_humidity_pct` | number | `45.0` | %RH; target limits to be agreed |
| `window_open` | boolean | `false` | `true` / `false` |
| **Sensor readings** | | | |
| `temperature_c` | number | `22.4` | °C |
| `humidity_pct` | number | `41.2` | %RH |
| `voc_index` | integer | `112` | VOC index |
| `co2_ppm` | integer | `684` | ppm |
| `lux` | number | `312.5` | lux |
| `distance_mm` | integer | `1180` | mm |
| `pm1_0_ugm3` | integer | `3` | µg/m³ |
| `pm2_5_ugm3` | integer | `5` | µg/m³ |
| `pm10_ugm3` | integer | `7` | µg/m³ |
| `presence` | boolean | `true` | Room occupied |
| **Metadata** | | | |
| `ts` | string | `"2026-11-08T12:00:00-05:00"` | ISO 8601 with timezone; omit until clock syncs |

## `intellithings/demo/second_bedroom`

| JSON key | Type | Example value | Meaning / allowed values |
|---|---|---|---|
| **Device / household states** | | | |
| `light_on` | boolean | `false` | `true` / `false` |
| `fan_on` | boolean | `false` | `true` / `false` |
| `ac_mode` | string | `"off"` | `"off"` / `"cool"` |
| `ac_target_temperature_c` | number | `22.0` | °C; target limits to be agreed |
| `window_open` | boolean | `false` | `true` / `false` |
| **Sensor readings** | | | |
| `temperature_c` | number | `22.4` | °C |
| `humidity_pct` | number | `41.2` | %RH |
| `voc_index` | integer | `112` | VOC index |
| `co2_ppm` | integer | `684` | ppm |
| `lux` | number | `312.5` | lux |
| `distance_mm` | integer | `1180` | mm |
| `pm1_0_ugm3` | integer | `3` | µg/m³ |
| `pm2_5_ugm3` | integer | `5` | µg/m³ |
| `pm10_ugm3` | integer | `7` | µg/m³ |
| `presence` | boolean | `true` | Room occupied |
| **Metadata** | | | |
| `ts` | string | `"2026-11-08T12:00:00-05:00"` | ISO 8601 with timezone; omit until clock syncs |

## `intellithings/demo/kitchen`

| JSON key | Type | Example value | Meaning / allowed values |
|---|---|---|---|
| **Device / household states** | | | |
| `light_on` | boolean | `false` | `true` / `false` |
| `fan_on` | boolean | `false` | `true` / `false` |
| `blinds_position_pct` | integer | `0` | 0–100; 0 closed, 100 open |
| `purifier_on` | boolean | `false` | `true` / `false` |
| `window_open` | boolean | `false` | `true` / `false` |
| `oven_on` | boolean | `false` | `true` / `false` |
| `microwave_on` | boolean | `false` | `true` / `false` |
| **Sensor readings** | | | |
| `temperature_c` | number | `22.4` | °C |
| `humidity_pct` | number | `41.2` | %RH |
| `voc_index` | integer | `112` | VOC index |
| `co2_ppm` | integer | `684` | ppm |
| `lux` | number | `312.5` | lux |
| `distance_mm` | integer | `1180` | mm |
| `pm1_0_ugm3` | integer | `3` | µg/m³ |
| `pm2_5_ugm3` | integer | `5` | µg/m³ |
| `pm10_ugm3` | integer | `7` | µg/m³ |
| `presence` | boolean | `true` | Room occupied |
| **Metadata** | | | |
| `ts` | string | `"2026-11-08T12:00:00-05:00"` | ISO 8601 with timezone; omit until clock syncs |

## `intellithings/demo/living_room`

| JSON key | Type | Example value | Meaning / allowed values |
|---|---|---|---|
| **Device / household states** | | | |
| `light_on` | boolean | `false` | `true` / `false` |
| `fan_on` | boolean | `false` | `true` / `false` |
| `ac_mode` | string | `"off"` | `"off"` / `"cool"` |
| `ac_target_temperature_c` | number | `22.0` | °C; target limits to be agreed |
| `tv_on` | boolean | `false` | `true` / `false` |
| `blinds_position_pct` | integer | `0` | 0–100; 0 closed, 100 open |
| `vacuum_state` | string | `"docked"` | `"docked"` / `"cleaning"` / `"returning"` |
| `window_open` | boolean | `false` | `true` / `false` |
| **Sensor readings** | | | |
| `temperature_c` | number | `22.4` | °C |
| `humidity_pct` | number | `41.2` | %RH |
| `voc_index` | integer | `112` | VOC index |
| `co2_ppm` | integer | `684` | ppm |
| `lux` | number | `312.5` | lux |
| `distance_mm` | integer | `1180` | mm |
| `pm1_0_ugm3` | integer | `3` | µg/m³ |
| `pm2_5_ugm3` | integer | `5` | µg/m³ |
| `pm10_ugm3` | integer | `7` | µg/m³ |
| `presence` | boolean | `true` | Room occupied |
| **Metadata** | | | |
| `ts` | string | `"2026-11-08T12:00:00-05:00"` | ISO 8601 with timezone; omit until clock syncs |

## `intellithings/demo/bathroom`

| JSON key | Type | Example value | Meaning / allowed values |
|---|---|---|---|
| **Device / household states** | | | |
| `light_on` | boolean | `false` | `true` / `false` |
| `exhaust_fan_on` | boolean | `false` | `true` / `false` |
| **Sensor readings** | | | |
| `temperature_c` | number | `22.4` | °C |
| `humidity_pct` | number | `41.2` | %RH |
| `voc_index` | integer | `112` | VOC index |
| `co2_ppm` | integer | `684` | ppm |
| `lux` | number | `312.5` | lux |
| `distance_mm` | integer | `1180` | mm |
| `pm1_0_ugm3` | integer | `3` | µg/m³ |
| `pm2_5_ugm3` | integer | `5` | µg/m³ |
| `pm10_ugm3` | integer | `7` | µg/m³ |
| `presence` | boolean | `true` | Room occupied |
| **Metadata** | | | |
| `ts` | string | `"2026-11-08T12:00:00-05:00"` | ISO 8601 with timezone; omit until clock syncs |

## `intellithings/demo/garage`

| JSON key | Type | Example value | Meaning / allowed values |
|---|---|---|---|
| **Device / household states** | | | |
| `light_on` | boolean | `false` | `true` / `false` |
| `fan_on` | boolean | `false` | `true` / `false` |
| `door_open` | boolean | `false` | `true` / `false` |
| `car` | string | `"parked"` | `"away"` / `"parked"` / `"running"` |
| **Sensor readings** | | | |
| `temperature_c` | number | `22.4` | °C |
| `humidity_pct` | number | `41.2` | %RH |
| `voc_index` | integer | `112` | VOC index |
| `co2_ppm` | integer | `684` | ppm |
| `lux` | number | `312.5` | lux |
| `distance_mm` | integer | `1180` | mm |
| `pm1_0_ugm3` | integer | `3` | µg/m³ |
| `pm2_5_ugm3` | integer | `5` | µg/m³ |
| `pm10_ugm3` | integer | `7` | µg/m³ |
| `presence` | boolean | `true` | Room occupied |
| **Metadata** | | | |
| `ts` | string | `"2026-11-08T12:00:00-05:00"` | ISO 8601 with timezone; omit until clock syncs |

## `intellithings/demo/office`

| JSON key | Type | Example value | Meaning / allowed values |
|---|---|---|---|
| **Device / household states** | | | |
| `light_on` | boolean | `false` | `true` / `false` |
| `fan_on` | boolean | `false` | `true` / `false` |
| **Sensor readings** | | | |
| `temperature_c` | number | `22.4` | °C |
| `humidity_pct` | number | `41.2` | %RH |
| `voc_index` | integer | `112` | VOC index |
| `co2_ppm` | integer | `684` | ppm |
| `lux` | number | `312.5` | lux |
| `distance_mm` | integer | `1180` | mm |
| `pm1_0_ugm3` | integer | `3` | µg/m³ |
| `pm2_5_ugm3` | integer | `5` | µg/m³ |
| `pm10_ugm3` | integer | `7` | µg/m³ |
| `presence` | boolean | `true` | Room occupied |
| **Metadata** | | | |
| `ts` | string | `"2026-11-08T12:00:00-05:00"` | ISO 8601 with timezone; omit until clock syncs |

## `intellithings/demo/laundry`

| JSON key | Type | Example value | Meaning / allowed values |
|---|---|---|---|
| **Device / household states** | | | |
| `light_on` | boolean | `false` | `true` / `false` |
| `fan_on` | boolean | `false` | `true` / `false` |
| `washer_on` | boolean | `false` | `true` / `false` |
| `dryer_on` | boolean | `false` | `true` / `false` |
| **Sensor readings** | | | |
| `temperature_c` | number | `22.4` | °C |
| `humidity_pct` | number | `41.2` | %RH |
| `voc_index` | integer | `112` | VOC index |
| `co2_ppm` | integer | `684` | ppm |
| `lux` | number | `312.5` | lux |
| `distance_mm` | integer | `1180` | mm |
| `pm1_0_ugm3` | integer | `3` | µg/m³ |
| `pm2_5_ugm3` | integer | `5` | µg/m³ |
| `pm10_ugm3` | integer | `7` | µg/m³ |
| `presence` | boolean | `true` | Room occupied |
| **Metadata** | | | |
| `ts` | string | `"2026-11-08T12:00:00-05:00"` | ISO 8601 with timezone; omit until clock syncs |

## `intellithings/demo/home`

| JSON key | Type | Example value | Meaning / allowed values |
|---|---|---|---|
| **Device / household states** | | | |
| `light_on` | boolean | `false` | `true` / `false` |
| `fan_on` | boolean | `false` | `true` / `false` |
| `front_door_locked` | boolean | `false` | `true` / `false` |
| `anyone_home` | boolean | `false` | `true` / `false` |
| `front_door_light_on` | boolean | `false` | `true` / `false` |
| `doorbell_motion` | boolean | `false` | `true` / `false` |
| **Metadata** | | | |
| `ts` | string | `"2026-11-08T12:00:00-05:00"` | ISO 8601 with timezone; omit until clock syncs |

## `intellithings/demo/home/doorbell`

Separate event topic: QoS 1, **not retained**. Publish once per press; consumers should deduplicate repeated `event_id` values.

| JSON key | Type | Example value | Meaning / allowed values |
|---|---|---|---|
| `event` | string | `"doorbell_pressed"` | Doorbell press event; not a room snapshot state |
| `event_id` | string | `"boot-7-press-12"` | Unique per press, including across reboots |
| `ts` | string | `"2026-11-08T12:00:00-05:00"` | ISO 8601 with timezone; omit until clock syncs |


## HA entity mapping

Status: **registered on the live HA (2026.9.3) on Nov 15** — all 126 entities exist and have live
values. Owner: Ryan (Seungmin Nam).

### What HA had before registration (Nov 15)

- **Areas:** only Living Room (`living_room`), Kitchen (`kitchen`) and Bedroom (`bedroom`).
- **Devices:** the three emulated nodes from `register_emulator.py` ("IntelliThings Kitchen",
  "Bedroom", "Living Room"), each in its matching area with 10 MQTT sensors
  (`sensor.intellithings_emu_<room>_<field>`, `binary_sensor.intellithings_emu_<room>_presence`).
- **Demo entities:** none. Nothing subscribes to `intellithings/demo/*` yet.
- **Exposed to Assist (what the agent sees through MCP):** temperature, humidity, CO₂, PM2.5 and
  PM10 for each of the three rooms, `input_boolean.cooling_actuator`, and the shopping list.
  Presence, VOC, lux, distance and PM1 are not exposed.
- **Other helpers:** `input_boolean.bedroom_light` exists but has no area, isn't exposed and isn't
  in `allowed_actuators.json`, so the agent can't see it.

### Naming and device grouping

Follows `register_emulator.py`'s existing pattern (`intellithings_<node>_<field>`), with a
`demo` segment inserted so demo and real entities never collide, per the open requirement above
to keep demo and real context distinguishable:

- **unique_id / entity_id:** `intellithings_demo_<room>_<field>` (e.g.
  `sensor.intellithings_demo_kitchen_temperature_c`, `binary_sensor.intellithings_demo_kitchen_light_on`)
- **One HA device per room:** identifiers `intellithings_demo_<room>`, name `IntelliThings Demo
  <Room>`, assigned to the room's existing area, the same way the emulated node devices already
  are.
- **The `intellithings_demo_*` IDs don't clash** with the existing `intellithings_emu_*` ones.
- **Discovery topic:** `homeassistant/<component>/intellithings_demo_<room>_<field>/config`,
  same mechanism `register_emulator.py` already uses, just pointed at the new per-room topics
  (`intellithings/demo/<room>`) instead of the old aggregate `intellithings/emulator/state`.
  No node-selection template needed this time — each room already has its own topic, so
  `value_template` is a direct `value_json.<field>` instead of the `selectattr` filter the
  emulator script needs for its single shared topic.

### Room topic → HA area

| Demo topic | HA area | Status |
|---|---|---|
| `master_bedroom` | Bedroom (`bedroom`) | **Exists.** Map to it, don't create "Master Bedroom": the pipeline already routes room messages to `bedroom` |
| `kitchen` | Kitchen (`kitchen`) | Exists |
| `living_room` | Living Room (`living_room`) | Exists |
| `second_bedroom` | Second Bedroom | New |
| `bathroom` | Bathroom | New |
| `garage` | Garage | New |
| `office` | Office | New |
| `laundry` | Laundry | New |
| `home` | *(none)* | Whole-house state, no area |

Entity IDs keep the topic name (`intellithings_demo_master_bedroom_*`); only the area maps to
`bedroom`.

### Sensor readings (all 8 rooms) — unchanged from the emulator's existing metrics

Reuse `register_emulator.py`'s `METRICS` table as-is (`temperature_c`, `humidity_pct`,
`voc_index`, `co2_ppm`, `lux`, `distance_mm`, `pm1_0_ugm3`, `pm2_5_ugm3`, `pm10_ugm3`,
`presence`) — same fields, same domains/units/device classes, just re-pointed at
`intellithings/demo/<room>` per room instead of the aggregate topic.

**Temperature units:** payloads stay in °C, as `docs/interfaces/mqtt.md` requires. Temperature
entities (room temperatures and AC targets) set `suggested_unit_of_measurement: °F`, so HA shows
°F to people and to the agent regardless of HA's unit system. Temperature, humidity and lux show
one decimal, matching the emulator entities.

### Device / household state fields — new mapping

| Field | Domain | `device_class` | Unit | Notes |
|---|---|---|---|---|
| `light_on`, `fan_on`, `purifier_on`, `oven_on`, `microwave_on`, `tv_on`, `exhaust_fan_on`, `washer_on`, `dryer_on`, `humidifier_on`, `front_door_light_on` | `binary_sensor` | *(none)* | — | Read-only mirror of the demo master's state. No HA device_class fits "is this appliance on" cleanly — don't force one |
| `anyone_home` | `binary_sensor` | `occupancy` | — | Same convention as per-room `presence` |
| `doorbell_motion` | `binary_sensor` | `motion` | — | |
| `window_open` | `binary_sensor` | `window` | — | |
| `door_open` (garage) | `binary_sensor` | `garage_door` | — | |
| `front_door_locked` | `binary_sensor` | `lock` | — | **Gotcha:** HA's `lock` binary_sensor device_class reports **`on` = unlocked/open**. `front_door_locked: true` needs an *inverted* `value_template` (`{{ 'off' if value_json.front_door_locked else 'on' }}`), not a direct passthrough — easy to get backwards |
| `ac_mode`, `car`, `vacuum_state` | `sensor` | *(none, text state)* | — | Enum values as-is (`"off"`/`"cool"`, `"away"`/`"parked"`/`"running"`, `"docked"`/`"cleaning"`/`"returning"`) |
| `ac_target_temperature_c` | `sensor` | `temperature` | `°C` | `state_class: measurement` |
| `humidifier_target_humidity_pct` | `sensor` | `humidity` | `%` | `state_class: measurement` |
| `blinds_position_pct` | `sensor` | *(none)* | `%` | `state_class: measurement` |

### `home` and `home/doorbell` — handled differently

- `home`'s fields (`front_door_locked`, `anyone_home`, `front_door_light_on`, `light_on`,
  `fan_on`, `doorbell_motion`) follow the same per-field rules above, just under
  `intellithings_demo_home_<field>` instead of a room name.
- `home/doorbell` is a **momentary event**, not room state (not retained, one message per press).
  Don't give it a persistent discovery entity the same way — either (a) an HA automation with an
  `mqtt` trigger straight on that topic, deduplicating by `event_id`, or (b) if the 3D view wants
  something to light up briefly, a `binary_sensor` with `off_delay` (e.g. 5s) driven by
  `event == "doorbell_pressed"`.

### Exposing to Assist — decided Nov 15

Registering an entity doesn't make it visible to the agent; it also has to be exposed to Assist.
The AI lead approved leaving HA's "Expose new entities" setting on, so HA exposed the entity
types it exposes by default: 51 of the 126 demo entities (temperatures, humidity, CO₂, PM2.5,
PM10, AC and humidifier targets, windows, the garage door, the front-door lock, doorbell motion).
The agent now sees 68 entities instead of 17. Its first summary after registration already
mentioned open windows, which only the demo entities report.

Still worth knowing:

- **Presence and `anyone_home` are not exposed** (HA doesn't auto-expose occupancy sensors), so
  the agent still can't reason about who is in a room. Expose them by hand if it should.
- **Demo and emulator readings share the same areas**, so the agent can see two kitchen
  temperatures at once. That's expected until demo/real context switching exists.

### What this does *not* solve

- **No command/write path yet.** These are all read-only mirrors of the demo master's state.
  Tapping a light in Zircon3D won't change anything until a command topic exists (see the MQTT
  contract sync item below) — that's separate work, not blocked by this mapping.
- **Demo vs. real context switching** (which the agent sees when) is still open — this mapping
  only defines the demo-side entities, it doesn't decide when HA hands the agent demo data vs.
  real data.

### Registering the entities

`ai/deployment/home_assistant/register_demo_rooms.py` publishes these discovery configs (126 in
total: 46 device states, 80 sensor readings). It reuses `register_emulator.py`'s `METRICS` for
the sensor fields and adds each entity's `ts` as an attribute. Run it from the repo with the
virtualenv and `.env` set up:

```bash
# Preview without publishing:
python ai/deployment/home_assistant/register_demo_rooms.py
# Start small: one room, the four fields the dashboard needs first
python ai/deployment/home_assistant/register_demo_rooms.py --rooms kitchen \
  --fields temperature_c humidity_pct presence light_on --publish
# Everything:
python ai/deployment/home_assistant/register_demo_rooms.py --publish
# Undo (deletes the same entities from HA):
python ai/deployment/home_assistant/register_demo_rooms.py --remove
```

`--remove` takes the same `--rooms`/`--fields` filters. Re-running `--publish` updates the
existing entities in place. The emulator already publishes all 9 demo topics, so every entity had
live values as soon as it was registered.

## Integration still to settle

- MQTT contract sync, command topics, target limits, publish/inference timing, and scenario roll/hold durations.
- Switching the agent between demo and real context. (Room-to-area mapping is settled above: `master_bedroom` → `bedroom`.)
- Simulator feedback: apply AI commands, reflect their effects, and prevent later snapshots from silently overwriting them; extend the agent beyond its current single turn-on action.
- Buzzer GPIO and hybrid-display node selection; verify press → snapshot → HA → AI action → simulation/dashboard/display end to end.

Existing contracts: [MQTT](../../docs/interfaces/mqtt.md), [HA entities](../../docs/interfaces/ha-entities.md). Existing implementation and scenario values: [emulator dataset](../emulator/esp32_c3_emulator/demo/demo_home.json). Dashboard implementation: [floor-plan plan](3d-floorplan-plan.md).
