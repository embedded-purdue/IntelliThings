# Home Assistant MCP probe

This standalone Python client needs no LLM or cloud account. It performs the MCP
handshake, prints the server's tools (including their JSON argument schemas), then
calls `homeassistant__GetLiveContext` if available. The default call only reads state.

## Local setup

Follow [local pipeline setup](../README.md#current-local-python-pipeline) for the
root `.venv`, `.env`, HA MCP integration, and entity exposure. The probes require
only the HA URL and access token; an OpenRouter key is not needed.

From the repository root:

```bash
.venv/bin/python ai/probes/probe.py
```

The script loads the root `.env` regardless of the working directory. Exported
environment variables override it. `HA_MCP_URL` or `--url` can override the full
endpoint; the default is `HA_BASE_URL` plus `/api/mcp`.

## Interactive discovery and calls

To print just the current home context, use the dedicated read-only script:

```bash
.venv/bin/python ai/probes/live_context.py
# Include the full MCP response envelope:
.venv/bin/python ai/probes/live_context.py --json
```

It uses the same `.env` settings as the probe and supports `--url` and `--timeout`.
It initializes the MCP session and calls `homeassistant__GetLiveContext` directly.

```bash
# List tool descriptions and inputSchema without making a tool call.
.venv/bin/python ai/probes/probe.py --list-only

# Explicitly request the current state snapshot.
.venv/bin/python ai/probes/probe.py --tool homeassistant__GetLiveContext --arguments '{}'
```

For any other tool, pass its exact advertised name to `--tool` and a JSON object
matching its `inputSchema` to `--arguments`. Explicit calls can operate devices;
discovery does not execute the listed tools. A failed tool result exits with status 1.

The central SDK calls are:

```python
await session.initialize()
tools = await session.list_tools()
result = await session.call_tool(name, arguments={})
```

The SDK handles JSON-RPC and Streamable HTTP. The script prints the call's name
and arguments followed by the complete MCP result, including text content,
structured content if supplied, and `isError`.

The live context is a **snapshot of Assist-exposed entities**, not an exhaustive
database export or a subscription to changes. Depending on available devices and
exposure settings, it can contain sensor readings, device states, names, and areas.
`unknown` and `unavailable` are Home Assistant states, not successful readings.
Run the probe again to refresh the snapshot. Continuous `state_changed` events
require a separate Home Assistant WebSocket client.

HTTP 404 means the integration or endpoint needs checking; 401 means the token
was rejected; connection refused means the host/port is unreachable. The script
does not install integrations, create tokens, or modify entity exposure settings.

`ConnectError: [Errno -2] Name or service not known` means hostname resolution
failed before authentication. On the project Pi, set
`HA_BASE_URL=http://127.0.0.1:8123` in the root `.env`; `homeassistant.local` may
not resolve. From another machine, use the Pi's reachable LAN address. Check for
exported `HA_BASE_URL` or `HA_MCP_URL` values overriding `.env` if the error persists.

References: [Home Assistant MCP Server](https://www.home-assistant.io/integrations/mcp_server/)
and [MCP Python SDK v1](https://github.com/modelcontextprotocol/python-sdk/tree/v1.x).
