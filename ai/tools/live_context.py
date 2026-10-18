#!/usr/bin/env python3
"""Print the current Assist-exposed home context using Home Assistant MCP."""

import argparse
import asyncio
import os
import sys
from pathlib import Path

from dotenv import load_dotenv

# Allow direct execution from any working directory.
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from agent.integrations.home_assistant import explain, get_live_context, parse_live_context


def main():
    load_dotenv(Path(__file__).resolve().parents[2] / ".env")
    base = os.getenv("HA_BASE_URL", "http://127.0.0.1:8123").rstrip("/")
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--url", default=os.getenv("HA_MCP_URL") or f"{base}/api/mcp")
    parser.add_argument("--timeout", type=float, default=30)
    parser.add_argument("--json", action="store_true", help="Print the full MCP result as JSON")
    args = parser.parse_args()
    if args.timeout <= 0:
        parser.error("--timeout must be positive")
    token = os.getenv("HA_LONG_LIVED_TOKEN", "").strip()
    if not token:
        parser.error("Set HA_LONG_LIVED_TOKEN in the repository .env or environment.")
    try:
        result = asyncio.run(get_live_context(args.url, token, args.timeout))
        texts, error = parse_live_context(result, indent=2)
        if args.json or not texts:
            print(result.model_dump_json(indent=2, exclude_none=True))
        else:
            print("\n".join(texts))
        return 1 if error else 0
    except KeyboardInterrupt:
        return 130
    except Exception as exc:
        print(f"Connection/tool error: {explain(exc).replace(token, '[REDACTED]')}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
