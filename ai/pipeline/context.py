"""Collect and persist a timestamped Home Assistant context snapshot."""

import asyncio
import json
import os
import sys
import tempfile
from datetime import datetime, timezone

from integrations.home_assistant import explain, get_live_context


def timestamp():
    return datetime.now(timezone.utc).isoformat()


def write_json(path, value):
    """Publish a complete file atomically, with owner-only permissions."""
    path.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
    name = None
    try:
        with tempfile.NamedTemporaryFile(mode="w", dir=path.parent, delete=False) as f:
            name = f.name
            json.dump(value, f, indent=2, ensure_ascii=False)
            f.write("\n")
            f.flush()
            os.fsync(f.fileno())
        os.replace(name, path)
    finally:
        if name and os.path.exists(name):
            os.unlink(name)


async def collect(url, token, timeout, output_dir):
    attempted_at = timestamp()
    try:
        if not token:
            raise ValueError("Set HA_LONG_LIVED_TOKEN in the repository .env or environment.")
        result = await asyncio.wait_for(get_live_context(url, token, timeout), timeout)
        if result.isError:
            raise RuntimeError("GetLiveContext returned an MCP tool error.")
        texts = []
        for block in result.content:
            if block.type != "text":
                continue
            try:
                payload = json.loads(block.text)
            except json.JSONDecodeError:
                payload = block.text
            if isinstance(payload, dict):
                if payload.get("success") is False:
                    raise RuntimeError("GetLiveContext reported success=false.")
                payload = payload.get("result", payload)
            texts.append(payload if isinstance(payload, str) else json.dumps(payload))
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
        return 1
    write_json(output_dir / "status.json", {
        "attempted_at": attempted_at, "ok": True, "fetched_at": snapshot["fetched_at"],
    })
    print(f"Saved live context to {output_dir / 'latest.json'} at {snapshot['fetched_at']}")
    return 0

