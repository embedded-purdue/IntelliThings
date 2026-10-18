# Home Assistant config

Local AI side: MQTT entities for every node, virtual devices, the throttled `/ha-event`
hand-off automation, MCP Server exposure and dashboards. No secrets — use `secrets.yaml`
(gitignored) with `!secret`. See [`../README.md`](../README.md).

## Room summaries and overall inference status

These MQTT sensors are registered in the project HA instance. They use the
message's `text` field as state (or `state` for inference status), with `ts` as
a timestamp attribute. MQTT discovery must remain enabled.

| Entity | Message topic |
|---|---|
| `sensor.kitchen_ai_summary` | `intellithings/emu-kitchen/display` |
| `sensor.bedroom_ai_summary` | `intellithings/emu-bedroom/display` |
| `sensor.living_room_ai_summary` | `intellithings/emu-living-room/display` |
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
All five topics use QoS 1 and retention. HA cards can all read the shared
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

  {% set ts = state_attr('sensor.ai_summary_overall', 'ts') %}
  {% if ts %}
  Updated: {{ as_local(as_datetime(ts)).strftime('%d %b %Y, %H:%M') }}
  {% endif %}
```
