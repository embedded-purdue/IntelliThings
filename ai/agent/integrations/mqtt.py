"""Publish AI text using HA's MQTT connection and the emulator display contract."""

import asyncio
import json
from pathlib import Path
import re

import httpx

from agent.storage import timestamp

CONFIG = Path(__file__).resolve().parents[2] / "config/display_nodes.json"
SUMMARY_TOPIC = "intellithings/ai/summary"
INFERENCE_TOPIC = "intellithings/ai/inference_status"
TEXT_BYTES = 400
PAYLOAD_BYTES = 512


def display_payload(summary, generated_at):
    """Keep UTF-8 text and complete JSON within the emulator's byte buffers."""
    def encode(text):
        return json.dumps({"text": text, "ts": generated_at}, ensure_ascii=False,
                          separators=(",", ":"))

    if len(encode("").encode("utf-8")) > PAYLOAD_BYTES:
        raise ValueError("Timestamp exceeds display payload capacity")
    text = summary
    truncated = False
    while True:
        display_text = text + ("…" if truncated else "")
        payload = encode(display_text)
        if (len(display_text.encode("utf-8")) <= TEXT_BYTES
                and len(payload.encode("utf-8")) <= PAYLOAD_BYTES):
            return payload, truncated
        if not text:
            raise ValueError("Cannot fit display message in emulator buffers")
        text = text[:-1]
        truncated = True


def load_nodes():
    nodes = json.loads(CONFIG.read_text(encoding="utf-8"))
    if (not isinstance(nodes, dict)
            or any(not re.fullmatch(r"[A-Za-z0-9_-]+", node)
                   or room not in ("kitchen", "bedroom", "living_room")
                   for node, room in nodes.items())):
        raise ValueError("display_nodes.json must map node IDs (without slashes or wildcards) "
                         "to kitchen, bedroom, or living_room")
    return nodes


async def _publish_messages(base_url, token, messages, timeout):
    """Attempt every destination concurrently, returning service-level results."""
    if not token:
        raise ValueError("HA_LONG_LIVED_TOKEN is required for MQTT publication")
    async with httpx.AsyncClient(
        base_url=base_url.rstrip("/"), headers={"Authorization": f"Bearer {token}"},
        timeout=timeout,
    ) as client:
        async def publish(message):
            topic, payload = message["topic"], message["payload"]
            details = {key: value for key, value in message.items() if key != "payload"}
            details["payload_bytes"] = len(payload.encode("utf-8"))
            try:
                async with asyncio.timeout(timeout):
                    response = await client.post("/api/services/mqtt/publish", json={
                        "topic": topic, "payload": payload, "qos": 1, "retain": True,
                    })
                    response.raise_for_status()
                return {**details, "ok": True}
            except Exception as exc:
                return {**details, "ok": False,
                        "error": f"{type(exc).__name__}: {exc}".replace(token, "[REDACTED]")}

        topics = await asyncio.gather(*(publish(message) for message in messages))
    return {"ok": all(item["ok"] for item in topics), "topics": topics, "qos": 1, "retain": True}


async def publish_summaries(base_url, token, recommendation, generated_at, timeout=10):
    # The overall HA sensor receives the full 255-character summary. Only device
    # messages use the emulator byte limits.
    messages = [{"topic": SUMMARY_TOPIC, "truncated": False, "payload": json.dumps(
        {"text": recommendation.summary, "ts": generated_at}, ensure_ascii=False)}]
    rooms = recommendation.room_summaries.model_dump()
    for node, room in load_nodes().items():
        payload, truncated = display_payload(rooms[room], generated_at)
        messages.append({"topic": f"intellithings/{node}/display", "payload": payload,
                         "room": room, "truncated": truncated})
    return {**await _publish_messages(base_url, token, messages, timeout), "generated_at": generated_at}


async def publish_inference_state(base_url, token, state, timeout=2):
    if state not in ("start thinking", "stop thinking"):
        raise ValueError("Invalid inference state")
    ts = timestamp()
    result = await _publish_messages(base_url, token, [{"topic": INFERENCE_TOPIC,
        "payload": json.dumps({"state": state, "ts": ts})}], timeout)
    return {**result, "state": state, "ts": ts}
