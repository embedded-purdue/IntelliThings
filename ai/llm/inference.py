"""Build prompts and persist inference separately from collection."""

import asyncio
from contextlib import AsyncExitStack

import hashlib
import json
import sys

from integrations.openrouter import complete
from integrations.home_assistant import connect, explain
from integrations.dashboard import publish_summary, ENTITY_ID
from integrations.mqtt import publish_summaries as publish_mqtt_summaries, publish_inference_state
from llm.control import Controller
from llm.response import Recommendation, parse_response
from pipeline.context import timestamp, write_json


async def infer(snapshot, output_dir, prompt_file, api_key, model, timeout, max_tokens, mcp_url=None, ha_token=None,
                ha_base_url=None):
    events = []
    started = asyncio.get_running_loop().time()
    start_ok = stop_ok = True
    try:
        if ha_base_url:
            start_ok = await set_thinking_state(ha_base_url, ha_token, "start thinking", output_dir, events)
        # Status publication shares the inference budget rather than extending it.
        remaining = max(0, timeout - (asyncio.get_running_loop().time() - started))
        result = await _infer(snapshot, output_dir, prompt_file, api_key, model, remaining,
                              max_tokens, mcp_url, ha_token, ha_base_url)
    finally:
        # Also runs on invalid model output, publication failure, and cancellation.
        if ha_base_url:
            stop_ok = await set_thinking_state(ha_base_url, ha_token, "stop thinking", output_dir, events)
    return 0 if result == 0 and start_ok and stop_ok else 1


async def set_thinking_state(base_url, token, state, output_dir, events):
    try:
        event = await publish_inference_state(base_url, token, state)
    except Exception as exc:
        error = f"{type(exc).__name__}: {exc}"
        event = {"ok": False, "state": state, "ts": timestamp(),
                 "error": error.replace(token, "[REDACTED]") if token else error}
    events.append(event)
    write_json(output_dir / "thinking_status.json", {"ok": all(item["ok"] for item in events),
                                                   "events": events})
    if not event["ok"]:
        print(f"Could not publish {state}; see thinking_status.json", file=sys.stderr)
    return event["ok"]


async def _infer(snapshot, output_dir, prompt_file, api_key, model, timeout, max_tokens, mcp_url=None, ha_token=None,
                 ha_base_url=None):
    status = {
        "attempted_at": timestamp(),
        "input_fetched_at": snapshot["fetched_at"],
        "requested_model": model,
    }
    try:
        prompt = prompt_file.read_text(encoding="utf-8").strip()
        if not prompt:
            raise ValueError("Prompt file is empty")
        if not snapshot.get("context", "").strip():
            raise ValueError("Collected context is empty")
        async with asyncio.timeout(timeout), AsyncExitStack() as stack:
            controller = None
            if mcp_url:
                session = await stack.enter_async_context(connect(mcp_url, ha_token, timeout))
                controller = Controller(session, output_dir)
                await controller.discover(snapshot)
            messages = [
                {"role": "system", "content": prompt},
                {"role": "user", "content": json.dumps({
                    "fetched_at": snapshot["fetched_at"],
                    "context": snapshot["context"],
                    "actuator_inventory": controller.inventory if controller else {},
                }, ensure_ascii=False)},
            ]
            tools = controller.tools if controller else []
            result = await complete(api_key, model, messages, timeout, max_tokens,
                response_schema=Recommendation.model_json_schema(), tools=tools)
            requests = [{"response_id": result.get("response_id"), "usage": result.get("usage")}]
            if result.get("tool_calls"):
                calls = result["tool_calls"]
                if len(calls) != 1:
                    raise ValueError("Only one tool call per cycle is supported")
                messages.append(result["message"])
                outcome = await controller.execute(calls[0])
                messages.append({"role": "tool", "tool_call_id": calls[0]["id"],
                                 "content": json.dumps(outcome)})
                result = await complete(api_key, model, messages, timeout, max_tokens,
                    response_schema=Recommendation.model_json_schema(), tools=tools, tool_choice="none")
                requests.append({"response_id": result.get("response_id"), "usage": result.get("usage")})
        recommendation = parse_response(result["content"])
        generated_at = timestamp()
        write_json(output_dir / "inference.json", {
            "schema_version": 5,
            "generated_at": generated_at,
            "input_fetched_at": snapshot["fetched_at"],
            "requested_model": model,
            "prompt_sha256": hashlib.sha256(prompt.encode("utf-8")).hexdigest(),
            **result,
            "recommendation": recommendation.model_dump(),
            "requests": requests,
            "actions": controller.actions if controller else [],
        })
    except Exception as exc:
        error = explain(exc)
        for secret in (api_key, ha_token):
            if secret:
                error = error.replace(secret, "[REDACTED]")
        write_json(output_dir / "inference_status.json", {**status, "ok": False, "error": error})
        write_json(output_dir / "dashboard_status.json", {**status, "ok": False,
                   "skipped": True, "reason": "inference_failed"})
        write_json(output_dir / "mqtt_status.json", {**status, "ok": False,
                   "skipped": True, "reason": "inference_failed"})
        print(f"Inference failed: {error}", file=sys.stderr)
        return 1
    write_json(output_dir / "inference_status.json", {**status, "ok": True})
    print(f"Saved model response to {output_dir / 'inference.json'}")
    if ha_base_url:
        async def publish_and_record(name, request, **metadata):
            publication_status = {"attempted_at": timestamp(),
                                  "input_fetched_at": snapshot["fetched_at"],
                                  "generated_at": generated_at, **metadata}
            try:
                details = await request
                publication_status.update(details or {"ok": True})
            except Exception as exc:
                error = str(exc).replace(ha_token, "[REDACTED]") if ha_token else str(exc)
                publication_status.update(ok=False, error=error)
            write_json(output_dir / f"{name}_status.json", publication_status)
            print(f"{name} publication {'succeeded' if publication_status['ok'] else 'failed'}")
            return publication_status["ok"]

        # Both outputs are attempted, even when one fails. Parallel publication keeps
        # the service budget: 60s collection + 45s inference + 10s outputs + 2s stop status.
        published = await asyncio.gather(
            publish_and_record("dashboard", publish_summary(ha_base_url, ha_token, recommendation.summary),
                               entity_id=ENTITY_ID),
            publish_and_record("mqtt", publish_mqtt_summaries(
                ha_base_url, ha_token, recommendation, generated_at)),
        )
        return 0 if all(published) else 1
    return 0
