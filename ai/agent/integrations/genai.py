"""Bounded Purdue GenAI Studio chat requests (vLLM hosted models)."""

import asyncio

import httpx

from agent.integrations.openrouter import parse_completion


async def complete(api_key, model, messages, timeout, max_tokens, response_schema=None,
                   tools=None, tool_choice="auto"):
    if not api_key or not model:
        raise ValueError("Set GENAI_API_KEY and GENAI_MODEL in .env or the environment.")
    body = {"model": model, "messages": messages, "stream": False,
            "max_tokens": max_tokens, "reasoning_effort": "low"}
    if tools:
        body.update(tools=tools, tool_choice=tool_choice)
    if response_schema:
        body["response_format"] = {"type": "json_schema", "json_schema": {
            "name": "home_recommendation", "strict": True, "schema": response_schema}}
    async with asyncio.timeout(timeout):
        async with httpx.AsyncClient(timeout=timeout) as client:
            response = await client.post(
                "https://genai.rcac.purdue.edu/api/chat/completions",
                headers={"Authorization": f"Bearer {api_key}"}, json=body)
            if response.is_error:
                raise RuntimeError(f"GenAI HTTP {response.status_code}")
            payload = response.json()
    return parse_completion(payload, model, tools, tool_choice, "GenAI")
