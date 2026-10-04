"""Build prompts and persist inference separately from collection."""

import hashlib
import json
import sys

from integrations.openrouter import complete
from llm.response import Recommendation, parse_response
from pipeline.context import timestamp, write_json


async def infer(snapshot, output_dir, prompt_file, api_key, model, timeout, max_tokens):
    status = {
        "attempted_at": timestamp(),
        "input_fetched_at": snapshot["fetched_at"],
        "requested_model": model,
    }
    try:
        prompt = prompt_file.read_text(encoding="utf-8").strip()
        if not prompt:
            raise ValueError("Prompt file is empty")
        if not snapshot.get("context", "").strip():
            raise ValueError("Collected context is empty")
        result = await complete(api_key, model, [
            {"role": "system", "content": prompt},
            {"role": "user", "content": json.dumps({
                "fetched_at": snapshot["fetched_at"],
                "context": snapshot["context"],
            }, ensure_ascii=False)},
        ], timeout, max_tokens, response_schema=Recommendation.model_json_schema())
        recommendation = parse_response(result["content"])
        write_json(output_dir / "inference.json", {
            "schema_version": 2,
            "generated_at": timestamp(),
            "input_fetched_at": snapshot["fetched_at"],
            "requested_model": model,
            "prompt_sha256": hashlib.sha256(prompt.encode("utf-8")).hexdigest(),
            **result,
            "recommendation": recommendation.model_dump(),
        })
    except Exception as exc:
        error = f"{type(exc).__name__}: {exc}"
        if api_key:
            error = error.replace(api_key, "[REDACTED]")
        write_json(output_dir / "inference_status.json", {**status, "ok": False, "error": error})
        print(f"Inference failed: {error}", file=sys.stderr)
        return 1
    write_json(output_dir / "inference_status.json", {**status, "ok": True})
    print(f"Saved model response to {output_dir / 'inference.json'}")
    return 0
