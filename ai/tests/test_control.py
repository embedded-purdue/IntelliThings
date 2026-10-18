"""MCP target restrictions, action journal, and model/tool round trip."""
import json
import sys
import tempfile
import unittest
from contextlib import asynccontextmanager
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import AsyncMock, patch

from mcp.types import CallToolResult, TextContent

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from llm.control import Controller, TURN_ON, visible_actuators
from llm import inference

CONTEXT = "Live Context: home:\n- names: Turn Cooling On\n  domain: input_boolean\n  state: 'off'\n"
TARGET = {"name": "Turn Cooling On", "domain": "input_boolean"}
CALL = {"id": "call_1", "type": "function", "function": {
    "name": TURN_ON, "arguments": json.dumps({"name": "Turn Cooling On", "domain": ["input_boolean"]})}}


def session():
    return SimpleNamespace(
        list_tools=AsyncMock(return_value=SimpleNamespace(nextCursor=None, tools=[SimpleNamespace(
            name=TURN_ON, description="Turn on", inputSchema={"properties": {"name": {}, "domain": {}}})])),
        call_tool=AsyncMock(return_value=CallToolResult(content=[TextContent(type="text", text='{"success": true}')])))


class ControlTests(unittest.IsolatedAsyncioTestCase):
    def test_only_visible_configured_targets(self):
        self.assertEqual(visible_actuators(CONTEXT, [TARGET])[0][0]["state"], "off")
        self.assertEqual(visible_actuators("Sensors only", [TARGET]), ([], [TARGET]))
        self.assertEqual(visible_actuators(CONTEXT.replace("'off'", "'unavailable'"), [TARGET]), ([], [TARGET]))

    async def test_target_validation_and_journal(self):
        with tempfile.TemporaryDirectory() as directory:
            s = session()
            c = Controller(s, Path(directory))
            await c.discover({"context": CONTEXT, "fetched_at": "now"})
            for args in [{"area": "Bedroom"}, {"name": "Other device", "domain": ["input_boolean"]},
                         {"name": "Turn Cooling On", "domain": ["light"]}]:
                bad = {**CALL, "function": {"name": TURN_ON, "arguments": json.dumps(args)}}
                with self.assertRaises(ValueError):
                    await c.execute(bad)
            s.call_tool.assert_not_called()
            await c.execute(CALL)
            s.call_tool.assert_awaited_once_with(TURN_ON, arguments={"name": "Turn Cooling On", "domain": ["input_boolean"]})
            self.assertEqual(json.loads((Path(directory)/"actions.json").read_text())["actions"][0]["status"], "returned")
            with self.assertRaises(ValueError):
                await c.execute(CALL)

    async def test_tool_error_and_uncertain_timeout_are_recorded(self):
        for failure in (False, True):
            with tempfile.TemporaryDirectory() as directory:
                s = session()
                c = Controller(s, Path(directory))
                await c.discover({"context": CONTEXT, "fetched_at": "now"})
                if failure:
                    s.call_tool.side_effect = TimeoutError()
                    with self.assertRaises(TimeoutError):
                        await c.execute(CALL)
                    self.assertEqual(c.actions[0]["status"], "unknown")
                else:
                    s.call_tool.return_value = CallToolResult(content=[TextContent(type="text", text='{"success":false}')])
                    self.assertEqual((await c.execute(CALL))["status"], "error")

    async def test_model_tool_result_then_json_round_trip(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            prompt = root / "prompt.txt"
            prompt.write_text("Use available tools then return JSON")
            s = session()
            @asynccontextmanager
            async def connect(*args):
                yield s
            first = {"tool_calls": [CALL], "message": {"role": "assistant", "content": None, "tool_calls": [CALL]}}
            final = {"content": json.dumps({"summary": "Flag turned on", "changes": [], "missing_information": []})}
            mock = AsyncMock(side_effect=[first, final])
            with patch.object(inference, "connect", connect), patch.object(inference, "complete", mock):
                result = await inference.infer({"context": CONTEXT, "fetched_at": "now"}, root,
                                               prompt, "key", "model", 3, 2048, mcp_url="test", ha_token="ha")
            self.assertEqual(result, 0)
            self.assertEqual(mock.await_count, 2)
            self.assertEqual(mock.call_args.kwargs["tool_choice"], "none")
            self.assertEqual(mock.call_args.args[2][-1]["role"], "tool")
            saved = json.loads((root/"inference.json").read_text())
            self.assertEqual(len(saved["actions"]), 1)
            self.assertEqual(len(saved["requests"]), 2)
