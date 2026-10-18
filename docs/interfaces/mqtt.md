# MQTT topics and payloads

> **Status: DRAFT — to be agreed Oct 4** by the Software and AI leads. Until then, this is
> the working draft that the emulator ESP32 (`ai/emulator/`) implements; the node firmware
> and Home Assistant should build against it. Changes need review from both leads.

Broker: Mosquitto add-on on the Home Assistant Pi, port **1883**.

## Node topics

Every sensor node — real or emulated — uses the same three topics under its node ID.

| Topic | Direction | QoS | Retained | Payload |
|---|---|---|---|---|
| `intellithings/<node_id>/sensors` | node → broker | 0 | no | Sensor snapshot JSON (below) |
| `intellithings/<node_id>/status` | node → broker | 1 | **yes** | `online` / `offline` |
| `intellithings/<node_id>/display` | HA → node | 1 | **yes** | AI message JSON (below) |

- **`status`** is the node's MQTT **Last Will** (`offline`, retained). The node publishes
  `online` (retained) every time it connects. Use it as the HA availability topic, so a
  dropped node shows as unavailable immediately.
- **`display`** is retained so a node that reboots re-shows the latest message. The node
  subscribes after every (re)connect.
- **`<node_id>`**: real nodes derive it from the chip's MAC (one firmware image for every
  node). The emulator uses `emu-kitchen`, `emu-bedroom` and `emu-living-room`; the `emu-`
  prefix always marks emulated data.

## Sensor snapshot — `…/sensors`

```json
{"node_id":"emu-kitchen","temperature_c":22.4,"humidity_pct":41.2,"voc_index":112,
 "co2_ppm":684,"lux":312.5,"distance_mm":1180,"pm1_0_ugm3":3,"pm2_5_ugm3":5,
 "pm10_ugm3":7,"presence":true,"ts":"2026-09-27T14:05:00-04:00"}
```

| Field | Type | Unit | Range | Source |
|---|---|---|---|---|
| `node_id` | string | — | — | same as in the topic |
| `temperature_c` | number, 1 decimal | °C | 0–50 | SHT41 |
| `humidity_pct` | number, 1 decimal | %RH | 0–100 | SHT41 |
| `voc_index` | integer | index | 1–500 (≈100 = typical) | SGP40 |
| `co2_ppm` | integer | ppm | 400–5000 | SCD41 |
| `lux` | number, 1 decimal | lux | 0–65535 | BH1750 |
| `distance_mm` | integer | mm | 30–1200 | VL53L0X |
| `pm1_0_ugm3` | integer | µg/m³ | 0–500 | PMS5003 |
| `pm2_5_ugm3` | integer | µg/m³ | 0–500 | PMS5003 |
| `pm10_ugm3` | integer | µg/m³ | 0–500 | PMS5003 |
| `presence` | boolean | — | — | LD2410C OUT pin |
| `ts` | string, ISO 8601 with offset | — | — | node clock (SNTP); **omitted until the clock has synced** |

- The SCD41 also measures temperature and humidity. They're **not** published, to avoid
  duplicate readings; `temperature_c` / `humidity_pct` always come from the SHT41.
- Every snapshot carries every field. Consumers should tolerate a missing `ts`.
- **Publish interval: not yet agreed for real nodes** (the firmware doc lists 5–15 s or
  on-change as open). The emulator publishes **every 3 s, even when values are unchanged**.

## AI message — `…/display`

```json
{"text":"CO₂ is elevated. Consider opening the window.","ts":"2026-10-18T14:05:00-04:00"}
```

- `text` (string, required): shown in the display's AI section until a new message arrives.
  Max length is part of the AI message contract (due Oct 18); the emulator accepts up to
  400 characters from its panel and truncates anything over 512 bytes.
- `ts` (string, optional): when the message was generated.
- Publishing an **empty retained payload** clears the topic.

## Reserved: `intellithings/emulator/#`

Used only by the emulator's control channel (see `ai/emulator/README.md`). Real nodes and
Home Assistant must not publish or depend on anything under it.

| Topic | Direction | Notes |
|---|---|---|
| `intellithings/emulator/status` | emulator → broker | `online` / `offline`, retained, Last Will |
| `intellithings/emulator/state` | emulator → broker | retained JSON status, every 3 s and after each command |
| `intellithings/emulator/cmd` | anyone → emulator | JSON command, **not retained** |
