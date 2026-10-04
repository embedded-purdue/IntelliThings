#!/usr/bin/env python3
"""Run one AI pipeline cycle; systemd schedules a cycle every five minutes."""

import argparse
import asyncio
import os
import sys
from pathlib import Path

from dotenv import load_dotenv

from pipeline.context import collect_snapshot, timestamp, write_json
from llm.inference import infer


async def run_cycle(args):
    snapshot = await collect_snapshot(
        args.url, os.getenv("HA_LONG_LIVED_TOKEN", "").strip(), args.timeout, args.output_dir
    )
    if snapshot is None or args.collect_only:
        write_json(args.output_dir / "inference_status.json", {
            "attempted_at": timestamp(), "ok": False, "skipped": True,
            "reason": "collection_failed" if snapshot is None else "collection_only",
        })
        return 1 if snapshot is None else 0
    return await infer(
        snapshot, args.output_dir, args.prompt_file,
        os.getenv("OPENROUTER_API_KEY", "").strip(),
        os.getenv("OPENROUTER_MODEL", "").strip(), args.llm_timeout, args.max_tokens,
    )


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
    return asyncio.run(run_cycle(args))


if __name__ == "__main__":
    sys.exit(main())
