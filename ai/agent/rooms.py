"""Discover summary destinations from the areas in each fresh HA context."""

import hashlib
import re
import unicodedata

from agent.controls import context_entities


def room_key(name):
    normalized = unicodedata.normalize('NFKD', name).encode('ascii', 'ignore').decode()
    key = re.sub(r'[^a-z0-9]+', '_', normalized.lower()).strip('_')
    if not key:
        key = 'area_' + hashlib.sha256(name.encode()).hexdigest()[:12]
    if not key[0].isalpha():
        key = 'area_' + key
    return key


def discover_rooms(context, configured_rooms=()):
    """Use HA area membership, never guess rooms from device name substrings.

    Configured display destinations remain present even when their sensors are
    absent. Entities without areas remain in the full context for the overview.
    """
    rooms = {}
    for entity in context_entities(context):
        if not isinstance(entity, dict):
            continue
        areas = entity.get('areas') or []
        if isinstance(areas, str):
            areas = [areas]
        if not isinstance(areas, list):
            raise ValueError('Unexpected HA area format; cannot determine room coverage')
        for area in areas:
            if not isinstance(area, str) or not area.strip():
                raise ValueError('HA area names must be nonempty strings')
            area = area.strip()
            key = room_key(area)
            if key in rooms and rooms[key]['name'] != area:
                raise ValueError(f'HA area names collide on summary key {key!r}')
            room = rooms.setdefault(key, {'name': area, 'entity_count': 0})
            room['entity_count'] += 1
    for key in configured_rooms:
        rooms.setdefault(key, {'name': key.replace('_', ' ').title(), 'entity_count': 0})
    return dict(sorted(rooms.items()))
