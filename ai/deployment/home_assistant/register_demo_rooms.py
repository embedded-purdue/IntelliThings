#!/usr/bin/env python3
"""Register the demo-room MQTT topics with HA using retained MQTT discovery messages.

Field mapping: ai/docs/demo-device-inventory.md ("HA entity mapping").
"""

import argparse
import json
import os
from pathlib import Path

import httpx
from dotenv import load_dotenv

from register_emulator import METRICS

# topic name -> (device name, HA area); the master bedroom maps to the existing Bedroom area
ROOMS = {
    "master_bedroom": ("Master Bedroom", "Bedroom"),
    "second_bedroom": ("Second Bedroom", "Second Bedroom"),
    "kitchen": ("Kitchen", "Kitchen"),
    "living_room": ("Living Room", "Living Room"),
    "bathroom": ("Bathroom", "Bathroom"),
    "garage": ("Garage", "Garage"),
    "office": ("Office", "Office"),
    "laundry": ("Laundry", "Laundry"),
    "home": ("Home", None),
}

# Each room's device states. The 10 sensor readings are added to every room except "home"
# from register_emulator.METRICS (see room_fields).
ROOM_STATES = {
    "master_bedroom": ["light_on", "fan_on", "ac_mode", "ac_target_temperature_c", "blinds_position_pct",
                       "humidifier_on", "humidifier_target_humidity_pct", "window_open"],
    "second_bedroom": ["light_on", "fan_on", "ac_mode", "ac_target_temperature_c", "window_open"],
    "kitchen": ["light_on", "fan_on", "blinds_position_pct", "purifier_on", "window_open", "oven_on",
                "microwave_on"],
    "living_room": ["light_on", "fan_on", "ac_mode", "ac_target_temperature_c", "tv_on", "blinds_position_pct",
                    "vacuum_state", "window_open"],
    "bathroom": ["light_on", "exhaust_fan_on"],
    "garage": ["light_on", "fan_on", "door_open", "car"],
    "office": ["light_on", "fan_on"],
    "laundry": ["light_on", "fan_on", "washer_on", "dryer_on"],
    "home": ["light_on", "fan_on", "front_door_locked", "anyone_home", "front_door_light_on", "doorbell_motion"],
}

# JSON key -> (entity name, kind, HA device class, unit)
STATES = {
    "light_on": ("Light", "bool", None, None),
    "fan_on": ("Fan", "bool", None, None),
    "purifier_on": ("Purifier", "bool", None, None),
    "oven_on": ("Oven", "bool", None, None),
    "microwave_on": ("Microwave", "bool", None, None),
    "tv_on": ("TV", "bool", None, None),
    "exhaust_fan_on": ("Exhaust fan", "bool", None, None),
    "washer_on": ("Washer", "bool", None, None),
    "dryer_on": ("Dryer", "bool", None, None),
    "humidifier_on": ("Humidifier", "bool", None, None),
    "front_door_light_on": ("Front door light", "bool", None, None),
    "anyone_home": ("Anyone home", "bool", "occupancy", None),
    "doorbell_motion": ("Doorbell motion", "bool", "motion", None),
    "window_open": ("Window", "bool", "window", None),
    "door_open": ("Garage door", "bool", "garage_door", None),
    # HA's lock device class reports "on" for unlocked, so the value is inverted.
    "front_door_locked": ("Front door lock", "inverted_bool", "lock", None),
    "ac_mode": ("AC mode", "text", None, None),
    "car": ("Car", "text", None, None),
    "vacuum_state": ("Vacuum", "text", None, None),
    "ac_target_temperature_c": ("AC target temperature", "number", "temperature", "°C"),
    "humidifier_target_humidity_pct": ("Humidifier target humidity", "number", "humidity", "%"),
    "blinds_position_pct": ("Blinds position", "number", None, "%"),
}

SENSORS = {key: (name, "bool" if key == "presence" else "number", device_class, unit)
           for key, name, unit, device_class in METRICS}


def value_template(key, kind):
    present = f"value_json.{key} is defined"
    if kind == "bool":
        return "{{ ('ON' if value_json." + key + " else 'OFF') if " + present + " else '' }}"
    if kind == "inverted_bool":
        return "{{ ('OFF' if value_json." + key + " else 'ON') if " + present + " else '' }}"
    return "{{ value_json." + key + " if " + present + " else none }}"


def room_fields(room):
    fields = {key: STATES[key] for key in ROOM_STATES[room]}
    if room != "home":
        fields.update(SENSORS)
    return fields


def discovery_messages(rooms, only_fields=None):
    for room in rooms:
        device_name, area = ROOMS[room]
        topic = f"intellithings/demo/{room}"
        device = {
            "identifiers": [f"intellithings_demo_{room}"],
            "name": f"IntelliThings Demo {device_name}",
            "manufacturer": "IntelliThings",
            "model": "Demo room (simulated)",
        }
        if area:
            device["suggested_area"] = area
        for key, (name, kind, device_class, unit) in room_fields(room).items():
            if only_fields and key not in only_fields:
                continue
            component = "sensor" if kind in ("number", "text") else "binary_sensor"
            unique_id = f"intellithings_demo_{room}_{key}"
            config = {
                "name": name,
                "unique_id": unique_id,
                "default_entity_id": f"{component}.{unique_id}",
                "state_topic": topic,
                "value_template": value_template(key, kind),
                "json_attributes_topic": topic,
                "json_attributes_template": "{{ {'ts': value_json.ts} | tojson if value_json.ts is defined else '{}' }}",
                "device": device,
            }
            if device_class:
                config["device_class"] = device_class
            if unit:
                config["unit_of_measurement"] = unit
            # Payloads stay in °C (docs/interfaces/mqtt.md); HA shows °F to people and the agent.
            if device_class == "temperature":
                config["suggested_unit_of_measurement"] = "°F"
            if component == "binary_sensor":
                config["payload_on"] = "ON"
                config["payload_off"] = "OFF"
            elif kind == "number":
                config["state_class"] = "measurement"
                config["suggested_display_precision"] = 1 if device_class in ("temperature", "humidity", "illuminance") else 0
            yield f"homeassistant/{component}/{unique_id}/config", config


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--publish", action="store_true", help="Publish through HA's configured MQTT integration")
    parser.add_argument("--remove", action="store_true", help="Publish empty configs, deleting these entities from HA")
    parser.add_argument("--rooms", nargs="+", choices=list(ROOMS), default=list(ROOMS), help="Rooms to include (default: all)")
    parser.add_argument("--fields", nargs="+", help="Only these JSON keys, e.g. temperature_c presence light_on")
    args = parser.parse_args()
    if args.publish and args.remove:
        parser.error("Use --publish or --remove, not both")

    messages = list(discovery_messages(args.rooms, set(args.fields) if args.fields else None))
    if not args.publish and not args.remove:
        print(json.dumps(dict(messages), indent=2, ensure_ascii=False))
        print(f"\n{len(messages)} discovery configurations (preview only; nothing published).")
        return

    load_dotenv(Path(__file__).resolve().parents[3] / ".env")
    token = os.getenv("HA_LONG_LIVED_TOKEN", "").strip()
    if not token:
        parser.error("Set HA_LONG_LIVED_TOKEN in .env or the environment")
    base = os.getenv("HA_BASE_URL", "http://127.0.0.1:8123").rstrip("/")
    with httpx.Client(base_url=base, headers={"Authorization": f"Bearer {token}"}, timeout=30) as client:
        for topic, config in messages:
            payload = "" if args.remove else json.dumps(config)
            response = client.post("/api/services/mqtt/publish", json={
                "topic": topic, "payload": payload, "qos": 1, "retain": True,
            })
            response.raise_for_status()
    action = "Removed" if args.remove else "Published"
    print(f"{action} {len(messages)} discovery configurations for {len(args.rooms)} demo rooms.")


if __name__ == "__main__":
    main()
