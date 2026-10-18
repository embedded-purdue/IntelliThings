# IntelliThings AI

The local Python agent reads Home Assistant sensors, requests summaries from
OpenRouter, optionally executes one allowed MCP turn-on action, and publishes
room summaries, the overall summary, and thinking status through MQTT. It also
updates the existing dashboard text helper. A systemd timer runs it every five minutes.

```text
main.py → agent/pipeline.py
           collect context → start thinking → infer → save → publish → stop thinking
```

## Find what you need

| Task | Start here |
|---|---|
| Follow one complete run | [`main.py`](main.py), then [`agent/pipeline.py`](agent/pipeline.py) |
| Understand model requests and tool interaction | [`agent/inference.py`](agent/inference.py) |
| Understand sensor collection | [`agent/context.py`](agent/context.py) |
| Change the prompt | [`prompts/home_status.txt`](prompts/home_status.txt) |
| Change summary fields or validation | [`agent/response.py`](agent/response.py) |
| Choose allowed actuators | [`config/allowed_actuators.json`](config/allowed_actuators.json) |
| Understand actuator discovery and execution | [`agent/controls.py`](agent/controls.py) |
| Route room summaries to displays | [`config/display_nodes.json`](config/display_nodes.json) |
| Investigate API or MQTT delivery | [`agent/integrations/`](agent/integrations/) |
| Follow summary publication and thinking status | [`agent/publishing.py`](agent/publishing.py) |
| Understand how runtime files are written | [`agent/storage.py`](agent/storage.py) |
| Configure HA entities and dashboard cards | [`deployment/home_assistant/README.md`](deployment/home_assistant/README.md) |
| Inspect HA state and exposed MCP tools | [`tools/README.md`](tools/README.md) |
| Install or inspect the timer | [`deployment/systemd/README.md`](deployment/systemd/README.md) |
| Operate the broker | [`deployment/mosquitto/README.md`](deployment/mosquitto/README.md) |
| Use the ESP32 emulator | [`emulator/README.md`](emulator/README.md) |

## Layout

```text
ai/
├── main.py                    CLI, environment loading, and overlap lock
├── agent/                     Application code
│   ├── pipeline.py            Run ordering, persistence, and failure decisions
│   ├── context.py             Context collection and snapshot records
│   ├── inference.py           Model/tool interaction and validated response
│   ├── controls.py            Actuator discovery and execution restrictions
│   ├── response.py            Response schema and saved-response loader
│   ├── publishing.py          Summary and thinking-status delivery results
│   ├── storage.py             Timestamps and atomic JSON writes
│   └── integrations/          Home Assistant MCP, OpenRouter, MQTT, helper clients
├── config/                    Editable actuator allowlist and display mapping
├── prompts/                   Editable model prompt
├── deployment/                Home Assistant, Mosquitto, and systemd setup
├── tools/                     Manual MCP inspection commands
├── tests/                     Offline regression tests
├── emulator/                  Existing ESP32 firmware and its guide
├── docs/                      Inference details, design notes, and future plans
└── requirements.txt
```

`pipeline.run_cycle()` coordinates collection and startup recovery;
`pipeline.run_inference()` shows the inference, save, and publish sequence.
The final thinking reset runs in `finally`, so failures and normal cancellation
still attempt it. `inference.infer()` returns validated summaries and their
record; it does not publish them. Controls keep their own action journal so a
tool call remains recorded even if the final response fails.

## Current local Python pipeline

Use Python 3.11 or later. Run commands from the repository root; the virtualenv
and ignored `.env` live there. The Pi uses Home Assistant Container and a separate
Mosquitto container.

```bash
python3 -m venv .venv
.venv/bin/pip install -r ai/requirements.txt
test -e .env || cp .env.example .env
```

Set `HA_BASE_URL`, `HA_LONG_LIVED_TOKEN`, `OPENROUTER_API_KEY`, and
`OPENROUTER_MODEL` in `.env`. Exported environment variables take precedence.
On the Pi, `HA_BASE_URL=http://127.0.0.1:8123`; the current model is
`openai/gpt-5.4-mini`. Optional `HA_MCP_URL` overrides the MCP endpoint only.
The agent uses `OPENROUTER_*`, not the root template's `LLM_*` variables; it uses
HA's configured MQTT connection rather than reading broker credentials.

In Home Assistant:

1. Add **Model Context Protocol Server** and select the Assist API / Control
   Home Assistant option.
2. Under **Settings → Voice assistants → Expose**, expose the required sensors
   and configured actuators to Assist.
3. Create a long-lived access token under your profile's security settings.
4. Connect the MQTT integration to the emulator's broker and register the
   [summary entities](deployment/home_assistant/README.md).

The existing helper publication requires `input_text.ai_summary_overall` with
maximum length 255. The configured **Turn Cooling On** helper must appear in MCP
context before it can be offered to the model. The model gets a second request
only when a tool call needs to be followed by the final summaries.

```bash
# Read sensors without model requests, actions, or MQTT publication:
.venv/bin/python ai/main.py --collect-only
# Run a full cycle:
.venv/bin/python ai/main.py
# Validate and print the latest successful summaries:
.venv/bin/python ai/agent/response.py ~/.local/state/intellithings/mcp/inference.json
# Run offline tests:
.venv/bin/python -m unittest discover -s ai/tests -v
```

Runtime files stay in `~/.local/state/intellithings/mcp/`. See
[outputs and failures](docs/inference.md#outputs-and-failures) for each file's
meaning; check status and timestamps before treating a saved result as current.

The existing emulator registration script uses the aggregate
`intellithings/emulator/state` topic. Its behavior is documented under
[broker setup](deployment/mosquitto/README.md#emulator-readings-in-home-assistant).
Room summaries use each node's `/display` topic; overall summary and thinking
status use `intellithings/ai/summary` and `intellithings/ai/inference_status`.

Future Lambda, HAOS, and chat-bot work is described in the
[roadmap](docs/roadmap.md) and [design notes](docs/architecture.md).
Team ownership and review rules are in the
[collaboration guidelines](../docs/Collaboration_Guidelines.md).
