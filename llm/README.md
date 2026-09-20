# LLM — LLM Subteam

The cloud AI agent: prompt engineering, MCP wiring, spatial-awareness logic, and the
proactive status statements shown on the companion display.

## Scope

- Build and host the cloud AI agent
- Design the decision prompts — what the agent sees, how it's asked to reason
- MCP wiring against the Home Assistant MCP Server
- Spatial awareness logic across multiple sensor nodes
- Proactive, AI-generated status statements for the 7" display
- Evals — how we know a prompt change made things better, not just different

## Planned layout

```
llm/
├── agent/              # the agent itself
├── prompts/            # versioned prompt templates
├── mcp/                # MCP client config / tool definitions
└── evals/              # scenario fixtures + scoring
```

## The job, concretely

The agent receives environmental state from Home Assistant — temperature, humidity,
air quality, presence, light level, across multiple rooms — and decides what the home
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

The LLM API key goes in `.env`, which is gitignored. **Never** commit it, and don't buy
your own key — ask a lead.

## Background

- Model Context Protocol: https://modelcontextprotocol.io
- Home Assistant MCP Server integration
- Home Assistant AI Task
