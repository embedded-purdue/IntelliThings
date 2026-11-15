"""Bounded, non-streaming OpenRouter chat requests."""

import asyncio

import httpx


async def complete(api_key, model, messages, timeout, max_tokens, response_schema=None,
                   tools=None, tool_choice="auto"):
    if not api_key:
        raise ValueError("Set OPENROUTER_API_KEY in .env or the environment.")
    if not model:
        raise ValueError("Set OPENROUTER_MODEL in .env or the environment.")
    async with asyncio.timeout(timeout):
        async with httpx.AsyncClient(timeout=timeout) as client:
            response = await client.post(
                "https://openrouter.ai/api/v1/chat/completions",
                headers={"Authorization": f"Bearer {api_key}"},
                json={
                    "model": model,
                    "messages": messages,
                    "stream": False,
                    "max_tokens": max_tokens,
                    "reasoning": {"effort": "low"},
                    **({"tools": tools, "tool_choice": tool_choice} if tools else {}),
                    **({"response_format": {
                        "type": "json_schema",
                        "json_schema": {"name": "home_recommendation", "strict": True,
                                        "schema": response_schema},
                    }, "provider": {"require_parameters": True}} if response_schema else {}),
                },
            )
            if response.is_error:
                # Do not include upstream bodies or credentials in logs.
                raise RuntimeError(f"OpenRouter HTTP {response.status_code}")
            payload = response.json()
    return parse_completion(payload, model, tools, tool_choice, "OpenRouter")


def parse_completion(payload, model, tools, tool_choice, provider):
    """Normalize the shared chat-completion envelope without exposing API bodies."""
    if not isinstance(payload, dict):
        raise ValueError(f"Malformed {provider} response")
    if payload.get("error"):
        raise RuntimeError(f"{provider} returned an API error")
    try:
        choice = payload["choices"][0]
        message = choice["message"]
        content = message.get("content")
        calls = message.get("tool_calls")
        if choice.get("finish_reason") == "tool_calls" and tools and tool_choice != "none" and calls:
            return {"response_id": payload.get("id"), "model": payload.get("model", model),
                    "content": content, "message": message, "tool_calls": calls,
                    "usage": payload.get("usage")}
        if choice.get("finish_reason") != "stop" or calls:
            raise ValueError(f"{provider} response did not finish normally")
        if not isinstance(content, str) or not content.strip():
            raise ValueError(f"{provider} returned no assistant text")
    except (KeyError, IndexError, TypeError, AttributeError) as exc:
        raise ValueError(f"Malformed {provider} response") from exc
    return {
        "response_id": payload.get("id"),
        "model": payload.get("model", model),
        "content": content,
        "usage": payload.get("usage"),
    }
