"""Shared context decoding and a single MCP handshake for clients and probes."""

from contextlib import asynccontextmanager
import json
from pathlib import Path
import sys
from types import SimpleNamespace
import unittest
from unittest.mock import AsyncMock, Mock, patch

from mcp.types import CallToolResult, TextContent

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from agent.integrations import home_assistant


class HomeAssistantTests(unittest.IsolatedAsyncioTestCase):
    def test_parse_text_json_and_error_without_losing_inspection_output(self):
        result = CallToolResult(content=[
            TextContent(type='text', text='Kitchen: 22 C'),
            TextContent(type='text', text='{"success":true,"result":{"bedroom":23}}'),
            TextContent(type='text', text=''),
        ])
        texts, error = home_assistant.parse_live_context(result)
        self.assertIsNone(error)
        self.assertEqual(texts, ['Kitchen: 22 C', '{"bedroom": 23}', ''])
        pretty, _ = home_assistant.parse_live_context(result, indent=2)
        self.assertEqual(pretty[1], json.dumps({'bedroom': 23}, indent=2))

        result.content.append(TextContent(type='text', text='{"success":false,"result":"Unavailable"}'))
        texts, error = home_assistant.parse_live_context(result)
        self.assertEqual(error, 'GetLiveContext reported success=false.')
        self.assertEqual(texts[-1], 'Unavailable')
        result.isError = True
        self.assertEqual(home_assistant.parse_live_context(result)[1],
                         'GetLiveContext returned an MCP tool error.')

    async def test_connect_initializes_once_before_notifying_and_yielding(self):
        client = AsyncMock()
        client.__aenter__.return_value = client
        session = AsyncMock()
        session.__aenter__.return_value = session
        initialized = SimpleNamespace(serverInfo='test server')
        session.initialize.return_value = initialized
        observer = Mock()

        @asynccontextmanager
        async def transport(*args, **kwargs):
            yield 'read', 'write', None

        with patch.object(home_assistant.httpx, 'AsyncClient', return_value=client), \
             patch.object(home_assistant, 'streamable_http_client', transport), \
             patch.object(home_assistant, 'ClientSession', return_value=session):
            async with home_assistant.connect('http://ha', 'secret', 2, on_initialize=observer) as connected:
                self.assertIs(connected, session)
                session.initialize.assert_awaited_once()
                observer.assert_called_once_with(initialized)
        session.__aexit__.assert_awaited_once()
        client.__aexit__.assert_awaited_once()
