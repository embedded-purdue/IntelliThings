# OpenRouter inference

`ai/main.py` collects fresh Home Assistant context, then sends `context` and
`fetched_at` to OpenRouter. It does not send the duplicate raw MCP envelope.
This stage can execute one targeted `intent__HassTurnOn` MCP call per cycle,
then save a structured recommendation.

## Configuration

Set these in the repository root `.env` (never commit credentials):

```dotenv
OPENROUTER_API_KEY=your-key
OPENROUTER_MODEL=openai/gpt-5.4-mini
```

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

- `latest.json` and `status.json`: snapshot and collection status.
- `inference.json`: last successful model response, requested and returned model,
  input timestamp, generation timestamp, response ID, token usage, and prompt hash.
- `inference_status.json`: latest inference success, failure, or skip reason.

Writes are atomic and owner-only. Collection failure skips inference; the pipeline
passes the newly collected snapshot directly to inference rather than rereading
an old file. Inference failure preserves both the sensor snapshot and previous
successful inference, and returns a nonzero exit code. Check inference status and
`input_fetched_at` before displaying a saved response as current.

Requests use the OpenRouter chat completions endpoint, low reasoning effort,
a 45-second total inference deadline, and a 2048-token completion cap (including
reasoning tokens). There are no application retries. Empty, truncated, or malformed
responses are failures. The systemd unit allows 120 seconds for collection plus
inference; reinstall the unit and reload systemd after updating an existing setup.

Install `ai/requirements.txt`; PyYAML parses the HA context inventory.

## Structured recommendations and Python loader

The prompt requests JSON, and the API request enforces the JSON Schema generated
from `ai/llm/response.py`. The pipeline validates the response before saving it.
`inference.json` now has envelope `schema_version: 3`, retains the original JSON
string in `content`, and exposes its parsed fields in `recommendation`:

- `summary`: short overview.
- `changes`: proposed changes, each with `area`, `action`, numeric or null
  `target_value`, string or null `unit`, `reasoning`, and an `evidence` string list.
- `missing_information`: a list of missing inputs or capabilities.

The `changes` array contains recommendations; executed calls are recorded separately in `actions`. Invalid JSON, missing fields,
or incorrect field types mark inference as failed and preserve the last valid file.
Older prose responses cannot be loaded with the structured loader; run a new cycle.

Print the extracted fields from a saved response:

```bash
.venv/bin/python ai/llm/response.py ~/.local/state/intellithings/mcp/inference.json
```

From Python with `ai/` on the import path:

```python
from llm.response import load_response

recommendation = load_response('/home/intellithings/.local/state/intellithings/mcp/inference.json')
print(recommendation.summary)
for change in recommendation.changes:
    print(change.area, change.action, change.target_value, change.unit)
    print(change.reasoning, change.evidence)
print(recommendation.missing_information)
```

The loader validates saved content; callers should still check
`inference_status.json` and `input_fetched_at` for freshness.

## MCP actuator control

Each cycle discovers MCP tools and matches the fresh GetLiveContext inventory
against `ai/llm/actuators.json`. Only configured, uniquely named entities visible
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
if the final model response fails. A pending/unknown result does not prove the
action failed; check HA before retrying. `inference.json` includes actions and
per-request token usage. Its top-level usage is only the final model request.
Manual and timer cycles sharing an output directory cannot overlap.

HassTurnOn cannot edit `input_text` messages, set thermostat targets, or change
sensor readings. Those require separate supported controls. The overall AI summary helper is updated automatically after successful inference,
independently of LLM tool calls; room summary helpers are not updated. Exposing devices is a configuration
operation, not an MCP control action; no replacement for GetLiveContext was needed.

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
covers collection (60), inference (45), and dashboard publication (10).

This is an automatic publication step, not an additional LLM tool. Runtime
`actuators.json` includes `llm_tools`, the exact callable definitions supplied to
OpenRouter, alongside `advertised_tools`, the wider MCP server tool list.
