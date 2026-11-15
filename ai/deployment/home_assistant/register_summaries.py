#!/usr/bin/env python3
"""Preview or register missing HA MQTT room summaries from a saved inference."""

import argparse
import json
import os
import sys
from pathlib import Path

import httpx
from dotenv import load_dotenv

sys.path.insert(0, str(Path(__file__).resolve().parents[2]))
from agent.response import load_response


def discovery_messages(rooms):
    for room in sorted(rooms):
        unique_id = f"intellithings_ai_summary_{room}"
        yield f"homeassistant/sensor/{unique_id}/config", {
            "name": f"{room.replace('_', ' ').title()} AI Summary",
            "unique_id": unique_id,
            "default_entity_id": f"sensor.{room}_ai_summary",
            "state_topic": f"intellithings/ai/rooms/{room}/summary",
            "value_template": "{{ value_json.text }}",
            "json_attributes_topic": f"intellithings/ai/rooms/{room}/summary",
            "json_attributes_template": "{{ {'ts': value_json.ts} | tojson }}",
            "icon": "mdi:brain",
        }


def main():
    load_dotenv(Path(__file__).resolve().parents[3] / ".env")
    state_home = Path(os.getenv("XDG_STATE_HOME") or Path.home() / ".local/state")
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--inference-file", type=Path,
                        default=state_home / "intellithings/mcp/inference.json")
    parser.add_argument("--publish", action="store_true")
    args = parser.parse_args()
    messages = list(discovery_messages(load_response(args.inference_file).room_summaries))
    if not args.publish:
        print(json.dumps(dict(messages), indent=2, ensure_ascii=False))
        return
    token = os.getenv("HA_LONG_LIVED_TOKEN", "").strip()
    if not token:
        parser.error("Set HA_LONG_LIVED_TOKEN in .env or the environment")
    base = os.getenv("HA_BASE_URL", "http://127.0.0.1:8123").rstrip("/")
    published = 0
    with httpx.Client(base_url=base, headers={"Authorization": f"Bearer {token}"}, timeout=30) as client:
        response = client.get("/api/states")
        response.raise_for_status()
        existing = {state["entity_id"] for state in response.json()}
        for topic, config in messages:
            entity = config["default_entity_id"]
            if entity in existing:
                print(f"Already registered: {entity}")
                continue
            response = client.post("/api/services/mqtt/publish", json={
                "topic": topic, "payload": json.dumps(config), "qos": 1, "retain": True,
            })
            response.raise_for_status()
            published += 1
            print(f"Published discovery: {entity}")
    print(f"Published {published} missing room summary configurations.")


if __name__ == "__main__":
    main()
