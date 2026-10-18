# AI — AI Agent & Smart Home Subteam

Cloud AI Lead: **@ProgrammingJohn** · Local AI Lead: **undecided** · Emulator: **@spicybutter (PM)** · 6 people + PM

**Local AI owns the smart-home environment. Cloud AI owns the intelligence. The interface
between them belongs to both.** Design notes:
[`AI_Agent_Notes.md`](AI_Agent_Notes.md) ·
team split and review rules: [Collaboration Guidelines §3](../docs/Collaboration_Guidelines.md).

## Current local Python pipeline

The existing Pi runs Home Assistant Container and a separate Mosquitto container.
`main.py` collects fresh MCP context, sends it to OpenRouter, optionally executes
one configured `intent__HassTurnOn` call, saves a structured recommendation, and
updates the HA dashboard summary. The model gets a second request only when a tool
call needs to be followed by a final response. This local implementation runs on
a five-minute systemd timer; it does not use Lambda or HA AI Task.

Run from the repository root:

```bash
python3 -m venv .venv
.venv/bin/pip install -r ai/requirements.txt
test -e .env || cp .env.example .env
```

Set `HA_BASE_URL`, `HA_LONG_LIVED_TOKEN`, `OPENROUTER_API_KEY`, and
`OPENROUTER_MODEL` in `.env`. Exported variables take precedence. On the Pi,
`HA_BASE_URL=http://127.0.0.1:8123`; the current model is `openai/gpt-5.4-mini`.
Optional `HA_MCP_URL` overrides the MCP endpoint only.

Use the [MCP setup](probes/README.md#local-setup) to enable the HA MCP Server
integration and expose entities to Assist. Dashboard publishing requires the
`input_text.ai_summary_overall` helper with maximum length 255. The virtual
**Turn Cooling On** helper must also appear in MCP context and the configured
[actuator allowlist](llm/actuators.json) to be offered as a control.

```bash
# Collect a snapshot without LLM requests or actions:
.venv/bin/python ai/main.py --collect-only
# Full cycle, including possible configured control and dashboard update:
.venv/bin/python ai/main.py
# Offline regression tests:
.venv/bin/python -m unittest discover -s ai/tests -v
```

Runtime files default to `~/.local/state/intellithings/mcp/`. Read
[inference and controls](llm/README.md) for the prompt, response schema, status
files, and dashboard behavior; [scheduling](systemd/README.md) for the timer; and
[broker operations](mosquitto/README.md) for the existing Pi deployment.

The legacy discovery script in `mosquitto/` consumes the aggregate
`intellithings/emulator/state` topic. The ESP32 emulator added alongside this
pipeline uses the newer per-node topics in [the MQTT contract](../docs/interfaces/mqtt.md).
Those require matching HA entity configuration; merging these files does not
migrate an existing HA deployment or add Python summary publishing to `.../display`.

## Layout

```
ai/
├── main.py       Local Python pipeline entry point
├── pipeline/     Context collection and persistence
├── integrations/ Home Assistant MCP, OpenRouter, dashboard REST clients
├── llm/          Inference orchestration, response validation, actuator allowlist
├── prompts/      Editable system prompt
├── probes/       Interactive MCP inspection
├── tests/        Offline Python regression tests
├── systemd/      Service and five-minute timer
├── mosquitto/    Broker configuration and legacy emulator registration
├── emulator/     Emulator ESP32 — 3 simulated nodes (owner: @spicybutter, PM)
├── ha/           Planned Home Assistant configuration
└── cloud/        Planned Lambda harness and chat bot
```

## Target system flow

```
ESP32 nodes (or the emulator)
   │ MQTT
   ▼
Home Assistant ── automation ── HTTPS POST /ha-event ──► Lambda agent harness
                                                            │
                                                            ▼
                                                 LLM (via OpenRouter)
                                                            │ tool calls
                                                            ▼
HA MCP Server ◄──────── MCP (via Nabu Casa) ─────── harness runs them
   │
   ▼
Real / virtual devices → dashboard + AI message → node display (retained MQTT)

Discord or Telegram → API Gateway /chat → same Lambda → agent → HA MCP Server
```

## Target local deployment — Raspberry Pi 5 / Home Assistant

- **HAOS** with the **Mosquitto broker add-on**
- MQTT entities from each node's `.../sensors` JSON (one entity per field), with
  `.../status` as availability
- **Virtual devices** for what we don't own: thermostat, lights, fan, purifier, humidifier,
  dehumidifier, smart plug
- **Hand-off automation** — fires on meaningful changes (threshold crossings, presence,
  periodic check), **throttled** so the LLM isn't called on every reading, POSTs a compact
  snapshot to `/ha-event` (e.g. via `rest_command`)
- Publish AI messages to each node's **retained** `.../display` topic
- **MCP Server** integration, exposing only the entities/services the agent needs
- **Home Assistant Cloud (Nabu Casa)** so the cloud agent can reach the MCP Server
- Dashboard: per-node/room readings, device states, AI decision history, 3D/floor-plan view

Start from the **emulator ESP32** (see below). Disable its nodes in HA once real nodes
publish.

**Why not HA's AI Task?** It only runs against LLM integrations configured inside HA, so it
can't target our Lambda. Fine for quick in-HA experiments, not the production path.

## Emulator ESP32 — owner: @spicybutter (PM)

One LuatOS ESP32-C3 Core board (C on ESP-IDF v6.1) with no sensors attached, so HA and the
agent can be built before the Software subteam's firmware is ready
([Project Guideline §3.1a](../docs/Project_Guideline.md)). **User guide** (control panel,
worked examples for each agent test scenario, scripting):
[`emulator/README.md`](emulator/README.md).

- Publishes as **three nodes** — `emu-kitchen`, `emu-bedroom`, `emu-living-room` — with the
  **full sensor set** every **3 s**, plus `.../status` with its own Last Will per node
- Follows the **same MQTT contract** as the real nodes
  ([`docs/interfaces/mqtt.md`](../docs/interfaces/mqtt.md)), so nothing in HA or the agent
  changes when real nodes replace it
- Subscribes to each node's `.../display` topic and prints AI messages over serial — tests
  the feedback path end to end
- Stateful simulation: each room drifts like a real room, with occupancy, cooking, CO₂
  build-up and PM/VOC events
- **Control panel** at `http://intellithings-emu.local/`: one-click scenarios (cook, stuffy
  room, PM2.5 spike, hot & occupied, leave…), custom value ranges, system status, received AI
  messages and a test-message sender. The same commands work over MQTT on
  `intellithings/emulator/cmd`
- Until the MQTT schema is fixed on Oct 4, the emulator's JSON is the working draft
- Stays available after real nodes come online, for testing the agent and dashboard
  without hardware

## Target cloud deployment — AWS agent harness

- **Custom harness**, not an agent framework (OpenClaw and Hermes Agent were evaluated and
  rejected — see the AI Agent Notes §1).
- **AWS Lambda + API Gateway (HTTP API)**, two routes into one function: `/ha-event` and
  `/chat`. Python.
- **OpenRouter** for model testing — one API across Claude, GPT, Gemini and open-weight
  models. It doesn't speak MCP, so **the harness is the MCP client**: fetch HA's tools →
  convert to OpenAI-style functions → send with the prompt → run the model's tool calls
  against HA → return results → repeat until a final answer.
- Prompt: comfort targets, the per-node snapshot, rules for when **not** to act, and a short
  explanation for the display.
- **Chat bot — 🗳 team decision: Discord or Telegram.** Discord needs the 3-second deferred
  reply and Ed25519 signature check; Telegram needs a webhook secret token.

Don't wait for HA — start from **mock snapshots** in the agreed schema.

## Testing the agent

Keep a shared set of standard scenarios (hot room + presence, rising CO₂, PM2.5 spike,
empty room, nothing wrong) and score every model and prompt on:

- Correct tool, device and arguments
- **Not acting** when nothing is needed
- Handling missing/bad sensor data and unavailable devices
- Short, clear explanations that fit the display's AI section
- Latency and cost per decision

The emulator's scripted events are a ready source of scenarios.

## Contracts (in `docs/interfaces/`)

| Contract | Between | Due |
|---|---|---|
| MQTT topics + JSON schema | Software ↔ AI | Oct 4 |
| `/ha-event` snapshot format | Local AI ↔ Cloud AI | Oct 4 |
| AI message format (`text`, `ts`) + max length | AI ↔ Software | Oct 18 |
| HA entity names + exposed MCP tools | Local ↔ Cloud (shared with all) | Oct 18 |
| `/chat` request format | Cloud AI | Oct 25 |

Local AI owns *HA → snapshot format*; Cloud AI owns *snapshot → decision → MCP calls*.
**Both leads approve changes to the interface between them.**

## Security

- The local Python pipeline reads HA and OpenRouter credentials from the ignored
  root `.env`; `.env.example` contains placeholders only.

- HA long-lived token, OpenRouter key, chat bot token and webhook secrets live in AWS
  Secrets Manager / Lambda environment config — **never in the repo**. HA secrets go in
  `secrets.yaml` (gitignored), referenced with `!secret`.
- Verify every chat request and restrict the bot to known channels/users.
- Expose the minimum set of HA entities through MCP.

## Review

Local HA change → a Local AI member or lead (Cloud AI Lead or a PM while the Local lead is
undecided) · cloud agent change → a Cloud AI member or lead · cloud ↔ HA interface change →
**both leads**.

Branches: `ai/ha-mqtt-entities`, `ai/ha-dashboard`, `ai/mcp-config`, `ai/lambda-agent`,
`ai/openrouter-tests`, `ai/chat-bot`. See [`CONTRIBUTING.md`](../CONTRIBUTING.md).
