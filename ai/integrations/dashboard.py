"""Publish the validated summary to the HA dashboard text helper."""

import asyncio

import httpx

ENTITY_ID = "input_text.ai_summary_overall"


async def publish_summary(base_url, token, summary, timeout=10):
    if not token:
        raise ValueError("HA_LONG_LIVED_TOKEN is required to publish the summary")
    if not isinstance(summary, str) or len(summary) > 255:
        raise ValueError("Dashboard summary must be a string of at most 255 characters")
    async with asyncio.timeout(timeout):
        async with httpx.AsyncClient(
            base_url=base_url.rstrip("/"),
            headers={"Authorization": f"Bearer {token}"}, timeout=timeout,
        ) as client:
            response = await client.post("/api/services/input_text/set_value", json={
                "entity_id": ENTITY_ID, "value": summary,
            })
            response.raise_for_status()
            # Confirm persisted state; a service response alone is not proof of the value.
            state = await client.get(f"/api/states/{ENTITY_ID}")
            state.raise_for_status()
            if state.json().get("state") != summary:
                raise ValueError("Dashboard helper did not retain the requested summary")
