"""Response validation and failure preservation checks."""

import json
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import AsyncMock, patch

from pydantic import ValidationError

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from llm.response import load_response, parse_response, Recommendation
from llm import inference
from integrations import openrouter
import httpx


def fixture():
    return {"summary": "Bedroom is warm", "changes": [{
        "area": "Bedroom", "action": "Consider lowering the thermostat target",
        "target_value": 73, "unit": "°F",
        "reasoning": "A lower target may improve comfort; preference is assumed",
        "evidence": ["Bedroom temperature: 78 °F"],
    }], "missing_information": ["Thermostat availability"]}


class ResponseTests(unittest.IsolatedAsyncioTestCase):
    def test_load_and_extract_fields(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "inference.json"
            path.write_text(json.dumps({"content": json.dumps(fixture())}))
            response = load_response(path)
            self.assertEqual(response.changes[0].target_value, 73)
            self.assertEqual(response.changes[0].reasoning, fixture()["changes"][0]["reasoning"])
            self.assertEqual(response.missing_information, ["Thermostat availability"])
        value = fixture()
        value["changes"][0].update(target_value=None, unit=None)
        self.assertIsNone(parse_response(json.dumps(value)).changes[0].target_value)

    def test_rejects_missing_reasoning_wrong_types_and_non_json(self):
        missing = fixture()
        del missing["changes"][0]["reasoning"]
        wrong = fixture()
        wrong["changes"][0]["target_value"] = "seventy-three"
        extra = fixture()
        extra["execute"] = True
        for value in [missing, wrong, extra, [], {"summary": "incomplete"}]:
            with self.assertRaises(ValidationError):
                parse_response(json.dumps(value))
        with self.assertRaises(ValidationError):
            parse_response('```json\n{}\n```')

    async def test_bad_json_preserves_previous_inference(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            prompt = root / "prompt.txt"
            prompt.write_text("Return JSON")
            saved = root / "inference.json"
            saved.write_text("previous result")
            with patch.object(inference, "complete", AsyncMock(return_value={"content": "not JSON"})):
                code = await inference.infer({"context": "Bedroom", "fetched_at": "now"},
                                             root, prompt, "secret", "model", 1, 100)
            self.assertEqual(code, 1)
            self.assertEqual(saved.read_text(), "previous result")
            self.assertFalse(json.loads((root / "inference_status.json").read_text())["ok"])

    async def test_schema_sent_to_provider(self):
        real_client = httpx.AsyncClient
        def handler(request):
            body = json.loads(request.content)
            self.assertTrue(body["response_format"]["json_schema"]["strict"])
            self.assertEqual(body["response_format"]["json_schema"]["schema"], Recommendation.model_json_schema())
            self.assertTrue(body["provider"]["require_parameters"])
            return httpx.Response(200, json={"choices": [{"finish_reason": "stop",
                "message": {"content": json.dumps(fixture())}}]})
        with patch.object(openrouter.httpx, "AsyncClient", side_effect=lambda **kwargs:
                          real_client(transport=httpx.MockTransport(handler), **kwargs)):
            await openrouter.complete("key", "model", [], 1, 2048,
                                      response_schema=Recommendation.model_json_schema())
