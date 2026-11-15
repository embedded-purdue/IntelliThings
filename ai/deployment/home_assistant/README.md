# Home Assistant config

Home Assistant summary entities and dashboard setup for the local agent.
The legacy sensor discovery script is [`register_emulator.py`](register_emulator.py);
see its [setup instructions](../mosquitto/README.md#emulator-readings-in-home-assistant).
Secrets stay outside version control. See [the AI README](../../README.md).

## Room summaries and overall inference status

These MQTT sensors are registered in the project HA instance. They use the
message's `text` field as state (or `state` for inference status), with `ts` as
a timestamp attribute. MQTT discovery must remain enabled.

| Entity | Message topic |
|---|---|
| `sensor.kitchen_ai_summary` | `intellithings/emu-kitchen/display` |
| `sensor.bedroom_ai_summary` | `intellithings/emu-bedroom/display` |
| `sensor.living_room_ai_summary` | `intellithings/emu-living-room/display` |
| `sensor.bathroom_ai_summary` | `intellithings/ai/rooms/bathroom/summary` |
| `sensor.garage_ai_summary` | `intellithings/ai/rooms/garage/summary` |
| `sensor.laundry_ai_summary` | `intellithings/ai/rooms/laundry/summary` |
| `sensor.office_ai_summary` | `intellithings/ai/rooms/office/summary` |
| `sensor.second_bedroom_ai_summary` | `intellithings/ai/rooms/second_bedroom/summary` |
| `sensor.ai_summary_overall` | `intellithings/ai/summary` |
| `sensor.ai_inference_status` | `intellithings/ai/inference_status` |

Summary messages use `{"text":"...","ts":"<ISO8601>"}`. Room
summaries target 3–6 words, at most 60 characters, and are grounded in each room's
readings; missing readings should produce a message such as "Sensor data unavailable".
The overall summary keeps its existing 255-character limit. One model response
contains only the overall and room summaries. Executed tool calls are recorded
separately in the pipeline's action log; summaries mention relevant limitations
and failed actions.

The shared inference status uses
`{"state":"start thinking","ts":"<ISO8601>"}` and
`{"state":"stop thinking","ts":"<ISO8601>"}`. The pipeline, not the LLM,
sets it around inference and publication, including handled failures and startup recovery.
Stop means inference is no longer running, not that it succeeded. Timestamps let
dashboards detect a stale status after an abrupt crash or lost MQTT connection.
All summary and status topics use QoS 1 and retention. HA cards can all read the shared
status; emulator firmware would need an additional subscription to use it.

The existing `input_text.ai_summary_overall` helper continues to receive the
overall summary for older dashboard cards. New cards can use the MQTT sensors:

```yaml
type: markdown
content: |
  {% if is_state('sensor.ai_inference_status', 'start thinking') %}
  **Thinking…**
  {% endif %}
  {{ states('sensor.ai_summary_overall') }}

  **Kitchen:** {{ states('sensor.kitchen_ai_summary') }}
  **Bedroom:** {{ states('sensor.bedroom_ai_summary') }}
  **Living room:** {{ states('sensor.living_room_ai_summary') }}
  **Second bedroom:** {{ states('sensor.second_bedroom_ai_summary') }}
  **Bathroom:** {{ states('sensor.bathroom_ai_summary') }}
  **Garage:** {{ states('sensor.garage_ai_summary') }}
  **Laundry:** {{ states('sensor.laundry_ai_summary') }}
  **Office:** {{ states('sensor.office_ai_summary') }}

  {% set ts = state_attr('sensor.ai_summary_overall', 'ts') %}
  {% if ts %}
  Updated: {{ as_local(as_datetime(ts)).strftime('%d %b %Y, %H:%M') }}
  {% endif %}
```

The pipeline discovers rooms from Assist-exposed HA areas and publishes every
room to `intellithings/ai/rooms/<room_key>/summary`. Bedroom is the demo master
bedroom's HA area; Second Bedroom is a separate destination. Existing emulator
display topics continue to receive copies for their three configured rooms.

After a successful inference includes a new room, register its summary entity:

```bash
# Preview discovery configurations from the latest successful inference:
.venv/bin/python ai/deployment/home_assistant/register_summaries.py
# Register missing entities using retained MQTT discovery:
.venv/bin/python ai/deployment/home_assistant/register_summaries.py --publish
```

The script preserves existing `sensor.<room_key>_ai_summary` entities, including
the original three display subscriptions. It adds missing entities with a `ts`
attribute; retained room messages populate their states immediately. It does not
edit dashboards or expose the generated summaries to Assist. If an entity was
manually renamed, check its registry entry before rerunning to avoid another
summary entity for that room. Use `--inference-file` for a different state directory.
