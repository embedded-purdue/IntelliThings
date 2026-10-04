"""Validate model JSON and load saved recommendation fields.

Run: .venv/bin/python ai/llm/response.py /path/to/inference.json
"""

import argparse
import json
import sys
from pathlib import Path

from pydantic import BaseModel, ConfigDict, Field, ValidationError


class ProposedChange(BaseModel):
    model_config = ConfigDict(extra="forbid", strict=True, allow_inf_nan=False)

    area: str
    action: str
    target_value: float | None
    unit: str | None
    reasoning: str
    evidence: list[str]


class Recommendation(BaseModel):
    model_config = ConfigDict(extra="forbid", strict=True)

    summary: str = Field(max_length=255)
    changes: list[ProposedChange]
    missing_information: list[str]


def parse_response(content: str) -> Recommendation:
    """Require a JSON object matching the recommendation contract."""
    return Recommendation.model_validate_json(content)


def load_response(path: str | Path) -> Recommendation:
    """Extract validated fields from a saved inference.json envelope."""
    envelope = json.loads(Path(path).read_text(encoding="utf-8"))
    if not isinstance(envelope, dict) or not isinstance(envelope.get("content"), str):
        raise ValueError("Expected an inference file with a string content field")
    return parse_response(envelope["content"])


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
