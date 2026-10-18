"""Emulator payload limits, MQTT delivery requests, and output independence."""

import asyncio
import json
from pathlib import Path
import sys
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import AsyncMock, patch

import httpx

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from integrations import mqtt
from llm import inference
from fixtures import ROOM_SUMMARIES
from llm.response import Recommendation
import main

STAMP = '2026-10-18T12:00:00+00:00'


class PayloadTests(unittest.TestCase):
    def test_short_summary_is_preserved(self):
        payload, truncated = mqtt.display_payload('Room is warm.', STAMP)
        self.assertFalse(truncated)
        self.assertEqual(json.loads(payload), {'text': 'Room is warm.', 'ts': STAMP})

    def test_byte_limits_include_unicode_and_json_escaping(self):
        for summary in ['🌡' * 255, '暖' * 255, '\x01' * 255, '"\\' * 127]:
            with self.subTest(summary=repr(summary[:4])):
                payload, truncated = mqtt.display_payload(summary, STAMP)
                value = json.loads(payload)
                self.assertTrue(truncated)
                self.assertLessEqual(len(payload.encode('utf-8')), 512)
                self.assertLessEqual(len(value['text'].encode('utf-8')), 400)
                self.assertTrue(value['text'].endswith('…'))
                self.assertTrue(summary.startswith(value['text'][:-1]))
                self.assertEqual(value['ts'], STAMP)

    def test_node_config_validates_room_mapping(self):
        with tempfile.TemporaryDirectory() as directory:
            config = Path(directory) / 'nodes.json'
            with patch.object(mqtt, 'CONFIG', config):
                for nodes in [['emu-kitchen'], {'node/#': 'kitchen'}, {'+': 'kitchen'},
                              {'': 'kitchen'}, {'node': True}, {'node': 'garage'}]:
                    config.write_text(json.dumps(nodes))
                    with self.assertRaises(ValueError):
                        mqtt.load_nodes()
                config.write_text('{"emu-kitchen":"kitchen"}')
                self.assertEqual(mqtt.load_nodes(), {'emu-kitchen': 'kitchen'})


class PublicationTests(unittest.IsolatedAsyncioTestCase):
    async def test_publish_all_nodes_despite_failure_and_timeout(self):
        real_client = httpx.AsyncClient
        seen = []
        expected = {
            'intellithings/ai/summary': 'Warm.',
            'intellithings/good/display': ROOM_SUMMARIES['kitchen'],
            'intellithings/bad/display': ROOM_SUMMARIES['bedroom'],
            'intellithings/slow/display': ROOM_SUMMARIES['living_room'],
        }

        async def handler(request):
            self.assertEqual(request.url.path, '/api/services/mqtt/publish')
            self.assertEqual(request.headers['Authorization'], 'Bearer secret')
            body = json.loads(request.content)
            seen.append(body)
            self.assertEqual(body['qos'], 1)
            self.assertIs(body['retain'], True)
            self.assertEqual(json.loads(body['payload']), {'text': expected[body['topic']], 'ts': STAMP})
            if body['topic'].endswith('/bad/display'):
                return httpx.Response(500)
            if body['topic'].endswith('/slow/display'):
                await asyncio.sleep(1)
            return httpx.Response(200, json=[])

        with patch.object(mqtt, 'load_nodes', return_value={
            'good': 'kitchen', 'bad': 'bedroom', 'slow': 'living_room'}), \
             patch.object(mqtt.httpx, 'AsyncClient', side_effect=lambda **kwargs:
                          real_client(transport=httpx.MockTransport(handler), **kwargs)):
            recommendation = Recommendation(summary='Warm.', room_summaries=ROOM_SUMMARIES)
            result = await mqtt.publish_summaries('http://ha', 'secret', recommendation, STAMP, timeout=0.05)
        self.assertEqual({item['topic'] for item in seen}, set(expected))
        self.assertFalse(result['ok'])
        self.assertEqual([item['ok'] for item in result['topics']], [True, True, False, False])
        self.assertIn('TimeoutError', result['topics'][3]['error'])
        self.assertNotIn('secret', json.dumps(result))

    async def test_empty_nodes_still_publishes_full_overall_summary(self):
        with patch.object(mqtt, 'load_nodes', return_value={}), \
             patch.object(httpx.AsyncClient, 'post', AsyncMock(return_value=httpx.Response(
                 200, request=httpx.Request('POST', 'http://ha/api/services/mqtt/publish')))) as post:
            recommendation = Recommendation(summary='🌡' * 255, room_summaries=ROOM_SUMMARIES)
            result = await mqtt.publish_summaries('http://ha', 'secret', recommendation, STAMP)
        post.assert_awaited_once()
        body = post.call_args.kwargs['json']
        self.assertEqual(body['topic'], 'intellithings/ai/summary')
        self.assertEqual(json.loads(body['payload'])['text'], recommendation.summary)
        self.assertTrue(result['ok'])
        self.assertFalse(result['topics'][0]['truncated'])

    async def test_thinking_status_payload(self):
        with patch.object(mqtt, 'timestamp', return_value=STAMP), \
             patch.object(httpx.AsyncClient, 'post', AsyncMock(return_value=httpx.Response(
                 200, request=httpx.Request('POST', 'http://ha/api/services/mqtt/publish')))) as post:
            for state in ['start thinking', 'stop thinking']:
                result = await mqtt.publish_inference_state('http://ha', 'secret', state)
                self.assertEqual(post.call_args.kwargs['json'], {
                    'topic': 'intellithings/ai/inference_status',
                    'payload': json.dumps({'state': state, 'ts': STAMP}), 'qos': 1, 'retain': True})
                self.assertTrue(result['ok'])

    async def test_mqtt_failure_preserves_dashboard_and_inference(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            prompt = root / 'prompt.txt'
            prompt.write_text('Return JSON')
            value = {'summary': 'Room is warm.', 'room_summaries': ROOM_SUMMARIES}
            with patch.object(inference, 'publish_inference_state', AsyncMock(return_value={'ok': True})), \
                 patch.object(inference, 'complete', AsyncMock(return_value={'content': json.dumps(value)})) as complete, \
                 patch.object(inference, 'publish_summary', AsyncMock(return_value=None)) as dashboard, \
                 patch.object(inference, 'publish_mqtt_summaries', AsyncMock(return_value={
                     'ok': False, 'topics': [{'topic': 'intellithings/emu-kitchen/display', 'ok': False}]})) as publish:
                result = await inference.infer({'context': 'room', 'fetched_at': 'now'}, root,
                    prompt, 'key', 'model', 2, 2048, ha_token='secret', ha_base_url='http://ha')
            self.assertEqual(result, 1)
            complete.assert_awaited_once()
            dashboard.assert_awaited_once_with('http://ha', 'secret', value['summary'])
            saved = json.loads((root / 'inference.json').read_text())
            publish.assert_awaited_once_with('http://ha', 'secret', Recommendation(**value), saved['generated_at'])
            self.assertTrue(json.loads((root / 'inference_status.json').read_text())['ok'])
            self.assertTrue(json.loads((root / 'dashboard_status.json').read_text())['ok'])
            self.assertFalse(json.loads((root / 'mqtt_status.json').read_text())['ok'])

    async def test_failed_inference_skips_publication(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            prompt = root / 'prompt.txt'
            prompt.write_text('Return JSON')
            with patch.object(inference, 'publish_inference_state', AsyncMock(return_value={'ok': True})), \
                 patch.object(inference, 'complete', AsyncMock(return_value={'content': 'invalid'})), \
                 patch.object(inference, 'publish_mqtt_summaries', AsyncMock()) as publish:
                result = await inference.infer({'context': 'room', 'fetched_at': 'now'}, root,
                    prompt, 'key', 'model', 2, 2048, ha_token='secret', ha_base_url='http://ha')
            self.assertEqual(result, 1)
            publish.assert_not_awaited()
            self.assertEqual(json.loads((root / 'mqtt_status.json').read_text())['reason'], 'inference_failed')

    async def test_collection_only_and_collection_failure_skip_inference(self):
        for snapshot, collect_only, reason in [
            ({'context': 'room', 'fetched_at': 'now'}, True, 'collection_only'),
            (None, False, 'collection_failed'),
        ]:
            with self.subTest(reason=reason), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                args = SimpleNamespace(url='http://ha', timeout=2, output_dir=root, collect_only=collect_only)
                with patch.object(main, 'set_thinking_state', AsyncMock(return_value=True)) as reset, \
                     patch.object(main, 'collect_snapshot', AsyncMock(return_value=snapshot)), \
                     patch.object(main, 'infer', AsyncMock()) as infer:
                    await main.run_cycle(args)
                infer.assert_not_awaited()
                if collect_only:
                    reset.assert_not_awaited()
                else:
                    reset.assert_awaited_once()
                    self.assertEqual(reset.call_args.args[2], 'stop thinking')
                self.assertEqual(json.loads((root / 'mqtt_status.json').read_text())['reason'], reason)
