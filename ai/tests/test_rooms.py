"""Room discovery, exact per-cycle coverage, and routing of newly added areas."""
import json
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import AsyncMock, patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from agent.rooms import discover_rooms
from agent.response import parse_response, response_schema, load_response
from agent.inference import infer
from agent.integrations import mqtt

CONTEXT = """Live Context: home:
- names: Demo Master Bedroom Temperature
  domain: sensor
  state: '70'
  areas: Bedroom
- names: Demo Second Bedroom Temperature
  domain: sensor
  state: '75'
  areas: Second Bedroom
- names: Demo Garage Temperature
  domain: sensor
  state: unavailable
  areas: Garage
- names: Front door
  domain: binary_sensor
  state: 'off'
"""


class RoomsTests(unittest.IsolatedAsyncioTestCase):
    def test_area_membership_and_missing_data(self):
        rooms = discover_rooms(CONTEXT, ['kitchen'])
        self.assertEqual(set(rooms), {'bedroom', 'second_bedroom', 'garage', 'kitchen'})
        self.assertEqual(rooms['second_bedroom'], {'name': 'Second Bedroom', 'entity_count': 1})
        self.assertEqual(rooms['kitchen']['entity_count'], 0)
        self.assertEqual(rooms['garage']['entity_count'], 1)
        self.assertNotIn('front_door', rooms)

    def test_topic_safe_keys_and_collision_rejection(self):
        text = CONTEXT.replace('Second Bedroom', 'Second / Bedroom+#')
        self.assertIn('second_bedroom', discover_rooms(text))
        with self.assertRaisesRegex(ValueError, 'collide'):
            discover_rooms(CONTEXT + "- names: Duplicate\n  areas: Second-Bedroom\n")
        text = CONTEXT + "- names: Extra\n  areas: [Office, Laundry]\n"
        self.assertTrue({'office', 'laundry'} <= set(discover_rooms(text)))

    def test_schema_and_validation_require_every_discovered_room(self):
        rooms = discover_rooms(CONTEXT)
        schema = response_schema(rooms)['properties']['room_summaries']
        self.assertEqual(set(schema['required']), set(rooms))
        self.assertFalse(schema['additionalProperties'])
        value = {'summary': 'Mixed conditions', 'room_summaries': {
            key: 'Sensor data unavailable' for key in rooms}}
        parse_response(json.dumps(value), rooms)
        del value['room_summaries']['second_bedroom']
        with self.assertRaisesRegex(ValueError, 'second_bedroom'):
            parse_response(json.dumps(value), rooms)
        value['room_summaries']['invented'] = 'Good'
        with self.assertRaises(ValueError):
            parse_response(json.dumps(value), rooms)
        with self.assertRaises(ValueError):
            parse_response('{"summary":"x","room_summaries":{"bad/#":"x"}}')

    async def test_new_room_reaches_model_saved_response_and_mqtt(self):
        rooms = discover_rooms(CONTEXT)
        value = {'summary': 'Garage readings missing', 'room_summaries': {
            'bedroom': 'Temperature looks comfortable',
            'second_bedroom': 'Warm; consider ventilation',
            'garage': 'Sensor data unavailable'}}
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            prompt = root / 'prompt.txt'
            prompt.write_text('Summarize every room in room_inventory')
            with patch('agent.inference.load_nodes', return_value={}), \
                 patch('agent.inference.complete', AsyncMock(return_value={
                     'content': json.dumps(value)})) as model:
                recommendation, record = await infer({'context': CONTEXT, 'fetched_at': 'now'},
                    root, prompt, 'secret', 'model', 2, 2048)
            request = json.loads(model.call_args.args[2][1]['content'])
            self.assertEqual(request['room_inventory'], rooms)
            self.assertIn('Front door', request['context'])
            self.assertEqual(model.call_args.kwargs['response_schema'], response_schema(rooms))
            path = root / 'inference.json'
            path.write_text(json.dumps(record))
            self.assertEqual(load_response(path).room_summaries, value['room_summaries'])
            with patch.object(mqtt, 'load_nodes', return_value={'demo-second-bedroom': 'second_bedroom'}), \
                 patch.object(mqtt, '_publish_messages', AsyncMock(return_value={'ok': True})) as publish:
                await mqtt.publish_summaries('http://ha', 'secret', recommendation, 'now')
            messages = {m['topic']: json.loads(m['payload']) for m in publish.call_args.args[2]}
            self.assertEqual(messages['intellithings/ai/rooms/second_bedroom/summary'],
                             {'text': value['room_summaries']['second_bedroom'], 'ts': 'now'})
            self.assertEqual(messages['intellithings/demo-second-bedroom/display'],
                             messages['intellithings/ai/rooms/second_bedroom/summary'])
