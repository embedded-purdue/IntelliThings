# OpenRouter inference

`ai/main.py` collects fresh Home Assistant context, then sends `context` and
`fetched_at` to OpenRouter. It does not send the duplicate raw MCP envelope.
This stage saves recommendations only; it does not execute device-control tools.

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

No new dependencies are needed; the integration uses the existing `httpx` client.

## Structured recommendations and Python loader

The prompt requests JSON, and the API request enforces the JSON Schema generated
from `ai/llm/response.py`. The pipeline validates the response before saving it.
`inference.json` now has envelope `schema_version: 2`, retains the original JSON
string in `content`, and exposes its parsed fields in `recommendation`:

- `summary`: short overview.
- `changes`: proposed changes, each with `area`, `action`, numeric or null
  `target_value`, string or null `unit`, `reasoning`, and an `evidence` string list.
- `missing_information`: a list of missing inputs or capabilities.

Changes remain recommendations, not executed actions. Invalid JSON, missing fields,
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
