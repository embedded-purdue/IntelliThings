# Cloud agent

Cloud AI side: the AWS Lambda agent harness behind API Gateway (`/ha-event`, `/chat`),
the MCP client loop against HA's MCP Server, OpenRouter model calls, the chat bot and
deployment config. Keys live in AWS Secrets Manager / Lambda environment config, never here.
See [`../README.md`](../README.md) and [`../AI_Agent_Notes.md`](../AI_Agent_Notes.md).
