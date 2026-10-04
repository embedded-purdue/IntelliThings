"""Shared Home Assistant MCP transport for the pipeline and inspection tools."""

from datetime import timedelta
from contextlib import asynccontextmanager

import httpx
from mcp import ClientSession
from mcp.client.streamable_http import streamable_http_client


@asynccontextmanager
async def connect(url, token, timeout):
    async with httpx.AsyncClient(
        headers={"Authorization": f"Bearer {token}"}, timeout=timeout
    ) as client:
        async with streamable_http_client(url, http_client=client) as (read, write, _):
            async with ClientSession(
                read, write, read_timeout_seconds=timedelta(seconds=timeout)
            ) as session:
                await session.initialize()
                yield session


async def get_live_context(url, token, timeout):
    async with connect(url, token, timeout) as session:
        return await session.call_tool("homeassistant__GetLiveContext", arguments={})


def explain(exc):
    """Unwrap transport task groups to make connection failures actionable."""
    if isinstance(exc, BaseExceptionGroup):
        return "; ".join(explain(child) for child in exc.exceptions)
    if isinstance(exc, httpx.HTTPStatusError):
        status = exc.response.status_code
        hint = {
            401: "Access token is missing, invalid, or expired.",
            403: "This user cannot access the selected MCP API.",
            404: "Add the Model Context Protocol Server integration in Home Assistant "
                 "and check the MCP URL (/api/mcp).",
        }.get(status, "Check the Home Assistant endpoint and server logs.")
        return f"HTTP {status}: {hint}"
    if isinstance(exc, (TimeoutError, httpx.TimeoutException)):
        return "Request timed out; check the server address or increase --timeout."
    return f"{type(exc).__name__}: {exc}"
