----

# **Placeholder**

----

# AI — AI Agent & Smart Home Subteam

The Home Assistant hub on the Raspberry Pi 5, the cloud AI agent, and everything
between them.

This is the integration point where hardware, software, and the reasoning layer all
have to land.

## Scope

**Home Assistant / hub**
- Home Assistant Container on the Raspberry Pi 5
- MQTT broker — ingest telemetry from the sensor nodes
- Matter for local device interop
- Home Assistant automations and **AI Task** integration
- Virtual device fleet (represents a full-size home) + physical demo devices
- Real-time dashboard, including the 3D floor-plan view
- Network setup and security

**Cloud agent**
- Build and host the cloud AI agent
- Design the decision prompts — what the agent sees, how it's asked to reason
- Enable and secure the **Home Assistant MCP Server** so the agent can call HA as tools
- Spatial awareness logic across multiple sensor nodes
- Proactive, AI-generated status statements for the 7" display
- Evals — how we know a prompt change made things better, not just different

## Pipeline layout

```text
ai/
├── main.py                   # Run one pipeline cycle
├── pipeline/context.py       # Collect and atomically save live context
├── integrations/             # Home Assistant MCP and OpenRouter clients
├── llm/                      # Prompt construction and inference persistence
├── probes/                   # Interactive MCP discovery and live-state tools
├── systemd/                  # Boot startup and five-minute schedule
├── mosquitto/                # Broker configuration and emulator registration
└── requirements.txt          # Shared Python dependencies
```

The running flow is devices → Mosquitto → Home Assistant → MCP → `main.py` →
`latest.json` → OpenRouter → `inference.json` (recommendations only).

## Start here

Run a single collection and inference cycle from the repository root:

```bash
python3 -m venv .venv
.venv/bin/pip install -r ai/requirements.txt
# Create .env from .env.example only if one does not already exist.
.venv/bin/python ai/main.py
```

Set `HA_BASE_URL=http://127.0.0.1:8123` and `HA_LONG_LIVED_TOKEN` in the root
`.env` on this Pi. Exported environment variables override `.env`.
`HA_MCP_URL` can override the complete MCP endpoint. Set `OPENROUTER_API_KEY` and
`OPENROUTER_MODEL=openai/gpt-5.4-mini` for inference, or pass `--collect-only`.

The [systemd timer](systemd/README.md) runs `main.py` at boot and every five minutes.
It runs without login and opens a fresh MCP session each cycle. Output remains at
`~/.local/state/intellithings/mcp/latest.json`, with collection health in
`status.json`. Failures preserve the last successful snapshot; consumers must
check its UTC `fetched_at` timestamp. `--output-dir`, `--url`, and `--timeout`
can override the defaults.

See [OpenRouter inference](llm/README.md) for prompt editing, response files,
timeout settings, and failure handling.

For the MQTT broker, follow the [Mosquitto setup](mosquitto/README.md).

To inspect a running hub's MCP tools and live state, use the
[Python MCP probes](probes/README.md).

You don't need the Pi to begin. Run Home Assistant locally in Docker:

```bash
docker run -d --name homeassistant \
  --privileged --restart=unless-stopped \
  -e TZ=America/Indiana/Indianapolis \
  -v "$(pwd)/home-assistant:/config" \
  --network=host \
  ghcr.io/home-assistant/home-assistant:stable
```

http://localhost:8123

## MQTT topic schema

Agree on this early and write it down here — the software team has to publish to
whatever we decide, and changing it later means reflashing every node.

Suggested shape:

```
intellithings/<node-id>/sensor/<metric>      # telemetry, node → broker
intellithings/<node-id>/status               # online/offline (LWT)
intellithings/<node-id>/command/<target>     # commands, broker → node
```

## The agent's job, concretely

It receives environmental state from Home Assistant — temperature, humidity, air
quality, presence, light level, across multiple rooms — and decides what the home
should do about it. It calls back into Home Assistant through MCP tools to actuate
devices, and it writes short status statements for the display.

Two distinct outputs, two distinct prompt problems:

1. **Decisions** — must be correct, conservative, and explainable. A wrong actuation is
   worse than no actuation.
2. **Status statements** — must be brief, useful, and not annoying. This is the part
   users actually see.

## Evals matter more than they look like they do

Prompt changes are invisible until they regress something. Before tuning, build a set
of fixture scenarios — sensor states with known-correct responses — and score against
them. Otherwise "better" is just vibes.

Keep prompts versioned in `prompts/`. When you change one, say why in the commit.

## Secrets

- Home Assistant secrets go in `secrets.yaml`, referenced with `!secret`. That file is
  **gitignored** — commit a `secrets.yaml.example` with blank values instead.
- The LLM API key goes in `.env`, also gitignored. Don't buy your own key — ask a lead.
- Also gitignored: `.storage/`, the SQLite database, logs. Runtime state, not config.

## Note on risk

Nobody started this project having configured Home Assistant, and HA AI Task, MCP
Server, and Matter are the thinnest-documented parts of the stack. Expect to
experiment, and expect this layer to gate the demo.

**Stand the hub up in week 1. Not week 6.**

## Background

- Model Context Protocol: https://modelcontextprotocol.io
- Home Assistant MCP Server integration
- Home Assistant AI Task
