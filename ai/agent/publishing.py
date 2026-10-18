"""Publish summaries and thinking state, recording delivery results separately."""

import asyncio
import sys

from agent.integrations.dashboard import publish_summary, ENTITY_ID
from agent.integrations.mqtt import publish_summaries as publish_mqtt_summaries, publish_inference_state
from agent.storage import timestamp, write_json


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


async def publish_outputs(base_url, token, recommendation, record, output_dir):
    """Attempt both outputs independently, preserving per-output status files."""
    generated_at = record["generated_at"]

    async def publish_and_record(name, request, **metadata):
        publication_status = {"attempted_at": timestamp(),
                              "input_fetched_at": record["input_fetched_at"],
                              "generated_at": generated_at, **metadata}
        try:
            details = await request
            publication_status.update(details or {"ok": True})
        except Exception as exc:
            error = str(exc).replace(token, "[REDACTED]") if token else str(exc)
            publication_status.update(ok=False, error=error)
        write_json(output_dir / f"{name}_status.json", publication_status)
        print(f"{name} publication {'succeeded' if publication_status['ok'] else 'failed'}")
        return publication_status["ok"]

    # Both outputs are attempted, even when one fails. Parallel publication keeps
    # the service budget: 60s collection + 45s inference + 10s outputs + 2s stop status.
    published = await asyncio.gather(
        publish_and_record("dashboard", publish_summary(base_url, token, recommendation.summary),
                           entity_id=ENTITY_ID),
        publish_and_record("mqtt", publish_mqtt_summaries(
            base_url, token, recommendation, generated_at)),
    )
    return all(published)
