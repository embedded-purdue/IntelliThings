"""Dashboard publication and inference failure separation."""
import json
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import AsyncMock, patch

import httpx
from pydantic import ValidationError

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from integrations import dashboard
from llm import inference
from llm.response import Recommendation


class DashboardTests(unittest.IsolatedAsyncioTestCase):
    async def test_exact_summary_is_published_and_verified(self):
        real_client = httpx.AsyncClient
        seen = []
        def handler(request):
            seen.append(request.method)
            if request.method == 'POST':
                self.assertEqual(request.url.path, '/api/services/input_text/set_value')
                self.assertEqual(json.loads(request.content), {
                    'entity_id': 'input_text.ai_summary_overall', 'value': 'Room is warm.'})
                return httpx.Response(200, json=[])
            return httpx.Response(200, json={'state': 'Room is warm.'})
        with patch.object(dashboard.httpx, 'AsyncClient', side_effect=lambda **kwargs:
                          real_client(transport=httpx.MockTransport(handler), **kwargs)):
            await dashboard.publish_summary('http://ha', 'secret', 'Room is warm.')
        self.assertEqual(seen, ['POST', 'GET'])

    def test_summary_length_is_validated(self):
        with self.assertRaises(ValidationError):
            Recommendation(summary='x'*256, changes=[], missing_information=[])
        self.assertEqual(len(Recommendation(summary='x'*255, changes=[], missing_information=[]).summary),255)

    async def test_publication_failure_preserves_successful_inference(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            prompt = root/'prompt.txt'
            prompt.write_text('Return JSON')
            value = {'summary':'Room is warm.', 'changes':[], 'missing_information':[]}
            with patch.object(inference, 'complete', AsyncMock(return_value={'content':json.dumps(value)})), \
                 patch.object(inference, 'publish_summary', AsyncMock(side_effect=RuntimeError('offline secret'))) as publish:
                result = await inference.infer({'context':'room', 'fetched_at':'now'}, root, prompt,
                    'key', 'model', 2, 2048, ha_token='secret', ha_base_url='http://ha')
            self.assertEqual(result,1)
            publish.assert_awaited_once_with('http://ha','secret','Room is warm.')
            self.assertTrue(json.loads((root/'inference_status.json').read_text())['ok'])
            self.assertEqual(json.loads((root/'inference.json').read_text())['recommendation'],value)
            status=(root/'dashboard_status.json').read_text()
            self.assertFalse(json.loads(status)['ok'])
            self.assertNotIn('secret',status)
