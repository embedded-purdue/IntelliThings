"""Collect and persist a timestamped Home Assistant context snapshot."""

import asyncio
import sys

from agent.integrations.home_assistant import explain, get_live_context, parse_live_context
from agent.storage import timestamp, write_json


async def collect_snapshot(url, token, timeout, output_dir):
    attempted_at = timestamp()
    try:
        if not token:
            raise ValueError("Set HA_LONG_LIVED_TOKEN in the repository .env or environment.")
        result = await asyncio.wait_for(get_live_context(url, token, timeout), timeout)
        texts, error = parse_live_context(result)
        if error:
            raise RuntimeError(error)
        snapshot = {
            "schema_version": 1,
            "fetched_at": timestamp(),
            "tool": "homeassistant__GetLiveContext",
            "context": "\n".join(texts),
            "mcp_result": result.model_dump(mode="json", exclude_none=True),
        }
        write_json(output_dir / "latest.json", snapshot)
    except Exception as exc:
        error = explain(exc)
        if token:
            error = error.replace(token, "[REDACTED]")
        write_json(output_dir / "status.json", {
            "attempted_at": attempted_at, "ok": False, "error": error,
        })
        print(f"Context collection failed: {error}", file=sys.stderr)
        return None
    write_json(output_dir / "status.json", {
        "attempted_at": attempted_at, "ok": True, "fetched_at": snapshot["fetched_at"],
    })
    print(f"Saved live context to {output_dir / 'latest.json'} at {snapshot['fetched_at']}")
    return snapshot
