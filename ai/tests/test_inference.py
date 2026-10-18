"""Inference transport and pipeline regression tests; no external requests."""

import json
import os
import sys
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import AsyncMock, patch

import httpx

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import main
from integrations import openrouter
from llm import inference


class InferenceTests(unittest.IsolatedAsyncioTestCase):
    async def test_transport_payload_and_errors(self):
        real_client = httpx.AsyncClient
        response_body = {"model": "openai/gpt-5.4-mini", "choices": [{
            "finish_reason": "stop", "message": {"content": "Room summary"}
        }]}
        def handler(request):
            body = json.loads(request.content)
            self.assertEqual(request.headers["Authorization"], "Bearer secret")
            self.assertEqual(body["model"], "openai/gpt-5.4-mini")
            self.assertEqual(body["max_tokens"], 2048)
            self.assertNotIn("tools", body)
            return httpx.Response(200, json=response_body)
        with patch.object(openrouter.httpx, "AsyncClient", side_effect=lambda **kwargs:
                          real_client(transport=httpx.MockTransport(handler), **kwargs)):
            result = await openrouter.complete("secret", "openai/gpt-5.4-mini", [], 1, 2048)
            self.assertEqual(result["content"], "Room summary")
            for bad in [
                {"error": {"message": "provider failure"}},
                {"choices": []},
                {"choices": [{"finish_reason": "length", "message": {"content": "partial"}}]},
                {"choices": [{"finish_reason": "stop", "message": {"content": ""}}]},
            ]:
                response_body = bad
                with self.assertRaises((ValueError, RuntimeError)):
                    await openrouter.complete("secret", "openai/gpt-5.4-mini", [], 1, 2048)

    async def test_persistence_prompt_and_failure_preserves_result(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory)
            prompt = output / "prompt.txt"
            prompt.write_text("My editable prompt")
            snapshot = {"fetched_at": "test-time", "context": "Kitchen: 22 C", "mcp_result": "duplicate"}
            mock = AsyncMock(return_value={"model": "test-model", "content": json.dumps({
                "summary": "Comfortable", "changes": [], "missing_information": []
            })})
            with patch.object(inference, "complete", mock):
                self.assertEqual(await inference.infer(snapshot, output, prompt, "secret", "test-model", 1, 100), 0)
            messages = mock.call_args.args[2]
            self.assertEqual(messages[0]["content"], "My editable prompt")
            self.assertNotIn("mcp_result", json.loads(messages[1]["content"]))
            saved = output / "inference.json"
            previous = saved.read_bytes()
            self.assertEqual(saved.stat().st_mode & 0o777, 0o600)
            self.assertEqual(json.loads(previous)["input_fetched_at"], "test-time")
            with patch.object(inference, "complete", AsyncMock(side_effect=RuntimeError("secret failed"))):
                self.assertEqual(await inference.infer(snapshot, output, prompt, "secret", "test-model", 1, 100), 1)
            self.assertEqual(previous, saved.read_bytes())
            status = (output / "inference_status.json").read_text()
            self.assertFalse(json.loads(status)["ok"])
            self.assertNotIn("secret", status)

    async def test_pipeline_uses_fresh_snapshot_and_skips_failed_collection(self):
        with tempfile.TemporaryDirectory() as directory:
            args = SimpleNamespace(url="test", timeout=1, output_dir=Path(directory),
                                   collect_only=False, prompt_file=Path("prompt.txt"),
                                   llm_timeout=1, max_tokens=100)
            snapshot = {"fetched_at": "fresh", "context": "new"}
            with patch.object(main, "collect_snapshot", AsyncMock(return_value=snapshot)), \
                 patch.object(main, "infer", AsyncMock(return_value=0)) as infer:
                self.assertEqual(await main.run_cycle(args), 0)
                self.assertIs(infer.call_args.args[0], snapshot)
            with patch.object(main, "collect_snapshot", AsyncMock(return_value=None)), \
                 patch.object(main, "infer", AsyncMock()) as infer:
                self.assertEqual(await main.run_cycle(args), 1)
                infer.assert_not_called()
                self.assertTrue(json.loads((args.output_dir / "inference_status.json").read_text())["skipped"])


if __name__ == "__main__":
    unittest.main()
