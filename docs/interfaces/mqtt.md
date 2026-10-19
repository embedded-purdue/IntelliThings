# MQTT topics and payloads

> **Status: agreed (Oct 4).** This file is **the single list of every MQTT topic** used in
> IntelliThings. Add a topic here before anyone publishes to it. Changes need review from the
> Software lead (@LukeTuthill) and the AI lead (@ProgrammingJohn).

**Broker:** Mosquitto in its own Docker container on the HA Pi, **`192.168.1.2:1883`**
(`ai/deployment/mosquitto/`):
- Login required; anonymous connections are off. Any logged-in user can use every topic.
- Plain MQTT, no TLS: for the team LAN only.
- `homeassistant.local` doesn't resolve on the team network, so use the IP.

## All topics

| Topic | Publisher → subscriber | QoS | Retained | Payload | Section |
|---|---|---|---|---|---|
| `intellithings/<node_id>/sensors` | node → HA | 0 | no | Sensor snapshot JSON | [Node topics](#node-topics) |
| `intellithings/<node_id>/status` | node → HA (Last Will) | 1 | **yes** | `online` / `offline` | [Node topics](#node-topics) |
| `intellithings/<node_id>/display` | AI pipeline (via HA) → node | 1 | **yes** | AI message JSON | [AI message](#ai-message-display) |
| `intellithings/ai/summary` | AI pipeline (via HA) → HA dashboard | 1 | **yes** | Overall summary JSON | [AI topics](#ai-topics-intellithingsai) |
| `intellithings/ai/inference_status` | AI pipeline (via HA) → HA dashboard | 1 | **yes** | Thinking status JSON | [AI topics](#ai-topics-intellithingsai) |
| `intellithings/emulator/status` | emulator → anyone | 1 | **yes** | `online` / `offline` | [Emulator](#reserved-intellithingsemulator) |
| `intellithings/emulator/state` | emulator → anyone | 0 | **yes** | Emulator status JSON | [Emulator](#reserved-intellithingsemulator) |
| `intellithings/emulator/cmd` | anyone → emulator | 1 | **no** | Command JSON | [Emulator](#reserved-intellithingsemulator) |
| `homeassistant/<component>/<unique_id>/config` | setup script → HA | 1 | **yes** | HA MQTT discovery config | [HA discovery](#home-assistant-discovery-homeassistant) |

**Reserved namespaces:** `intellithings/ai/#` and `intellithings/emulator/#`. A `<node_id>` must
never be `ai` or `emulator`.

## Node topics

Every sensor node, real or emulated, uses the same three topics under its node ID.

- **`status`** is the node's MQTT **Last Will** (`offline`, retained). The node publishes
  `online` (retained) every time it connects. HA uses it as the availability topic, so a dropped
  node shows as unavailable immediately.
- **`display`** is retained, so a node that reboots shows the latest message again. The node
  subscribes after every (re)connect.
- **`<node_id>`**: real nodes derive it from the chip's MAC (one firmware image for every node).
  The emulator uses `emu-kitchen`, `emu-bedroom` and `emu-living-room`; the `emu-` prefix
  always marks emulated data.
- **The three real nodes go in the kitchen, the bedroom and the living room.**

### Sensor snapshot: `…/sensors`

```json
{"node_id":"emu-kitchen","temperature_c":22.4,"humidity_pct":41.2,"voc_index":112,
 "co2_ppm":684,"lux":312.5,"distance_mm":1180,"pm1_0_ugm3":3,"pm2_5_ugm3":5,
 "pm10_ugm3":7,"presence":true,"ts":"2026-09-27T14:05:00-04:00"}
```

| Field | Type | Unit | Range | Source |
|---|---|---|---|---|
| `node_id` | string | — | — | same as in the topic |
| `temperature_c` | number, 1 decimal | **°C** | 0–50 | SHT41 |
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

- **Temperatures are always °C over MQTT.** Showing °F is a display setting only
  ([`display-vp-map.md`](display-vp-map.md)).
- The SCD41 also measures temperature and humidity. They're **not** published, to avoid
  duplicate readings; `temperature_c` and `humidity_pct` always come from the SHT41.
- Every snapshot carries every field. Consumers should tolerate a missing `ts`.
- **Publish interval: not yet agreed for real nodes.** The emulator publishes **every 3 s, even
  when values are unchanged**.

## AI message: `…/display`

```json
{"text":"Warm; ventilation recommended","ts":"2026-10-18T12:00:00+00:00"}
```

- **`text`** (string, required): shown in the display's AI section until a different text
  arrives (no timeout).
- **`ts`** (string, optional): when the message was generated, ISO 8601 with offset. The pipeline
  sends UTC (`+00:00`).
- **Publishing an empty retained payload clears the topic.**
- **Sender:** the AI pipeline publishes through HA's `mqtt.publish` service after each successful
  run. Node-to-room routing is in `ai/config/display_nodes.json`, which lists the emulator's IDs
  today. **Real node IDs have to be added there.**
- **Length:**
  - The pipeline currently sends **room messages of at most 60 characters** (3–6 words).
  - **How much the DWIN AI section can show is still being tested** ([`display-vp-map.md`](display-vp-map.md#ai-message-length-needs-testing)). The final limit goes here once it's measured.
  - The emulator accepts at most 400 bytes of text in a 512-byte payload; the pipeline keeps
    device payloads within that.

## AI topics: `intellithings/ai/#`

Published by the AI pipeline through HA after each run, for HA dashboard cards. **Nodes don't
subscribe to these yet.**

| Topic | Payload | Notes |
|---|---|---|
| `intellithings/ai/summary` | `{"text":"…","ts":"<ISO 8601>"}` | Overall summary, at most 255 characters |
| `intellithings/ai/inference_status` | `{"state":"start thinking","ts":"…"}` or `{"state":"stop thinking","ts":"…"}` | Set by the pipeline (not the model) around each run, including failures. "Stop" means not running, not "succeeded". Use `ts` to spot a stale flag after a crash |

> **Open discussion:** should the nodes subscribe to `intellithings/ai/inference_status` and
> show the display's "thinking" line? If so, which text should it show?

## Reserved: `intellithings/emulator/#`

Used only by the emulator ESP32's control channel (see `ai/emulator/README.md`). Real nodes
mustn't publish or depend on anything under it.

| Topic | Direction | Notes |
|---|---|---|
| `intellithings/emulator/status` | emulator → broker | `online` / `offline`, retained, Last Will |
| `intellithings/emulator/state` | emulator → broker | retained JSON status, every 3 s and after each command |
| `intellithings/emulator/cmd` | anyone → emulator | JSON command, **not retained**. A retained command is ignored on reconnect |

## Home Assistant discovery: `homeassistant/…`

HA creates MQTT entities from retained discovery configs on
`homeassistant/<component>/<unique_id>/config`.

- **`ai/deployment/home_assistant/register_emulator.py`** writes them for the emulator's 30
  sensor entities. **It's a temporary arrangement:** it reads the emulator's aggregate
  `intellithings/emulator/state` topic, not the per-node `…/sensors` topics. **Real nodes need
  discovery configs on their own `…/sensors` and `…/status` topics.**
- The AI summary sensors are also registered as MQTT entities; see
  [`ha-entities.md`](ha-entities.md).
