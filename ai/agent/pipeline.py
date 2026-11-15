"""Coordinate collection, inference, persistence, and publication for one run."""

import asyncio
import os
import sys

from agent.context import collect_snapshot
from agent.inference import infer
from agent.integrations.home_assistant import explain
from agent.publishing import publish_outputs, set_thinking_state
from agent.storage import timestamp, write_json


async def run_cycle(args):
    token = os.getenv("HA_LONG_LIVED_TOKEN", "").strip()
    base = os.getenv("HA_BASE_URL", "http://127.0.0.1:8123").rstrip("/")
    collection = collect_snapshot(args.url, token, args.timeout, args.output_dir)
    if args.collect_only:
        snapshot = await collection
    else:
        # Recover a retained thinking flag from an interrupted previous run while
        # collecting. The output-directory lock is already held by main().
        snapshot, _ = await asyncio.gather(collection, set_thinking_state(
            base, token, "stop thinking", args.output_dir, []))
    if snapshot is None or args.collect_only:
        reason = "collection_failed" if snapshot is None else "collection_only"
        for stage in ("mqtt", "dashboard", "inference"):
            write_json(args.output_dir / f"{stage}_status.json", {
                "attempted_at": timestamp(), "ok": False, "skipped": True, "reason": reason,
            })
        return 1 if snapshot is None else 0
    return await run_inference(
        snapshot, args.output_dir, args.prompt_file,
        os.getenv("OPENROUTER_API_KEY", "").strip(),
        os.getenv("OPENROUTER_MODEL", "").strip(), args.llm_timeout, args.max_tokens,
        mcp_url=args.url, ha_token=token, ha_base_url=base,
        genai_api_key=os.getenv("GENAI_API_KEY", "").strip(),
        genai_model=os.getenv("GENAI_MODEL", "").strip() or "gpt-oss:120b",
    )


async def run_inference(snapshot, output_dir, prompt_file, api_key, model, timeout, max_tokens,
                        mcp_url=None, ha_token=None, ha_base_url=None, *,
                        genai_api_key="", genai_model="gpt-oss:120b"):
    """Generate, save, and publish one result; always attempt the final thinking reset."""
    events = []
    started = asyncio.get_running_loop().time()
    start_ok = stop_ok = True
    try:
        if ha_base_url:
            start_ok = await set_thinking_state(ha_base_url, ha_token, "start thinking", output_dir, events)
        # Starting status shares the existing inference budget.
        remaining = max(0, timeout - (asyncio.get_running_loop().time() - started))
        status = {"attempted_at": timestamp(), "input_fetched_at": snapshot["fetched_at"],
                  "requested_model": genai_model if genai_api_key else model}
        try:
            recommendation, record = await infer(
                snapshot, output_dir, prompt_file, api_key, model, remaining,
                max_tokens, mcp_url, ha_token,
                genai_api_key=genai_api_key, genai_model=genai_model)
            write_json(output_dir / "inference.json", record)
        except Exception as exc:
            error = explain(exc)
            for secret in (api_key, ha_token, genai_api_key):
                if secret:
                    error = error.replace(secret, "[REDACTED]")
            write_json(output_dir / "inference_status.json", {**status, "ok": False, "error": error})
            for stage in ("dashboard", "mqtt"):
                write_json(output_dir / f"{stage}_status.json", {**status, "ok": False,
                           "skipped": True, "reason": "inference_failed"})
            print(f"Inference failed: {error}", file=sys.stderr)
            return 1

        write_json(output_dir / "inference_status.json", {**status, "ok": True,
                   "provider": record.get("provider", "openrouter"),
                   "requested_model": record["requested_model"]})
        print(f"Saved model response to {output_dir / 'inference.json'}")
        published = True
        if ha_base_url:
            published = await publish_outputs(ha_base_url, ha_token, recommendation, record, output_dir)
    finally:
        # Covers validation errors, timeouts, output failures, and cancellation.
        if ha_base_url:
            stop_ok = await set_thinking_state(ha_base_url, ha_token, "stop thinking", output_dir, events)
    return 0 if published and start_ok and stop_ok else 1
