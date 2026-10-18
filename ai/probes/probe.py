#!/usr/bin/env python3
"""Inspect Home Assistant MCP tools and request a live Assist context snapshot."""

import argparse
import asyncio
import json
import os
import sys
from datetime import timedelta
from pathlib import Path

import httpx
from dotenv import load_dotenv
from mcp import ClientSession
from mcp.client.streamable_http import streamable_http_client


# Allow direct execution from any working directory.
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from integrations.home_assistant import explain, get_live_context


def show(label, value):
    print(f"\n--- {label} ---", flush=True)
    if hasattr(value, "model_dump"):
        value = value.model_dump(mode="json", exclude_none=True)
    print(json.dumps(value, indent=2, ensure_ascii=False), flush=True)


async def probe(args, token):
    headers = {"Authorization": f"Bearer {token}"}
    async with httpx.AsyncClient(headers=headers, timeout=args.timeout) as client:
        async with streamable_http_client(args.url, http_client=client) as (read, write, _):
            async with ClientSession(
                read, write, read_timeout_seconds=timedelta(seconds=args.timeout)
            ) as session:
                # MCP handshake must precede discovery or tool calls.
                initialized = await session.initialize()
                show("Server", initialized)

                tools = []
                cursor = None
                while True:
                    page = await session.list_tools(cursor=cursor)
                    tools.extend(page.tools)
                    cursor = page.nextCursor
                    if not cursor:
                        break
                show("Available tools and input schemas", [
                    tool.model_dump(mode="json", exclude_none=True) for tool in tools
                ])

                if args.list_only:
                    return 0

                # Only this known read-only tool is called automatically.
                name = args.tool or next((tool.name for tool in tools if tool.name in {
                    "homeassistant__GetLiveContext", "GetLiveContext"
                }), None)
                if name is None:
                    print("No GetLiveContext tool advertised. Check the integration's "
                          "Assist API selection and exposed entities. "
                          "Use --tool and --arguments for another advertised tool.")
                    return 0
                if name not in {tool.name for tool in tools}:
                    raise ValueError(f"Tool {name!r} is not advertised by this server.")

                # This maps to MCP JSON-RPC tools/call with name and arguments.
                show("tools/call request", {"name": name, "arguments": args.arguments})
                result = await session.call_tool(name, arguments=args.arguments)
                show("Tool result", result)
                return 1 if result.isError else 0


def main():
    # Environment variables take precedence over the repository's ignored .env.
    load_dotenv(Path(__file__).resolve().parents[2] / ".env")
    base = os.getenv("HA_BASE_URL", "http://127.0.0.1:8123").rstrip("/")
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--url", default=os.getenv("HA_MCP_URL") or f"{base}/api/mcp")
    parser.add_argument("--list-only", action="store_true", help="Discover tools without calling any")
    parser.add_argument("--tool", help="Explicit tool to call; may change devices depending on the tool")
    parser.add_argument("--arguments", default="{}", help="JSON object matching the tool's inputSchema")
    parser.add_argument("--timeout", type=float, default=30, help="Request timeout in seconds")
    args = parser.parse_args()
    if args.timeout <= 0:
        parser.error("--timeout must be positive")
    if args.list_only and args.tool:
        parser.error("--list-only and --tool cannot be used together")
    try:
        args.arguments = json.loads(args.arguments)
    except json.JSONDecodeError as exc:
        parser.error(f"--arguments must be valid JSON: {exc.msg}")
    if not isinstance(args.arguments, dict):
        parser.error("--arguments must be a JSON object")
    if args.arguments and not args.tool:
        parser.error("Use --tool when supplying arguments")
    token = os.getenv("HA_LONG_LIVED_TOKEN", "").strip()
    if not token:
        parser.error("Set HA_LONG_LIVED_TOKEN in the repository .env or environment. "
                     "Create it in Home Assistant: user profile > Security > Long-lived access tokens.")
    try:
        return asyncio.run(probe(args, token))
    except KeyboardInterrupt:
        return 130
    except Exception as exc:
        print(f"Connection/tool error: {explain(exc).replace(token, '[REDACTED]')}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
