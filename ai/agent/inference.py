"""Build prompts, run the bounded model/tool interaction, and validate summaries."""

import asyncio
from contextlib import AsyncExitStack
import hashlib
import json

from agent.integrations.openrouter import complete
from agent.integrations.home_assistant import connect
from agent.controls import Controller
from agent.response import Recommendation, parse_response
from agent.storage import timestamp


async def infer(snapshot, output_dir, prompt_file, api_key, model, timeout, max_tokens,
                mcp_url=None, ha_token=None):
    """Return validated summaries and their persistence record; propagate failures.

    Controls journal their own actions to output_dir before/after execution so
    an action is still recorded if the final model response fails.
    """
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
    record = {
        "schema_version": 5,
        "generated_at": generated_at,
        "input_fetched_at": snapshot["fetched_at"],
        "requested_model": model,
        "prompt_sha256": hashlib.sha256(prompt.encode("utf-8")).hexdigest(),
        **result,
        "recommendation": recommendation.model_dump(),
        "requests": requests,
        "actions": controller.actions if controller else [],
    }
    return recommendation, record
