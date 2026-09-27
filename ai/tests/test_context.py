"""Regression checks for snapshot persistence without a running HA server."""

import json
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import AsyncMock, patch

from mcp.types import CallToolResult, TextContent

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from pipeline import context


class CollectionTests(unittest.IsolatedAsyncioTestCase):
    async def test_success_publishes_private_readable_snapshot(self):
        result = CallToolResult(content=[TextContent(
            type="text", text=json.dumps({"success": True, "result": "Kitchen: 22 C"})
        )])
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory)
            with patch.object(context, "get_live_context", AsyncMock(return_value=result)):
                self.assertEqual(await context.collect("http://test", "token", 1, output), 0)
            snapshot = json.loads((output / "latest.json").read_text())
            self.assertEqual(snapshot["context"], "Kitchen: 22 C")
            self.assertTrue(snapshot["fetched_at"])
            self.assertEqual((output / "latest.json").stat().st_mode & 0o777, 0o600)
            self.assertTrue(json.loads((output / "status.json").read_text())["ok"])

    async def test_failures_preserve_previous_snapshot_and_redact_token(self):
        failures = [
            AsyncMock(side_effect=RuntimeError("unreachable secret-token")),
            AsyncMock(side_effect=TimeoutError()),
            AsyncMock(return_value=CallToolResult(content=[], isError=True)),
            AsyncMock(return_value=CallToolResult(content=[TextContent(
                type="text", text='{"success": false}'
            )])),
        ]
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory)
            context.write_json(output / "latest.json", {"context": "previous"})
            previous = (output / "latest.json").read_bytes()
            for failure in failures:
                with patch.object(context, "get_live_context", failure):
                    self.assertEqual(await context.collect("http://test", "secret-token", 1, output), 1)
                self.assertEqual((output / "latest.json").read_bytes(), previous)
                status = (output / "status.json").read_text()
                self.assertFalse(json.loads(status)["ok"])
                self.assertNotIn("secret-token", status)


if __name__ == "__main__":
    unittest.main()
