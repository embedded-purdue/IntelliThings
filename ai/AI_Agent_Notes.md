# IntelliThings — AI Agent & Cloud Hosting Notes
*AI subteam (Cloud AI Lead @ProgrammingJohn · Local AI Lead undecided) | Last updated Sept 26, 2026*

> Working notes and design reasoning for the AI side of IntelliThings. The current decisions are summarized in `docs/Project_Guideline.md` §5–6; how the AI team splits and reviews work is in `docs/Collaboration_Guidelines.md` §3. If anything here disagrees with the Project Guideline, the guideline wins.

---

## 1. Decision: a lightweight custom agent harness, not a third-party agent framework

Two market products were evaluated:

- **OpenClaw** — self-hosted, general-purpose autonomous agent runtime (long-running Node.js daemon). Built for broad tasks: browser automation, coding/shell execution, messaging integrations. Extends through its own "skills" format rather than MCP.
- **Hermes Agent** (Hermify) — hosted product built around persistent cross-session memory and a chat interface, with a documented pattern for connecting to Home Assistant's MCP Server via a long-lived access token.

**Why neither is adopted:**
- The requirement is narrow: receive a sensor snapshot from Home Assistant → call an LLM with that snapshot + HA's tools → let it act through HA's MCP Server → return a short explanation. That's a bounded tool-calling loop, not open-ended autonomy.
- A small harness you own is easier for a student team to debug and a stronger portfolio piece than configuring a third-party daemon or hosted product.
- It matches the project's differentiator vs. Xiaomi Miloco: open protocols, no lock-in. Depending on a hosted service or a bespoke skills format would reintroduce the lock-in the project avoids everywhere else.

---

## 2. How Home Assistant hands work to the agent

**HA automation → HTTPS POST → `/ha-event` on API Gateway.** An automation fires on meaningful changes (threshold crossings, presence changes, a periodic check), is throttled so the LLM isn't called on every reading, and sends a compact JSON snapshot (e.g. via `rest_command`).

**Why not HA's AI Task integration:** AI Task is a building block that runs against LLM integrations configured *inside* Home Assistant (`ai_task.generate_data`). It can't target our own Lambda harness. It's still handy for quick in-HA experiments, but the production path is the HTTP hand-off.

The `/ha-event` snapshot format is a shared contract between Local AI and Cloud AI, due **Oct 4** (written in `docs/interfaces/`).

---

## 3. The agent loop: OpenRouter + client-side MCP

**Model testing through OpenRouter.** OpenRouter gives one OpenAI-compatible API and one credit balance for many models (Claude, GPT, Gemini, open-weight models), so the team can compare models on the same prompts before committing. Credits purchased Sept 24 ($21.19). The final model is chosen from these tests (target: Nov 8 session).

**OpenRouter doesn't connect to MCP servers itself**, unlike the Claude/OpenAI APIs' built-in MCP connectors. So the harness is the **MCP client**:

```
/ha-event snapshot (or chat message)
  → fetch tool list from HA's MCP Server (via Nabu Casa URL)
  → convert MCP tools to OpenAI-style function definitions
  → send prompt + tools to the model (OpenRouter)
  → model returns tool calls → harness runs them against HA's MCP Server
  → send tool results back to the model → repeat until a final answer
  → publish the short explanation to the node's display (via HA → MQTT, retained)
```

This loop works with any provider, so it carries over unchanged whichever model wins.

**Compare models on** a shared set of standard scenarios (hot room + presence, rising CO₂, PM2.5 spike, empty room, nothing wrong):
- Correct tool, device and arguments
- **Not acting** when nothing needs doing
- Handling missing/bad sensor data and unavailable devices
- Short, clear explanations that fit the display's AI section
- Latency and cost per decision

The AI subteam's **emulator ESP32** (owner: @spicybutter, PM; 3 simulated nodes with scripted events) is a ready source of these scenarios.

---

## 4. Cloud hosting: AWS Lambda + API Gateway (confirmed)

Chosen over Render/Railway/Fly.io/Cloudflare Workers for its industry relevance. Pay-per-invocation is effectively free at this project's volume (Lambda free tier: 1M requests + 400,000 GB-s/month; API Gateway HTTP API ~$1.00 per million requests, 1M/month free for 12 months on a new account).

Two routes into the same function:
- `/ha-event` — snapshots from Home Assistant
- `/chat` — the chat bot's webhook

Cold starts add ~1–2 s — fine for automations and chat.

---

## 5. Chat interface — 🗳 TEAM DECISION: Discord or Telegram

A chat channel ("turn off all the lights") is a second trigger path into the *same* agent function, not a separate system. Both platforms work on Lambda with no always-on server:

| | **Discord** | **Telegram** |
|---|---|---|
| How messages reach Lambda | *Interactions Endpoint URL*: Discord POSTs each **slash command** (e.g. `/home turn off the lights`) to `/chat` | *Webhook* (`setWebhook`): Telegram POSTs **every message** sent to the bot to `/chat` |
| What users type | Slash commands only — reading ordinary chat messages needs Discord's always-on Gateway connection, which doesn't fit Lambda | Plain messages in a private chat or group — no command syntax |
| Response timing | **Must acknowledge within 3 s.** Return a deferred "thinking…" reply, run the agent asynchronously (self-invoke or second Lambda), then edit the message via the follow-up webhook | No 3-second rule. Run the agent, reply with `sendMessage`, then return HTTP 200 (Telegram redelivers failed/timed-out updates) |
| Request verification | Ed25519 signature on every request (`X-Signature-Ed25519`, `X-Signature-Timestamp`) — required to register the endpoint | Secret token set in `setWebhook`, checked via `X-Telegram-Bot-Api-Secret-Token` |
| Access control | Private channel or role | Known chat/user IDs only |
| Fit for the team | Team already works in Discord | Separate app; phone-first |
| Effort | Higher (async deferred reply + signature crypto) | Lower (one synchronous handler) |

The `/chat` request format is written once the platform is chosen (due Oct 25). Chat bot build: Oct 25 session.

---

## 6. Networking: Nabu Casa (decided)

HA runs on the Pi behind the home router. HA calling *out* to Lambda is easy. The agent calling *back into* HA's MCP Server needs a way in — **Home Assistant Cloud (Nabu Casa)** was chosen: a secure public URL with no port-forwarding, $6.50/month or $65/year. (Cloudflare Tunnel was the free alternative considered.) Set up by the Oct 18 session, when the MCP loop is tested end to end.

---

## 7. Security

- HA long-lived token, OpenRouter key, chat bot token and webhook secrets live in AWS Secrets Manager / Lambda environment config — **never in the repo**.
- Verify every chat request (Discord signature or Telegram secret token); limit the bot to known channels/users.
- Expose **only** the HA entities and services the agent needs, via HA's MCP exposure settings.

---

## 8. AI team timeline (from the Project Guideline)

| Session | AI work |
|---|---|
| **Sep 27** | Mosquitto + MQTT data into HA from the emulator ESP32; start the cloud pipeline (Lambda + API Gateway + OpenRouter) |
| Oct 4 | MQTT entities from the agreed schema; virtual devices; first OpenRouter prompt tests; `/ha-event` format fixed |
| Oct 18 | HA → Lambda hand-off; MCP client loop against HA's MCP Server; Nabu Casa; start switching to real rig data |
| Oct 25 | Chat bot; model comparison; decision logic |
| Nov 1 | Dashboard incl. 3D/floor-plan view; full-pipeline tests on the rigs |
| Nov 8 | Pick final model; dashboard polish |
| Nov 15 | Add the 3 final nodes to HA; test on assembled nodes |
| Dec 6 | Full-system verification, ECE SPARK prep |

---

## Sources

- [Home Assistant – AI Task integration](https://www.home-assistant.io/integrations/ai_task/) · [Model Context Protocol Server integration](https://www.home-assistant.io/integrations/mcp_server/)
- [Apidog – How to use MCP servers with OpenRouter](https://apidog.com/blog/use-mcp-servers-with-openrouter/)
- [Discord – Interactions / receiving and responding](https://discord.com/developers/docs/interactions/receiving-and-responding) · [OneUptime – Serverless Discord bot on AWS](https://oneuptime.com/blog/post/2026-02-12-build-a-serverless-discord-bot-on-aws/view) · [Telegram Bot API](https://core.telegram.org/bots/api)
- [Nabu Casa – Home Assistant Cloud pricing](https://www.nabucasa.com/pricing/)
- [AWS Lambda pricing](https://aws.amazon.com/lambda/pricing) · [API Gateway pricing](https://aws.amazon.com/api-gateway/pricing)
