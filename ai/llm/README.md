# OpenRouter handoff

OpenRouter is the next stage; collection currently makes no LLM requests.
`ai/main.py` is the pipeline entry point, with collection and persistence in
`ai/pipeline/context.py`. Interactive scripts in `ai/probes/` are diagnostic tools
and are not dependencies of the production pipeline.

## Configuration for the next stage

The root `.env.example` includes blank `OPENROUTER_API_KEY` and `OPENROUTER_MODEL`
settings. Put actual values in the ignored root `.env` when implementing the
connection. No model has been selected and these settings are not consumed yet.

## Input contract

The current `latest.json` contains:

```json
{
  "schema_version": 1,
  "fetched_at": "2026-09-27T19:42:03+00:00",
  "tool": "homeassistant__GetLiveContext",
  "context": "Live Context: ...",
  "mcp_result": {"content": [], "isError": false}
}
```

Use `context` as the model's home-state input and `fetched_at` as the observation
time. The raw MCP envelope is retained for debugging. Context includes only
Assist-exposed entities and does not recursively fetch things such as shopping
list items. Treat unknown and unavailable readings as missing data.

## Implementation boundary

1. Add an OpenRouter client in `ai/integrations/openrouter.py`, with explicit
   model selection, bounded requests, and credentials loaded from the environment.
2. Add prompt construction and response handling here in `ai/llm/`.
3. Extend `ai/main.py` to invoke inference after successful collection, using the
   newly collected snapshot. Skip inference when collection fails; do not silently
   send the previous snapshot as fresh data.
4. Save model results separately, tagged with the input snapshot timestamp and
   model ID. Keep collection status separate from inference failures so an LLM
   outage does not discard sensor data.
5. Revisit the systemd service's 90-second budget when adding the LLM request.

The first integration can produce status text. Calling HA tools to change devices
is a separate stage and is not wired into the collector.
