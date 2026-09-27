#!/usr/bin/env python3
"""Run one AI pipeline cycle; systemd schedules a cycle every five minutes."""

import argparse
import asyncio
import os
import sys
from pathlib import Path

from dotenv import load_dotenv

from pipeline.context import collect


def main():
    load_dotenv(Path(__file__).resolve().parents[1] / ".env")
    base = os.getenv("HA_BASE_URL", "http://127.0.0.1:8123").rstrip("/")
    state_home = Path(os.getenv("XDG_STATE_HOME") or Path.home() / ".local/state")
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--url", default=os.getenv("HA_MCP_URL") or f"{base}/api/mcp")
    parser.add_argument("--timeout", type=float, default=60)
    parser.add_argument("--output-dir", type=Path, default=state_home / "intellithings/mcp")
    args = parser.parse_args()
    if args.timeout <= 0:
        parser.error("--timeout must be positive")
    return asyncio.run(collect(
        args.url, os.getenv("HA_LONG_LIVED_TOKEN", "").strip(), args.timeout, args.output_dir
    ))


if __name__ == "__main__":
    sys.exit(main())
