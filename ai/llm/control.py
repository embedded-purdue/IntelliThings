"""Discover MCP controls and restrict execution to configured, visible actuators."""

import json
from pathlib import Path

import yaml

from pipeline.context import timestamp, write_json

TURN_ON = "intent__HassTurnOn"
CONFIG = Path(__file__).with_name("actuators.json")


def context_entities(text):
    # HA GetLiveContext currently returns a heading followed by YAML entities.
    start = text.find("\n- names:")
    if start < 0:
        return []
    entities = yaml.safe_load(text[start:])
    return entities if isinstance(entities, list) else []


def visible_actuators(context, configured):
    entities = context_entities(context)
    visible, missing = [], []
    for actuator in configured:
        matches = [e for e in entities if isinstance(e, dict)
                   and e.get("domain") == actuator["domain"]
                   and actuator["name"] in str(e.get("names", "")).split(", ")]
        if len(matches) == 1 and matches[0].get("state") in ("on", "off"):
            visible.append({**actuator, "state": matches[0]["state"]})
        else:
            missing.append(actuator)
    return visible, missing


class Controller:
    def __init__(self, session, output_dir):
        self.session = session
        self.output_dir = output_dir
        self.inventory = {}
        self.tools = []
        self.actions = []

    async def discover(self, snapshot):
        advertised = []
        cursor = None
        while True:
            page = await self.session.list_tools(cursor=cursor)
            advertised.extend(page.tools)
            cursor = page.nextCursor
            if not cursor:
                break
        visible, missing = visible_actuators(snapshot["context"], json.loads(CONFIG.read_text()))
        self.inventory = {"visible_actuators": visible, "missing_actuators": missing,
                          "advertised_tools": [t.name for t in advertised]}
        tool = next((t for t in advertised if t.name == TURN_ON), None)
        if tool and visible:
            schema = tool.inputSchema
            if not {"name", "domain"}.issubset(schema.get("properties", {})):
                raise ValueError("HassTurnOn schema no longer supports name/domain targeting")
            # Narrow the advertised API to named, configured targets. Never area-wide calls.
            self.tools = [{"type": "function", "function": {
                "name": TURN_ON, "description": tool.description,
                "parameters": {"type": "object", "additionalProperties": False,
                    "required": ["name", "domain"], "properties": {
                        "name": {"type": "string", "enum": [a["name"] for a in visible]},
                        "domain": {"type": "array", "minItems": 1, "maxItems": 1,
                                   "items": {"type": "string", "enum": sorted({a["domain"] for a in visible})}},
                    }},
            }}]
        write_json(self.output_dir / "actuators.json", {
            "input_fetched_at": snapshot["fetched_at"], **self.inventory,
            "turn_on_available": bool(self.tools),
            "llm_tools": self.tools,
        })
        self.save_actions()

    def save_actions(self):
        write_json(self.output_dir / "actions.json", {"updated_at": timestamp(), "actions": self.actions})

    async def execute(self, call):
        function = call.get("function", {})
        args = json.loads(function.get("arguments", "{}"))
        if function.get("name") != TURN_ON or not self.tools:
            raise ValueError("Tool is not permitted")
        if not isinstance(args, dict) or set(args) != {"name", "domain"}:
            raise ValueError("A precise actuator name and domain are required")
        matches = [a for a in self.inventory["visible_actuators"]
                   if args["name"] == a["name"] and args["domain"] == [a["domain"]]]
        if len(matches) != 1:
            raise ValueError("Actuator is not configured and visible in MCP context")
        if self.actions:
            raise ValueError("Only one actuator call is allowed per cycle")
        record = {"tool": TURN_ON, "arguments": args, "started_at": timestamp(), "status": "pending"}
        self.actions.append(record)
        self.save_actions()  # Persist intent before performing a side effect.
        try:
            result = await self.session.call_tool(TURN_ON, arguments=args)
            payload = result.model_dump(mode="json", exclude_none=True)
            failed = result.isError
            for block in result.content:
                if block.type == "text":
                    try:
                        value = json.loads(block.text)
                    except ValueError:
                        continue
                    if isinstance(value, dict):
                        data = value.get("data", {})
                        if (value.get("success") is False or value.get("error")
                                or value.get("response_type") == "error"
                                or (isinstance(data, dict) and data.get("failed"))):
                            failed = True
            record.update(status="error" if failed else "returned", result=payload)
            return {"status": record["status"], "mcp_result": payload}
        except BaseException:
            # A transport timeout may occur after HA applied the action. Never retry it.
            record["status"] = "unknown"
            raise
        finally:
            self.save_actions()
