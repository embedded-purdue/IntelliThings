#!/usr/bin/env python3
"""Run one AI pipeline cycle; systemd schedules a cycle every five minutes."""

import argparse
import asyncio
import fcntl
import os
import sys
from pathlib import Path

from dotenv import load_dotenv

from agent.pipeline import run_cycle


def main():
    load_dotenv(Path(__file__).resolve().parents[1] / ".env")
    base = os.getenv("HA_BASE_URL", "http://127.0.0.1:8123").rstrip("/")
    state_home = Path(os.getenv("XDG_STATE_HOME") or Path.home() / ".local/state")
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--url", default=os.getenv("HA_MCP_URL") or f"{base}/api/mcp")
    parser.add_argument("--timeout", type=float, default=60)
    parser.add_argument("--output-dir", type=Path, default=state_home / "intellithings/mcp")
    parser.add_argument("--collect-only", action="store_true", help="Skip OpenRouter inference")
    parser.add_argument("--prompt-file", type=Path,
                        default=Path(__file__).resolve().parent / "prompts/home_status.txt")
    parser.add_argument("--llm-timeout", type=float, default=45)
    parser.add_argument("--max-tokens", type=int, default=2048)
    args = parser.parse_args()
    if args.timeout <= 0 or args.llm_timeout <= 0 or args.max_tokens <= 0:
        parser.error("Timeouts and --max-tokens must be positive")
    args.output_dir.mkdir(parents=True, exist_ok=True, mode=0o700)
    with (args.output_dir / ".pipeline.lock").open("w") as lock:
        try:
            fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError:
            print("A pipeline cycle is already running for this output directory.", file=sys.stderr)
            return 1
        return asyncio.run(run_cycle(args))


if __name__ == "__main__":
    sys.exit(main())
