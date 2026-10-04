"""Bounded, non-streaming OpenRouter chat requests."""

import asyncio

import httpx


async def complete(api_key, model, messages, timeout, max_tokens, response_schema=None):
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
    if payload.get("error"):
        raise RuntimeError("OpenRouter returned an API error")
    try:
        choice = payload["choices"][0]
        content = choice["message"]["content"]
        if choice.get("finish_reason") != "stop":
            raise ValueError("OpenRouter response did not finish normally")
        if not isinstance(content, str) or not content.strip():
            raise ValueError("OpenRouter returned no assistant text")
    except (KeyError, IndexError, TypeError) as exc:
        raise ValueError("Malformed OpenRouter response") from exc
    return {
        "response_id": payload.get("id"),
        "model": payload.get("model", model),
        "content": content,
        "usage": payload.get("usage"),
    }
