"""Build prompts, run the bounded model/tool interaction, and validate summaries."""

import asyncio
from contextlib import AsyncExitStack
import hashlib
import json

from agent.integrations.openrouter import complete
from agent.integrations.genai import complete as complete_genai
from agent.integrations.home_assistant import connect
from agent.controls import Controller
from agent.response import Recommendation, parse_response
from agent.storage import timestamp


async def infer(snapshot, output_dir, prompt_file, api_key, model, timeout, max_tokens,
                mcp_url=None, ha_token=None, *, genai_api_key="", genai_model="gpt-oss:120b"):
    """Return validated summaries and their persistence record; propagate failures.

    Controls journal their own actions to output_dir before/after execution so
    an action is still recorded if the final model response fails.
    """
    prompt = prompt_file.read_text(encoding="utf-8").strip()
    if not prompt:
        raise ValueError("Prompt file is empty")
    if not snapshot.get("context", "").strip():
        raise ValueError("Collected context is empty")
    loop = asyncio.get_running_loop()
    deadline = loop.time() + timeout
    async with asyncio.timeout_at(deadline), AsyncExitStack() as stack:
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
        provider = "genai" if genai_api_key else "openrouter"
        # Bound the whole Purdue phase, leaving time for the paid fallback.
        genai_deadline = loop.time() + min(20, max(0, deadline - loop.time()) / 2)
        requests = []

        async def request(tool_choice="auto"):
            nonlocal provider
            while True:
                primary = provider == "genai"
                requested_model = genai_model if primary else model
                attempt = {"provider": provider, "requested_model": requested_model}
                requests.append(attempt)
                try:
                    remaining = (genai_deadline if primary else deadline) - loop.time()
                    if remaining <= 0:
                        raise TimeoutError()
                    result = await (complete_genai if primary else complete)(
                        genai_api_key if primary else api_key, requested_model,
                        messages, remaining, max_tokens,
                        response_schema=Recommendation.model_json_schema(), tools=tools,
                        tool_choice=tool_choice)
                    attempt.update(response_id=result.get("response_id"), usage=result.get("usage"))
                    calls = result.get("tool_calls")
                    if calls:
                        if tool_choice == "none" or not controller or not isinstance(calls, list) or len(calls) != 1:
                            raise ValueError("Only one tool call per cycle is supported")
                        controller.validate(calls[0])
                    else:
                        parse_response(result["content"])
                    return {**result, "provider": provider, "requested_model": requested_model}
                except Exception as exc:
                    # Failure categories are useful without retaining credentials or API bodies.
                    attempt["error"] = type(exc).__name__
                    if not primary:
                        raise
                    provider = "openrouter"

        result = await request()
        if result.get("tool_calls"):
            calls = result["tool_calls"]
            if len(calls) != 1:
                raise ValueError("Only one tool call per cycle is supported")
            messages.append(result["message"])
            outcome = await controller.execute(calls[0])
            messages.append({"role": "tool", "tool_call_id": calls[0]["id"],
                             "content": json.dumps(outcome)})
            result = await request(tool_choice="none")
    recommendation = parse_response(result["content"])
    generated_at = timestamp()
    record = {
        "schema_version": 6,
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
