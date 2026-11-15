"""Provider routing exercised through real inference and mocked HTTP/MCP I/O."""
import asyncio
from contextlib import asynccontextmanager
import json
import os
from pathlib import Path
import sys
import tempfile
import unittest
from types import SimpleNamespace
from unittest.mock import AsyncMock, patch

import httpx

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from agent import inference, pipeline
from fixtures import ROOM_SUMMARIES
from test_control import CALL, CONTEXT, session


def reply(content=None, calls=None):
    return {"id": "response-1", "choices": [{
        "finish_reason": "tool_calls" if calls else "stop",
        "message": {"role": "assistant", "content": content,
                    **({"tool_calls": calls} if calls else {})}}]}


SUMMARY = json.dumps({"summary": "Comfortable", "room_summaries": ROOM_SUMMARIES})


class GenAITests(unittest.IsolatedAsyncioTestCase):
    async def run_inference(self, handler, *, key="purdue-key", timeout=2, mcp=None):
        real_client = httpx.AsyncClient
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            prompt = root / "prompt.txt"
            prompt.write_text("Return home summaries")
            @asynccontextmanager
            async def connect(*args):
                yield mcp
            with patch.object(httpx, "AsyncClient", side_effect=lambda **kw:
                              real_client(transport=httpx.MockTransport(handler), **kw)), \
                 patch.object(inference, "connect", connect):
                result = await pipeline.run_inference(
                    {"context": CONTEXT, "fetched_at": "now"}, root, prompt,
                    "router-key", "router-model", timeout, 2048,
                    mcp_url="test" if mcp else None,
                    genai_api_key=key, genai_model="gpt-oss:120b")
            files = {p.name: json.loads(p.read_text()) for p in root.glob("*.json")}
            return result, files

    async def test_primary_success_uses_schema_and_no_openrouter(self):
        seen = []
        def handler(request):
            seen.append(request.url.host)
            body = json.loads(request.content)
            self.assertEqual(request.headers["Authorization"], "Bearer purdue-key")
            self.assertEqual(request.url.path, "/api/chat/completions")
            self.assertEqual(body["model"], "gpt-oss:120b")
            self.assertEqual(body["response_format"]["type"], "json_schema")
            self.assertNotIn("provider", body)
            self.assertNotIn("reasoning", body)
            return httpx.Response(200, json=reply(SUMMARY))
        code, files = await self.run_inference(handler)
        self.assertEqual(code, 0)
        self.assertEqual(seen, ["genai.rcac.purdue.edu"])
        self.assertEqual(files["inference.json"]["provider"], "genai")
        self.assertEqual(files["inference.json"]["requested_model"], "gpt-oss:120b")

    async def test_model_failures_fall_back_and_record_reason(self):
        failures = [httpx.Response(503), httpx.Response(401),
                    httpx.Response(200, json=None), httpx.Response(200, json={"error": "bad"}),
                    httpx.Response(200, json=reply("not JSON")),
                    httpx.Response(200, json=reply('{"summary":"missing rooms"}')),
                    httpx.Response(200, json=reply("")),
                    httpx.Response(200, json={"choices": []}),
                    httpx.ConnectError("purdue-key must not leak")]
        for failure in failures:
            with self.subTest(failure=failure):
                seen = []
                def handler(request):
                    seen.append(request.url.host)
                    if request.url.host == "genai.rcac.purdue.edu":
                        if isinstance(failure, Exception):
                            raise failure
                        return failure
                    self.assertEqual(request.headers["Authorization"], "Bearer router-key")
                    return httpx.Response(200, json=reply(SUMMARY))
                code, files = await self.run_inference(handler)
                self.assertEqual(code, 0)
                self.assertEqual(seen, ["genai.rcac.purdue.edu", "openrouter.ai"])
                record = files["inference.json"]
                self.assertEqual(record["provider"], "openrouter")
                self.assertTrue(record["requests"][0]["error"])
                self.assertNotIn("purdue-key", json.dumps(files))

    async def test_timeout_leaves_time_for_fallback(self):
        async def handler(request):
            if request.url.host == "genai.rcac.purdue.edu":
                await asyncio.sleep(1)
            return httpx.Response(200, json=reply(SUMMARY))
        code, files = await self.run_inference(handler, timeout=.2)
        self.assertEqual(code, 0)
        self.assertEqual(files["inference.json"]["provider"], "openrouter")
        self.assertEqual(files["inference.json"]["requests"][0]["error"], "TimeoutError")

    async def test_switch_stays_on_openrouter_for_tool_followup(self):
        seen = []
        def handler(request):
            seen.append(request.url.host)
            if len(seen) == 1:
                return httpx.Response(503)
            return httpx.Response(200, json=reply(calls=[CALL]) if len(seen) == 2 else reply(SUMMARY))
        code, files = await self.run_inference(handler, mcp=session())
        self.assertEqual(code, 0)
        self.assertEqual(seen, ["genai.rcac.purdue.edu", "openrouter.ai", "openrouter.ai"])
        self.assertEqual(len(files["inference.json"]["actions"]), 1)

    async def test_primary_budget_is_shared_across_tool_requests(self):
        seen = []
        async def handler(request):
            seen.append(request.url.host)
            if request.url.host == "genai.rcac.purdue.edu":
                await asyncio.sleep(.12)
            return httpx.Response(200, json=reply(calls=[CALL]) if len(seen) == 1 else reply(SUMMARY))
        code, files = await self.run_inference(handler, timeout=.4, mcp=session())
        self.assertEqual(code, 0)
        self.assertEqual(seen, ["genai.rcac.purdue.edu", "genai.rcac.purdue.edu", "openrouter.ai"])
        self.assertEqual(files["inference.json"]["requests"][1]["error"], "TimeoutError")

    async def test_cycle_loads_primary_key_and_default_model_from_environment(self):
        real_client = httpx.AsyncClient
        def handler(request):
            self.assertEqual(request.url.host, "genai.rcac.purdue.edu")
            self.assertEqual(request.headers["Authorization"], "Bearer env-key")
            self.assertEqual(json.loads(request.content)["model"], "gpt-oss:120b")
            return httpx.Response(200, json=reply(SUMMARY))
        @asynccontextmanager
        async def connect(*args):
            yield session()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            prompt = root / "prompt.txt"
            prompt.write_text("Return home summaries")
            args = SimpleNamespace(url="test", timeout=2, output_dir=root, collect_only=False,
                                   prompt_file=prompt, llm_timeout=2, max_tokens=2048)
            with patch.dict(os.environ, {"GENAI_API_KEY": " env-key ", "GENAI_MODEL": " "}, clear=True), \
                 patch.object(pipeline, "collect_snapshot", AsyncMock(return_value={"context": CONTEXT, "fetched_at": "now"})), \
                 patch.object(pipeline, "set_thinking_state", AsyncMock(return_value=True)), \
                 patch.object(pipeline, "publish_outputs", AsyncMock(return_value=True)), \
                 patch.object(inference, "connect", connect), \
                 patch.object(httpx, "AsyncClient", side_effect=lambda **kw:
                              real_client(transport=httpx.MockTransport(handler), **kw)):
                self.assertEqual(await pipeline.run_cycle(args), 0)
            self.assertEqual(json.loads((root / "inference_status.json").read_text())["provider"], "genai")

    async def test_missing_key_skips_purdue(self):
        seen = []
        def handler(request):
            seen.append(request.url.host)
            return httpx.Response(200, json=reply(SUMMARY))
        code, _ = await self.run_inference(handler, key="")
        self.assertEqual(code, 0)
        self.assertEqual(seen, ["openrouter.ai"])

    async def test_fallback_retains_tool_result_without_repeating_action(self):
        s = session()
        seen = []
        def handler(request):
            seen.append(request.url.host)
            body = json.loads(request.content)
            if len(seen) == 1:
                return httpx.Response(200, json=reply(calls=[CALL]))
            if request.url.host == "genai.rcac.purdue.edu":
                return httpx.Response(500)
            self.assertEqual(body["tool_choice"], "none")
            self.assertEqual(body["messages"][-1]["role"], "tool")
            self.assertEqual(json.loads(body["messages"][-1]["content"])["status"], "returned")
            return httpx.Response(200, json=reply(SUMMARY))
        code, files = await self.run_inference(handler, mcp=s)
        self.assertEqual(code, 0)
        self.assertEqual(s.call_tool.await_count, 1)
        self.assertEqual(len(files["inference.json"]["actions"]), 1)
        self.assertEqual(seen, ["genai.rcac.purdue.edu", "genai.rcac.purdue.edu", "openrouter.ai"])

    async def test_invalid_tool_target_falls_back_before_execution(self):
        s = session()
        bad = {**CALL, "function": {"name": "forbidden", "arguments": "{}"}}
        def handler(request):
            return httpx.Response(200, json=reply(calls=[bad]) if
                request.url.host == "genai.rcac.purdue.edu" else reply(SUMMARY))
        code, files = await self.run_inference(handler, mcp=s)
        self.assertEqual(code, 0)
        self.assertEqual(s.call_tool.await_count, 0)
        self.assertEqual(files["inference.json"]["provider"], "openrouter")

    async def test_failed_actuator_is_never_retried_through_fallback(self):
        s = session()
        s.call_tool.side_effect = TimeoutError()
        seen = []
        def handler(request):
            seen.append(request.url.host)
            return httpx.Response(200, json=reply(calls=[CALL]))
        code, files = await self.run_inference(handler, mcp=s)
        self.assertEqual(code, 1)
        self.assertEqual(seen, ["genai.rcac.purdue.edu"])
        self.assertEqual(files["actions.json"]["actions"][0]["status"], "unknown")

    async def test_both_fail_skips_publication(self):
        code, files = await self.run_inference(lambda request: httpx.Response(503))
        self.assertEqual(code, 1)
        self.assertNotIn("inference.json", files)
        self.assertTrue(files["mqtt_status.json"]["skipped"])

    async def test_cancellation_is_not_a_provider_failure(self):
        seen = []
        def handler(request):
            seen.append(request.url.host)
            raise asyncio.CancelledError()
        with self.assertRaises(asyncio.CancelledError):
            await self.run_inference(handler)
        self.assertEqual(seen, ["genai.rcac.purdue.edu"])
