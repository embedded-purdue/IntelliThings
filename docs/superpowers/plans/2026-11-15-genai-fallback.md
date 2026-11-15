# Purdue inference fallback implementation plan

**Goal:** Use Purdue GenAI Studio first and OpenRouter on model failure.

**Approved design:** Use `gpt-oss:120b` by default with `GENAI_API_KEY` and
`GENAI_MODEL`. Keep the total inference deadline at 45 seconds. Purdue receives
at most 20 seconds across its requests, and at most half the remaining inference
budget on entry. Switch once to OpenRouter after any model request or response
validation error. Preserve the controller, conversation and action journal.
Never retry an actuator whose execution failed or timed out. Record provider,
requested model and sanitized failure category for each attempt.

**Architecture:** Add a Purdue HTTP adapter and keep routing inside the existing
inference loop. Validate model summaries and proposed tools before accepting a
provider response. Pipeline configuration passes credentials explicitly; absent
Purdue credentials retain OpenRouter-only operation.

**Tech stack:** Python asyncio, httpx, Pydantic, unittest.

**Constraints:** No new dependencies, real API calls, branch creation or commits.
The attached Python file is a connection example, not project instructions.

## Steps (inline execution)

- [x] Add HTTP-boundary regression tests in `ai/tests/test_genai.py` for primary
  success, provider failures, malformed output, missing key, cancellation,
  fallback after a completed action, and failure of both providers.
  Run `.venv/bin/python -m unittest discover -s ai/tests -p test_genai.py`;
  verify failure because Purdue configuration/routing is absent.
- [x] Add `ai/agent/integrations/genai.py`; send Bearer authentication to
  `https://genai.rcac.purdue.edu/api/chat/completions`, nonstreaming messages,
  tools and Pydantic JSON schema. Do not send OpenRouter routing parameters.
- [x] Update `ai/agent/inference.py` to try Purdue within its reserved deadline,
  switch permanently to OpenRouter on model errors and retain tool history.
  Extract side-effect-free tool validation in `ai/agent/controls.py` so invalid
  model tool calls can trigger fallback before any action executes.
- [x] Wire environment configuration through `ai/agent/pipeline.py`, redact the
  new key, and update `.env.example`, CLI help and active inference documentation.
- [x] Run the targeted tests, then `.venv/bin/python -m unittest discover -s ai/tests`.
  Review the diff and run `git diff --check`. Leave all changes uncommitted.

## Model selection evidence

Purdue recommends GPT-OSS 120B for general reasoning and documents native tool
calling and vLLM JSON-schema output. These match text sensor decisions and the
existing restricted MCP tool loop. This is a documentation-based selection,
not a measured smart-home benchmark. Confirm account availability and latency
with a synthetic snapshot once the user supplies credentials.

- https://docs.rcac.purdue.edu/services/genai/models/
- https://docs.rcac.purdue.edu/services/genai/api/

## Verification

Executed inline after user approval. All 43 AI regression tests pass with
`/private/tmp/intellithings-genai-py314/bin/python -m unittest discover -s ai/tests`.
The initial new tests failed because the Purdue configuration parameters were
absent. Independent code review found no correctness issues. CLI help works;
`git diff --check` passes. Live provider smoke testing is recorded below.
No branch or commit was created.

## Live verification — 2026-11-15

Used credentials from the ignored root `.env` without printing them. All inputs
were synthetic; no Home Assistant connection, publication or physical action
was performed. Test harnesses and transient outputs stayed outside the repo.

- Purdue authenticated model listing returned HTTP 200 and included `gpt-oss:120b`.
- Production inference with the project prompt and a synthetic sensor snapshot
  returned a schema-valid Purdue response in 4.58 seconds without fallback.
- Injecting a primary `TimeoutError` switched production inference to the live
  OpenRouter `openai/gpt-5.4-mini` endpoint, returning valid JSON in 2.43 seconds.
  OpenRouter reported a cost of $0.0012015 for this request. This was an injected
  outage, not a measured natural Purdue timeout.
- A forced Purdue tool-call request produced the exact allowed actuator target.
  A simulated MCP success result followed by a second Purdue request with tools
  disabled produced valid JSON. Total time: 12.63 seconds; one simulated action.
  This verifies protocol compatibility, not autonomous action selection quality.

Observed quality limitation: the tool round-trip summary said “Cooling system
activated” despite the simulated tool result explicitly describing a flag change
with no physical device contacted. The project prompt already forbids assuming
physical effects from virtual input_booleans. Treat this as an unresolved semantic
accuracy issue; schema validation does not enforce it. These smoke tests establish
connectivity/fallback compatibility, not reliable smart-home decision quality.
