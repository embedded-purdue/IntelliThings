"""Build prompts and persist inference separately from collection."""

import asyncio
from contextlib import AsyncExitStack

import hashlib
import json
import sys

from integrations.openrouter import complete
from integrations.home_assistant import connect, explain
from integrations.dashboard import publish_summary, ENTITY_ID
from llm.control import Controller
from llm.response import Recommendation, parse_response
from pipeline.context import timestamp, write_json


async def infer(snapshot, output_dir, prompt_file, api_key, model, timeout, max_tokens, mcp_url=None, ha_token=None,
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
        write_json(output_dir / "inference.json", {
            "schema_version": 3,
            "generated_at": timestamp(),
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
        print(f"Inference failed: {error}", file=sys.stderr)
        return 1
    write_json(output_dir / "inference_status.json", {**status, "ok": True})
    print(f"Saved model response to {output_dir / 'inference.json'}")
    if ha_base_url:
        dashboard_status = {"attempted_at": timestamp(), "input_fetched_at": snapshot["fetched_at"],
                            "entity_id": ENTITY_ID}
        try:
            await publish_summary(ha_base_url, ha_token, recommendation.summary)
        except Exception as exc:
            error = str(exc).replace(ha_token, "[REDACTED]") if ha_token else str(exc)
            write_json(output_dir / "dashboard_status.json", {**dashboard_status, "ok": False, "error": error})
            print(f"Dashboard update failed: {error}", file=sys.stderr)
            return 1
        write_json(output_dir / "dashboard_status.json", {**dashboard_status, "ok": True})
        print(f"Updated {ENTITY_ID}")
    return 0
