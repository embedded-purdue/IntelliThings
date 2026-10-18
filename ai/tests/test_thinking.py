"""Shared inference status follows real work and resets on failure/cancellation."""

import asyncio
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import AsyncMock, patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from llm import inference
from fixtures import ROOM_SUMMARIES

VALUE = {'summary': 'Home summary.', 'room_summaries': ROOM_SUMMARIES}


class ThinkingTests(unittest.IsolatedAsyncioTestCase):
    async def test_status_order_and_cleanup_on_all_inference_outcomes(self):
        for outcome in ['success', 'provider_error', 'invalid', 'timeout', 'cancelled']:
            with self.subTest(outcome=outcome), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                prompt = root / 'prompt.txt'
                prompt.write_text('Return JSON')
                events = []

                async def status(base, token, state):
                    events.append(state)
                    return {'ok': True, 'state': state}

                async def complete(*args, **kwargs):
                    events.append('model')
                    if outcome == 'provider_error':
                        raise RuntimeError('provider failed')
                    if outcome == 'timeout':
                        await asyncio.sleep(1)
                    if outcome == 'cancelled':
                        raise asyncio.CancelledError()
                    return {'content': 'invalid' if outcome == 'invalid' else json.dumps(VALUE)}

                async def publish(*args):
                    events.append('summaries')
                    return {'ok': True}

                with patch.object(inference, 'publish_inference_state', side_effect=status), \
                     patch.object(inference, 'complete', side_effect=complete) as model, \
                     patch.object(inference, 'publish_mqtt_summaries', side_effect=publish), \
                     patch.object(inference, 'publish_summary', AsyncMock(return_value=None)):
                    call = inference.infer({'context': 'rooms', 'fetched_at': 'now'}, root, prompt,
                        'key', 'model', 0.05, 2048, ha_token='secret', ha_base_url='http://ha')
                    if outcome == 'cancelled':
                        with self.assertRaises(asyncio.CancelledError):
                            await call
                    else:
                        self.assertEqual(await call, 0 if outcome == 'success' else 1)
                model.assert_awaited_once()
                self.assertEqual(events[0], 'start thinking')
                self.assertEqual(events[-1], 'stop thinking')
                self.assertEqual('summaries' in events, outcome == 'success')
                status_file = json.loads((root / 'thinking_status.json').read_text())
                self.assertTrue(status_file['ok'])
                self.assertEqual([e['state'] for e in status_file['events']],
                                 ['start thinking', 'stop thinking'])

    async def test_status_failure_does_not_discard_inference_or_skip_final_reset(self):
        for failed_state in ['start thinking', 'stop thinking']:
            with self.subTest(failed_state=failed_state), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                prompt = root / 'prompt.txt'
                prompt.write_text('Return JSON')

                async def status(base, token, state):
                    if state == failed_state:
                        raise RuntimeError('offline secret')
                    return {'ok': True, 'state': state}

                with patch.object(inference, 'publish_inference_state', side_effect=status) as states, \
                     patch.object(inference, 'complete', AsyncMock(return_value={'content': json.dumps(VALUE)})), \
                     patch.object(inference, 'publish_mqtt_summaries', AsyncMock(return_value={'ok': True})), \
                     patch.object(inference, 'publish_summary', AsyncMock(return_value=None)):
                    result = await inference.infer({'context': 'rooms', 'fetched_at': 'now'}, root, prompt,
                        'key', 'model', 1, 2048, ha_token='secret', ha_base_url='http://ha')
                self.assertEqual(result, 1)
                self.assertEqual(states.await_count, 2)
                self.assertEqual(states.call_args.args[2], 'stop thinking')
                self.assertTrue(json.loads((root / 'inference_status.json').read_text())['ok'])
                status_text = (root / 'thinking_status.json').read_text()
                self.assertFalse(json.loads(status_text)['ok'])
                self.assertNotIn('secret', status_text)
