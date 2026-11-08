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


## Integration still to settle

- MQTT contract sync, command topics, target limits, publish/inference timing, and scenario roll/hold durations.
- HA entity mappings and switching the agent between demo and real context; room names must map to the existing bedroom/kitchen/living-room display outputs.
- Simulator feedback: apply AI commands, reflect their effects, and prevent later snapshots from silently overwriting them; extend the agent beyond its current single turn-on action.
- Buzzer GPIO and hybrid-display node selection; verify press → snapshot → HA → AI action → simulation/dashboard/display end to end.

Existing contracts: [MQTT](../../docs/interfaces/mqtt.md), [HA entities](../../docs/interfaces/ha-entities.md). Existing implementation and scenario values: [emulator dataset](../emulator/esp32_c3_emulator/demo/demo_home.json). Dashboard implementation: [floor-plan plan](3d-floorplan-plan.md).
