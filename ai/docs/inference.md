# OpenRouter inference

`ai/main.py` collects fresh Home Assistant context, then sends `context` and
`fetched_at` to OpenRouter. It does not send the duplicate raw MCP envelope.
This stage can execute one targeted `intent__HassTurnOn` MCP call per cycle,
then save a structured recommendation.

Code entry points: [`agent/pipeline.py`](../agent/pipeline.py) coordinates stages
and saves the inference result; [`agent/inference.py`](../agent/inference.py)
handles model/tool interaction and returns validated summaries;
[`agent/publishing.py`](../agent/publishing.py) records output delivery;
[`agent/storage.py`](../agent/storage.py) provides timestamps and atomic writes.

## Configuration

Complete [local pipeline setup](../README.md#current-local-python-pipeline) first.
The model and credentials come from the root `.env` through `OPENROUTER_MODEL`
and `OPENROUTER_API_KEY`. All commands below run from the repository root.

Exported environment variables take precedence. Edit
[`../prompts/home_status.txt`](../prompts/home_status.txt) to change the system
prompt. It is loaded on every cycle, independently of the working directory.

```bash
.venv/bin/python ai/main.py
# Collect without an LLM request:
.venv/bin/python ai/main.py --collect-only
# Optional overrides:
.venv/bin/python ai/main.py --prompt-file ai/prompts/home_status.txt --llm-timeout 45 --max-tokens 2048
```

## Outputs and failures

The output directory defaults to `~/.local/state/intellithings/mcp/` and can be
changed with `--output-dir`:

| File | Meaning |
|---|---|
| `latest.json` | Last successful context snapshot, observation time, and raw MCP result |
| `status.json` | Latest collection success or failure |
| `inference.json` | Last successful model response, parsed recommendation, actions, timestamps, and usage |
| `inference_status.json` | Latest inference success, failure, or skip reason |
| `actuators.json` | Generated discovery inventory and `llm_tools`; not the static allowlist |
| `actions.json` | Latest discovery/execution cycle's action records; not permanent history |
| `dashboard_status.json` | Dashboard publication success, failure, or skip reason |
| `mqtt_status.json` | Per-topic MQTT publication results, truncation, or skip reason |
| `thinking_status.json` | Start/stop thinking publication results; startup reset until inference begins |

Writes are atomic and owner-only. Collection failure skips inference; the pipeline
passes the newly collected snapshot directly to inference rather than rereading
an old file. Inference failure preserves both the sensor snapshot and previous
successful inference, and returns a nonzero exit code. Check inference status and
`input_fetched_at` before displaying a saved response as current.

Requests use the OpenRouter chat completions endpoint, low reasoning effort,
a 45-second total inference deadline, and a 2048-token per-request completion cap
(including reasoning tokens). There are no application retries. Empty, truncated, or malformed
responses are failures. The systemd unit allows 120 seconds for collection, inference,
and parallel dashboard/MQTT publication; reinstall the unit and reload systemd after updating an existing setup.

Install `ai/requirements.txt`; PyYAML parses the HA context inventory.

## Structured recommendations and Python loader

The prompt requests JSON, and the API request enforces the JSON Schema generated
from `ai/agent/response.py`. The pipeline validates the response before saving it.
`inference.json` now has envelope `schema_version: 5`, retains the original JSON
string in `content`, and exposes its parsed fields in `recommendation`:

- `summary`: short overview, at most 255 characters.
- `room_summaries`: required `kitchen`, `bedroom`, and `living_room` strings, each
  nonempty and at most 60 characters. The prompt targets 3–6 words per room and
  uses "Sensor data unavailable" when that room has no usable readings.

The model returns only these two fields. Relevant missing data, unavailable
controls, and failed tool calls are described briefly in the summaries. Executed
calls are recorded separately by the pipeline in `actions`, alongside timestamps
and request usage. Invalid JSON, extra or missing fields,
or incorrect field types mark inference as failed and preserve the last valid file.
Older responses with `changes`/`missing_information` or without `room_summaries`
cannot be loaded with the updated structured loader; run a new cycle.
All summaries share a single final model
response; the existing optional tool-call round trip is unchanged.

Print the extracted fields from a saved response:

```bash
.venv/bin/python ai/agent/response.py ~/.local/state/intellithings/mcp/inference.json
```

From Python with `ai/` on the import path:

```python
from agent.response import load_response

recommendation = load_response('/home/intellithings/.local/state/intellithings/mcp/inference.json')
print(recommendation.summary)
print(recommendation.room_summaries.kitchen)
print(recommendation.room_summaries.bedroom)
print(recommendation.room_summaries.living_room)
```

The loader validates saved content; callers should still check
`inference_status.json` and `input_fetched_at` for freshness.

## MCP actuator control

Each cycle discovers MCP tools and matches the fresh GetLiveContext inventory
against [`allowed_actuators.json`](../config/allowed_actuators.json). This is the static
allowlist, distinct from runtime `actuators.json`. Only configured, uniquely named entities visible
in MCP context with on/off state can be targeted. The initial target is the
virtual `input_boolean.cooling_actuator` named **Turn Cooling On**. It is exposed
to Assist on the project HA instance. It is a flag, not a temperature setpoint;
no physical heating/cooling behavior is assumed.

To add a virtual actuator, expose it in HA under Settings → Voice assistants →
Expose, add its exact friendly name, domain, and purpose to the configuration,
and inspect runtime `actuators.json` after the next cycle. Missing entities are
reported there and sent to the LLM as missing rather than silently controllable.
If HA changes the context format, unrecognized entities are not offered as targets.

The model receives a narrowed `intent__HassTurnOn` schema with exact names and
domains. Broad area calls and other tools are rejected. Up to one call is executed,
its MCP result is sent back to the model, and a second request generates the final
JSON with tools disabled. The same total `--llm-timeout` budget covers discovery,
model requests, and the tool call. There are no action retries.

`actions.json` records the call before execution and its result afterward, even
if the final model response fails. Discovery resets this file each cycle, so it
is not a historical action log. A pending/unknown result does not prove the
action failed; check HA before retrying. `inference.json` includes actions and
per-request token usage. Its top-level usage is only the final model request.
Manual and timer cycles sharing an output directory cannot overlap.

HassTurnOn cannot edit `input_text` messages, set thermostat targets, or change
sensor readings. Those require separate supported controls. The overall AI summary
helper is updated automatically after successful inference, independently of LLM
tool calls; room summary helpers are not updated. Exposing devices is a
configuration operation, not an MCP control action.

## Dashboard summary

Every successful inference publishes `recommendation.summary` verbatim to
`input_text.ai_summary_overall` using HA's `input_text.set_value` service over
REST, then reads the helper to verify the value. The prompt and response schema
limit summary to 255 characters. Existing responses with longer summaries must
be regenerated before using the updated loader.

The pipeline uses `HA_BASE_URL` and `HA_LONG_LIVED_TOKEN` for this step, with a
10-second deadline. `dashboard_status.json` records publication success, failure,
or a skipped cycle. Collection-only and failed inference runs leave the previous
helper text intact. A dashboard failure returns a nonzero exit code while keeping
the successful inference and its status. The existing 120-second service budget
covers collection (60), inference including start status (45), parallel
dashboard/MQTT publication (10), and stop status (2).

This is an automatic publication step, not an additional LLM tool. Runtime
`actuators.json` includes `llm_tools`, the exact callable definitions supplied to
OpenRouter, alongside `advertised_tools`, the wider MCP server tool list.

## MQTT display summaries

After successful inference, the pipeline publishes `recommendation.summary` to
`intellithings/ai/summary`. Each room's short message goes to its own display
topic, using the node-to-room mapping in [`display_nodes.json`](../config/display_nodes.json).
This mapping is independent of Assist exposure and the actuator allowlist. The
default display topics match the ESP32 emulator's existing subscriptions:

```text
intellithings/emu-kitchen/display
intellithings/emu-bedroom/display
intellithings/emu-living-room/display
```

Each payload is JSON with the appropriate summary in `text` and the same `ts`
from `inference.json`'s `generated_at`, for example a kitchen display message:

```json
{"text":"Warm; ventilation recommended","ts":"2026-10-18T12:00:00+00:00"}
```

Publication calls HA's REST `mqtt.publish` service using `HA_BASE_URL` and
`HA_LONG_LIVED_TOKEN`. HA must have a working MQTT integration connected to the
emulator's broker. No additional Python dependency, broker credentials, Assist
exposure, or model request is required. QoS is 1 and messages are retained, so
subscribing devices receive the latest summary after reconnecting. Retained
summaries have no automatic expiry; use `ts` to judge freshness.

The emulator accepts at most 400 UTF-8 bytes of text and a 512-byte JSON payload.
Device payloads respect these limits, shortening text with an ellipsis if needed
and accounting for Unicode and JSON escaping. The overall MQTT summary is sent
verbatim, as is the dashboard helper. `mqtt_status.json` records the generation
timestamp and each topic's `truncated`, `payload_bytes`, and `ok`/`error`.
Success means HA accepted the publish service call;
it is not confirmation that a device displayed the message.

Dashboard and MQTT publication run independently and concurrently, with bounded
requests. All configured topics are attempted even if one fails. A publication
failure returns a nonzero pipeline exit code without discarding successful
inference or preventing the other output. Collection-only, collection failure,
and inference failure skip summary publication and leave retained summaries unchanged. The next
successful cycle publishes a fresh summary; there are no application retries.

To add or remove destinations, edit `display_nodes.json`; the next cycle reads it.
The file maps node IDs to `kitchen`, `bedroom`, or `living_room`; use `{}` to
disable device copies while still publishing the overall summary and status.
Removing a destination does not clear its previously retained message.

Run `.venv/bin/python ai/main.py`, then inspect
`~/.local/state/intellithings/mcp/mqtt_status.json`. On the emulator, check
**AI messages** at `http://intellithings-emu.local/` (or its LAN IP), the
`/api/messages` endpoint, or the `[AI MESSAGE]` serial output. Compare both text
and timestamp with the saved inference. Reconnecting a display subscriber should
deliver the retained message without another model run.

## Shared thinking status

The pipeline publishes retained JSON to `intellithings/ai/inference_status`:
`{"state":"start thinking","ts":"<ISO8601>"}` before inference and
`{"state":"stop thinking","ts":"<ISO8601>"}` after inference and output
publication. Python controls this state; it never requires a model request.
The final reset also runs on validation errors, model timeouts, publication
failures, and normal task cancellation. Each status request has a 2-second bound.

At the start of each full cycle, a stop/reset is sent concurrently with context
collection to recover a stale retained flag from a previous interrupted run.
`--collect-only` sends no MQTT messages. `thinking_status.json` records the
startup reset until inference starts, then the start/stop results for that
inference. Status failures do not prevent generation or summary delivery, but
a failed start or stop publication makes the cycle return a nonzero exit code.

Stop means the pipeline is no longer thinking, not that the inference succeeded;
check `inference_status.json` and summary timestamps for success/freshness.
A hard process kill or unreachable broker can leave an old flag until the next
successful reset; use the status timestamp to detect that case. All HA cards
can read `sensor.ai_inference_status`. Emulator firmware needs an additional
subscription to render the shared status; its room message subscriptions already work.
