"""Stage ordering and persistence across a complete cycle without external requests."""

from contextlib import asynccontextmanager
import json
from pathlib import Path
import sys
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import AsyncMock, patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from agent import inference, pipeline, publishing
from fixtures import ROOM_SUMMARIES


class PipelineTests(unittest.IsolatedAsyncioTestCase):
    async def test_cycle_saves_response_before_outputs_and_stops_afterwards(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            prompt = root / 'prompt.txt'
            prompt.write_text('Return summaries')
            args = SimpleNamespace(url='http://ha/api/mcp', timeout=1, output_dir=root,
                collect_only=False, prompt_file=prompt, llm_timeout=1, max_tokens=2048)
            events = []
            value = {'summary': 'Overall summary', 'room_summaries': ROOM_SUMMARIES}

            async def collect(*args):
                events.append('collect')
                return {'context': 'Kitchen: 22 C', 'fetched_at': 'now'}

            async def state(base, token, name):
                events.append(name)
                return {'ok': True, 'state': name}

            async def model(*args, **kwargs):
                events.append('model')
                return {'content': json.dumps(value)}

            async def output(*args):
                saved = json.loads((root / 'inference.json').read_text())
                self.assertEqual(saved['recommendation'], value)
                self.assertTrue(json.loads((root / 'inference_status.json').read_text())['ok'])
                events.append('output')
                return {'ok': True}

            @asynccontextmanager
            async def connect(*args):
                yield SimpleNamespace(list_tools=AsyncMock(return_value=SimpleNamespace(
                    tools=[], nextCursor=None)))

            with patch.object(pipeline, 'collect_snapshot', side_effect=collect), \
                 patch.object(inference, 'connect', connect), \
                 patch.object(inference, 'complete', side_effect=model) as complete, \
                 patch.object(publishing, 'publish_inference_state', side_effect=state), \
                 patch.object(publishing, 'publish_summary', side_effect=output), \
                 patch.object(publishing, 'publish_mqtt_summaries', side_effect=output):
                self.assertEqual(await pipeline.run_cycle(args), 0)
            complete.assert_awaited_once()
            self.assertCountEqual(events[:2], ['collect', 'stop thinking'])
            self.assertEqual(events[2:], ['start thinking', 'model', 'output', 'output', 'stop thinking'])
            for name in ['mqtt_status.json', 'dashboard_status.json', 'thinking_status.json']:
                self.assertTrue(json.loads((root / name).read_text())['ok'])
