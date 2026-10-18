# Cloud agent

Planning only: the current implementation runs locally from `ai/main.py`.

Cloud AI side: the AWS Lambda agent harness behind API Gateway (`/ha-event`, `/chat`),
the MCP client loop against HA's MCP Server, OpenRouter model calls, the chat bot and
deployment config. Keys live in AWS Secrets Manager / Lambda environment config, never here.
See [the local pipeline](../README.md), [design notes](architecture.md), and
[the roadmap](roadmap.md).
