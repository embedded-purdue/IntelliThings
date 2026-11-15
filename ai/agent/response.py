"""Validate model JSON and load saved recommendation fields.

Run: .venv/bin/python ai/agent/response.py /path/to/inference.json
"""

import argparse
import json
import sys
from pathlib import Path
from typing import Annotated
from copy import deepcopy

from pydantic import BaseModel, ConfigDict, Field, StringConstraints, ValidationError


ShortSummary = Annotated[str, StringConstraints(strip_whitespace=True, min_length=1, max_length=60)]


RoomKey = Annotated[str, StringConstraints(pattern=r"^[a-z][a-z0-9_]*$")]


class Recommendation(BaseModel):
    model_config = ConfigDict(extra="forbid", strict=True)

    summary: str = Field(min_length=1, max_length=255)
    room_summaries: dict[RoomKey, ShortSummary]


def response_schema(room_keys):
    """Require exactly this cycle's discovered rooms in structured output."""
    schema = deepcopy(Recommendation.model_json_schema())
    value_schema = schema["properties"]["room_summaries"]["patternProperties"].values()
    value_schema = next(iter(value_schema))
    schema["properties"]["room_summaries"] = {
        "type": "object", "additionalProperties": False,
        "properties": {key: deepcopy(value_schema) for key in room_keys},
        "required": list(room_keys),
    }
    return schema


def parse_response(content: str, expected_rooms=None) -> Recommendation:
    """Require a JSON object matching the recommendation contract."""
    result = Recommendation.model_validate_json(content)
    if expected_rooms is not None and set(result.room_summaries) != set(expected_rooms):
        missing = sorted(set(expected_rooms) - set(result.room_summaries))
        extra = sorted(set(result.room_summaries) - set(expected_rooms))
        raise ValueError(f"Room coverage mismatch: missing={missing}, extra={extra}")
    return result


def load_response(path: str | Path) -> Recommendation:
    """Extract validated fields from a saved inference.json envelope."""
    envelope = json.loads(Path(path).read_text(encoding="utf-8"))
    if not isinstance(envelope, dict) or not isinstance(envelope.get("content"), str):
        raise ValueError("Expected an inference file with a string content field")
    return parse_response(envelope["content"], envelope.get("room_inventory"))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("path", type=Path, help="Saved inference.json")
    args = parser.parse_args()
    try:
        recommendation = load_response(args.path)
    except (OSError, ValueError, ValidationError) as exc:
        print(f"Cannot load recommendations: {exc}", file=sys.stderr)
        return 1
    print(recommendation.model_dump_json(indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main())
