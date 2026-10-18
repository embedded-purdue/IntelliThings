#!/usr/bin/env python3
"""Register emulator readings with HA using retained MQTT discovery messages."""

import argparse
import json
import os
from pathlib import Path

import httpx
from dotenv import load_dotenv


NODES = {"emu-kitchen": "Kitchen", "emu-bedroom": "Bedroom", "emu-living-room": "Living Room"}
# JSON key, entity name, unit, HA device class
METRICS = [
    ("temperature_c", "Temperature", "°C", "temperature"),
    ("humidity_pct", "Humidity", "%", "humidity"),
    ("voc_index", "VOC index", None, None),
    ("co2_ppm", "CO2", "ppm", "carbon_dioxide"),
    ("lux", "Illuminance", "lx", "illuminance"),
    ("distance_mm", "Distance", "mm", "distance"),
    ("pm1_0_ugm3", "PM1", "µg/m³", "pm1"),
    ("pm2_5_ugm3", "PM2.5", "µg/m³", "pm25"),
    ("pm10_ugm3", "PM10", "µg/m³", "pm10"),
    ("presence", "Presence", None, "occupancy"),
]


def discovery_messages():
    for node_id, room in NODES.items():
        select_node = (
            "{% set node = value_json.nodes | default([]) | selectattr('id', 'equalto', '"
            + node_id + "') | first | default(none) %}"
        )
        for key, name, unit, device_class in METRICS:
            component = "binary_sensor" if key == "presence" else "sensor"
            unique_id = f"intellithings_{node_id.replace('-', '_')}_{key}"
            value = f"node.reading.{key}"
            config = {
                "name": name,
                "unique_id": unique_id,
                "default_entity_id": f"{component}.{unique_id}",
                "state_topic": "intellithings/emulator/state",
                "value_template": select_node + "{{ " + value + " if node is not none else none }}",
                "expire_after": 60,
                "availability_topic": "intellithings/emulator/state",
                "availability_template": select_node + (
                    "{{ 'online' if node is not none and node.mqtt == 'connected' "
                    "and node.last_publish_age_s | default(999) | float(999) < 60 else 'offline' }}"
                ),
                "device": {
                    "identifiers": [f"intellithings_{node_id}"],
                    "name": f"IntelliThings {room}",
                    "manufacturer": "IntelliThings",
                    "model": "Emulated sensor node",
                    "suggested_area": room,
                },
            }
            if device_class:
                config["device_class"] = device_class
            if unit:
                config["unit_of_measurement"] = unit
            if component == "binary_sensor":
                config["payload_on"] = "ON"
                config["payload_off"] = "OFF"
                config["value_template"] = select_node + (
                    "{{ ('ON' if node.reading.presence else 'OFF') if node is not none else '' }}"
                )
            else:
                config["state_class"] = "measurement"
                config["suggested_display_precision"] = 1 if key in {"temperature_c", "humidity_pct", "lux"} else 0
            yield f"homeassistant/{component}/{unique_id}/config", config


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--publish", action="store_true", help="Publish through HA's configured MQTT integration")
    args = parser.parse_args()
    messages = list(discovery_messages())
    if not args.publish:
        print(json.dumps(dict(messages), indent=2, ensure_ascii=False))
        return
    load_dotenv(Path(__file__).resolve().parents[2] / ".env")
    token = os.getenv("HA_LONG_LIVED_TOKEN", "").strip()
    if not token:
        parser.error("Set HA_LONG_LIVED_TOKEN in .env or the environment")
    base = os.getenv("HA_BASE_URL", "http://127.0.0.1:8123").rstrip("/")
    with httpx.Client(base_url=base, headers={"Authorization": f"Bearer {token}"}, timeout=30) as client:
        for topic, config in messages:
            response = client.post("/api/services/mqtt/publish", json={
                "topic": topic, "payload": json.dumps(config), "qos": 1, "retain": True,
            })
            response.raise_for_status()
    print(f"Published {len(messages)} discovery configurations for {len(NODES)} room devices.")


if __name__ == "__main__":
    main()
