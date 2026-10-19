# Local LLM agent on a Raspberry Pi 5 (16 GB): research report

*IntelliThings · AI subteam · researched 2026-10-17 · status: research, not a decision*

> **Context.** The agent already runs on the HA Pi (`ai/`, Project Guideline §6) and calls a
> cloud model through OpenRouter, with the harness acting as the MCP client. This report
> evaluates running the **model itself locally, CPU-only** on the same Pi. Switching would be a
> team decision. The good news: the harness carries over almost unchanged. It only needs to
> point at a local OpenAI-compatible endpoint (`llama-server`) instead of OpenRouter.

**How to read the numbers.** Every figure is tagged:

- **[measured]**: someone ran it on a real Pi 5 (source given).
- **[independent]**: third-party evaluation, not the model's developer.
- **[dev]**: the developer's own claim (model card or blog).
- **[estimate]**: my calculation. The method is shown in §5.

No Pi 5 performance figures have been invented. Where none exist, the cell says so.

---

## 1. Executive summary

1. **The Pi 5 can run a 2–4B-parameter agent model, but the time is spent reading the prompt,
   not writing the answer.** Measured prompt processing is only ~25–40 tokens/s for 2–4B models,
   and generation is ~4–8.5 tokens/s. MCP tool definitions alone can be thousands of tokens.
   Without prompt caching, one agent step would take minutes; with it, a 1–3-step decision takes
   roughly **20–90 s** [estimate]. That's fine for autonomous, event-driven actions but slow for
   chat.
2. **No small model is reliable at long multi-step workflows. That's measured, not assumed.**
   In the independent AgentFloor study, every model ≤4B **failed to reach 80% reliability on
   branching or multi-source tasks**, and the best got only 49% across the whole ladder. Single
   tool calls and two-step chains are where small models are usable.
3. **So the winning design is architecture D, not a cleverer model:**
   - A **deterministic event processor** decides *when* to call the LLM. This replaces the
     "decision model".
   - The **harness narrows** the dynamically discovered MCP tools to the few relevant to the
     event.
   - The LLM makes **one or two constrained, schema-checked tool calls** plus a short
     explanation.
   - The harness checks the result itself, then logs everything to SQLite.
4. **Models to benchmark first:**
   - **MiniCPM5-2B**: fastest measured on a Pi 5 at this capability level, standard dense
     architecture, strong developer-reported tool scores.
   - **NVIDIA Nemotron 3 Nano 4B**: best independent agentic score at ≤4B.
   - **Qwen3.5-4B**, with Qwen3.5-2B as the faster fallback: strong developer-reported τ²-Bench
     scores, Apache 2.0, broad tooling.

   **Gemma 4 E4B** is a useful accuracy reference but is measured slow on the Pi (~3.7 tokens/s).
5. **Excluded:**
   - **Jev** (closed and hosted, not local).
   - **35B-class MoE models** (Qwen3.5-35B-A3B, MiniCPM-V 4.7 35B-A3B). At 4-bit they don't fit
     in 16 GB next to Home Assistant [estimate].
   - **Vision models** (unnecessary for structured sensor data).
   - **Base FunctionGemma 270M** (scored 0% before fine-tuning [independent]).
6. **Memory is a database problem, not a model problem.** SQLite holds observations, episodes,
   decisions, outcomes and learned per-room patterns. Each prompt gets a fixed-size,
   deterministic slice. "Learning" means updating statistics and preferences, not retraining.

---

## 2. Ranked shortlist

The ranking weighs, in order:
1. Evidence of reliable tool use (independent results above developer claims).
2. Measured or well-founded Pi speed.
3. llama.cpp/ARM maturity.
4. License.

"Q4 size" is the Q4_K_M file size where known, otherwise an estimate (~0.6 bytes per
parameter). "Pi 5 speed" is generation (tg) / prompt processing (pp) in tokens/s.

| # | Model (developer) | Released | Params / architecture | Q4 size | Context | Tool calling | Pi 5 speed tg / pp | License | Main risk |
|---|---|---|---|---|---|---|---|---|---|
| 1 | **MiniCPM5-2B** (OpenBMB) | Sep 7, 2026 | 2.52B, dense (Llama-style, 42 layers) | **1.56 GB** [measured] | 131K | XML tool calls; SGLang `minicpm5` parser. BFCL v4 **66.6**, τ²-Telecom 97.1 [dev] | **8.5 / 38.8** [measured, Pi 5 8 GB, llama.cpp; tg best at 2 threads] | Apache 2.0 | Weeks old; no independent agent evaluation yet; the llama.cpp tool-call parser path needs testing |
| 2 | **NVIDIA Nemotron 3 Nano 4B** (NVIDIA) | Mar 17, 2026 | 3.97B, hybrid Mamba2 + Transformer (21 Mamba / 4 attention / 17 MLP layers) | ~2.5 GB [estimate]; official GGUF | Trained to 49K | RL-trained for multi-turn tool use [dev]. AgentFloor **49.3** overall, best ≤4B [independent] | Not measured. Est. ~4.5–5 tg [estimate] | NVIDIA license (**verify**) | License terms; no Pi data; hybrid prompt-cache behaviour in llama.cpp (§5) |
| 3 | **Qwen3.5-4B** (Alibaba Qwen) | Mar 2, 2026 | ~4B, hybrid Gated DeltaNet + gated attention, dense FFN (32 layers); has a vision encoder | ~2.6 GB [estimate] | 262K | `qwen3_coder` tool parser. BFCL-V4 **50.3**, τ²-Bench **79.9** [dev] | Not measured. Est. ~4.5 tg [estimate] | Apache 2.0 | **Thinking is on by default**, so disable it; ARM speed of the DeltaNet layers unmeasured |
| 4 | **Qwen3.5-2B** (Alibaba Qwen) | Mar 2, 2026 | ~2B, same hybrid design | ~1.3 GB [estimate] | 262K | BFCL-V4 43.6, τ² 48.8 [dev]. AgentFloor **44.0**, and the **best ≤4B at two-step chains (80%)** [independent] | Not measured. Est. ~8–9 tg [estimate] | Apache 2.0 | Weaker at branching (40%) |
| 5 | **Gemma 4 E4B** (Google) | Apr 2, 2026 | 4.5B effective / 8B including per-layer embeddings | 8.95 GB file (incl. embeddings) | 128K | Native function calling; τ² 42.2 [dev]. AgentFloor **46.0**, **96% on single calls** [independent] | **3.7 / 22.5** [measured, Pi 5 16 GB]; peak RAM 5.8 GiB | Apache 2.0 per card (license file named `gemma_4_license`, **verify**) | Slow on the Pi; heavy on RAM |
| 6 | **Ministral 3 3B Instruct 2512** (Mistral) | Dec 2025 | 3B (+ vision) | ~2 GB [estimate] | 256K | Native function calling + JSON [dev]. AgentFloor **42.7**, **88% single calls** (clears the 80% bar) [independent] | Not measured. Est. ~6 tg, similar to Llama 3.2 3B [estimate] | Apache 2.0 | Weak at branching (16%) |
| 7 | **Granite 4.0 H-Tiny** (IBM) | Oct 2025 | **7B total / ~1B active**, hybrid Mamba + MoE | ~4 GB [estimate] | Long | Built for tool calling and structured output [dev] | Not measured. Est. ~10–15 tg if MoE routing is efficient on CPU [estimate, low confidence] | Apache 2.0 | The only MoE that fits comfortably; unproven on ARM |
| 8 | **Granite 4.0 H-Micro / Micro 3B** (IBM) | Oct 2025 | 3B, hybrid (H) or plain transformer | ~1.9 GB [estimate] | Long | AgentFloor "granite4:3b" **40.0**, 80% single calls [independent] | Not measured. Est. ~6 tg [estimate] | Apache 2.0 | Mid-pack at chains (56%) |
| 9 | **LFM2.5-1.2B-Instruct** (Liquid AI) | ~early 2026 | 1.17B, hybrid (10 gated-conv + 6 attention layers) | ~0.75 GB [estimate] | 32K | Pythonic or JSON tool calls; BFCLv3 **49.1** [dev] (replaces the deprecated LFM2-1.2B-Tool) | Not measured. Est. ~15 tg [estimate] | LFM Open License (**check commercial terms**) | Fast candidate for routing/extraction, not deep reasoning |
| 10 | **Gemma 4 E2B** (Google) | Apr 2, 2026 | 2.3B effective / 5.1B incl. embeddings | 3.1 GB | 128K | Native function calling; τ² **24.5** [dev] | **7.0–7.6 / 29.7–45** [measured, two sources] | Apache 2.0 (verify) | Fast, but weak at tools |
| 11 | **Granite 4.0 H 1B / H 350M** (IBM) | Oct 28, 2025 | ~1.5B / 350M, hybrid | ~0.9 GB / ~0.25 GB [estimate] | Long | Leads its size class on BFCLv3 [dev, numbers only in a chart image] | Not measured | Apache 2.0 | Router or extractor only |
| 12 | **Qwen3.5-0.8B** (Alibaba Qwen) | Mar 2, 2026 | 0.8B, hybrid | ~0.5 GB [estimate] | 262K | BFCL-V4 25.3, τ² 11.6 [dev] | Not measured | Apache 2.0 | Classification or routing only |

**Considered and not shortlisted**

| Model | Why |
|---|---|
| **Qwen3 0.6B / 1.7B / 4B, Llama 3.2 3B, Gemma 3 4B** (your list) | Superseded by the rows above. AgentFloor: qwen3:1.7b 33.6, qwen3:0.6b 29.2 [independent]. One home test of 24 commands: Qwen3-4B-2507 83.3%, Llama 3.2 3B 70.8% [independent, small sample] |
| **MiniCPM5-1B** (OpenBMB, May 19, 2026) | Plausible router-tier model (dense, 131K context). License and tool-calling details not verified |
| **FunctionGemma 270M** (Google) | Meant to be fine-tuned; the base model scored **0.0** on AgentFloor [independent]. The community **Home-FunctionGemma 270M** (trained on 240k Home Assistant requests) suits a *fixed* intent layer, not dynamic MCP |
| **vLLM Semantic Router Decision 3.0, `d3-edge`** (0.59B, based on Qwen3.5-0.8B, Apache 2.0) | Returns probabilities for multiple-choice / yes-no / scale questions and **cannot generate tool calls**. Runs on Transformers/PyTorch or the vllm-sr engine (`--platform cpu`); no GGUF; **ARM64 CPU latency unknown**. Only useful for a fuzzy "is this worth waking the LLM" check, if rules turn out not to be enough |
| **Jev (TypeSafe, "System One")** | **Closed, hosted, waitlisted**; not a local model. Several unofficial "Jev" domains exist; use only TypeSafe's own docs |
| **Qwen3.5-35B-A3B, MiniCPM-V 4.7 35B-A3B** | ~35B total parameters is roughly 20 GB at 4-bit [estimate], which doesn't fit beside HA in 16 GB. The MiniCPM-V 4.7 release also had no model card or license when first posted |
| **Gemma 4 26B-A4B** (25.2B total / 3.8B active) | Matched GPT-5 on AgentFloor overall [independent], but ~15 GB at 4-bit [estimate]. At ~3-bit (~11 GB) it *might* fit; experimental only, if every 4B-class model fails |
| **MiniCPM-V 4.7 1B, LFM2.5-VL, other vision models** | Vision isn't needed for structured sensor data |
| **xLAM-2 1B/3B** (Salesforce) | Strong at function calling but **CC-BY-NC-4.0** (non-commercial) |

---

## 3. Top three recommendations

These are the models to **benchmark first**, not a final pick. Section 8 is the deciding test.

1. **MiniCPM5-2B: the efficiency pick.**
   - **Fastest measured** at this tier: 8.5 tokens/s generation and 38.8 tokens/s prompt
     processing on a Pi 5.
   - Peak RAM of only **2.9 GB** [measured].
   - A plain dense Llama-style architecture, so llama.cpp's ARM kernels and prompt-cache reuse
     work as well as they can.
   - Apache 2.0, with the highest developer-reported tool scores in its size class.
   - **Risk:** brand new and only vendor-benchmarked. Its tool calls are XML and need a parser:
     test whether `llama-server --jinja` produces OpenAI-style `tool_calls`, otherwise parse them
     in the harness.
2. **NVIDIA Nemotron 3 Nano 4B: the reliability pick.**
   - **Best independent agentic result at ≤4B** (AgentFloor 49.3).
   - Specifically RL-trained for multi-turn tool use.
   - Its hybrid Mamba layers keep the KV cache tiny; NVIDIA publishes an official GGUF.
   - **Risks:** license terms need checking; no Pi measurements; prompt-cache reuse for hybrid
     models (§5).
3. **Qwen3.5-4B, falling back to Qwen3.5-2B: the ecosystem pick.**
   - Apache 2.0, strong developer-reported τ²-Bench (79.9), 262K context, and the widest tooling
     support (Ollama, llama.cpp, Unsloth GGUFs).
   - The 2B version had the best independent two-step-chain score at ≤4B.
   - **Run in non-thinking mode.** Thinking is on by default for the 4B, and small models in
     thinking mode burn tokens and can loop.

**Accuracy reference:** run **Gemma 4 E4B** in the same benchmark to see how much accuracy the
faster models give up. It's too slow (~3.7 tokens/s) to be the everyday model.

---

## 4. Agent architecture

| | A: one small LLM | B: decision model + LLM | C: tool model + reasoning model | **D: deterministic core + one small LLM (recommended)** |
|---|---|---|---|---|
| Wakes the LLM | On every event | A ~0.6B classifier decides | On every event | **Rules, hysteresis, cooldowns** |
| Tool selection | The LLM, from the full tool list | The LLM | Specialist tool model | The LLM, from a **pre-narrowed** list |
| Reliability | Lowest (long prompts, long tool lists, multi-step) | Better gating, same execution weakness | Two models' errors compound; hand-offs are fragile | Highest: the LLM only does what small models do well |
| Pi cost | One model, but huge prompts | Two resident models | Two models plus a hand-off prompt each step | One model, small prompts |
| Testability | Poor | Classifier is a black box | Poor | Rules are unit-testable; the LLM is evaluated on a narrow task |

**Design D in detail:**

1. **Event processor (Python, no model).**
   - Subscribes to HA (`state_changed` via the WebSocket API, or MQTT).
   - Computes features: 10-minute trends, an occupancy state machine (LD2410C state + VL53L0X
     distance), and comfort deltas from targets.
   - Fires an *event* only when a threshold is crossed with **hysteresis**, a room-level
     **cooldown** has passed, and the same event hasn't just been handled.
   - Also runs a periodic re-evaluation, e.g. every 30–60 min per occupied room.

   This replaces the decision model: it's free, explainable and deterministic. Add `d3-edge` or
   Qwen3.5-0.8B later only if logs show rules miss things that matter.
2. **Context builder.** A compact, fixed-size prompt:
   - The room's current readings, labelled **[observed]**.
   - Inferences with confidence, labelled **[inferred]** (e.g. "likely cooking, 0.7").
   - Current device states.
   - Up to N relevant memories (§6).
3. **Tool scoping.**
   - MCP discovery still runs (`tools/list` from HA's MCP server) and is cached.
   - The harness then passes only the tools relevant to the event's room and domain, roughly 3–8.
   - Small models get worse with long tool lists; HA itself recommends exposing **fewer than 25
     entities** to small local models.
   - The tool list stays dynamic; only the slice each prompt sees is narrowed.
4. **Constrained generation.**
   - Ask for one JSON object: an action list (each a tool + arguments), a `no_action` reason, and
     an `explanation`.
   - Enforce it with llama.cpp's JSON-schema / grammar sampling, which makes the output
     *syntactically* valid every time; validate the arguments against the tool's schema.
   - **Budget:** at most 2 tool rounds per event.
5. **Execution and checking (the harness, not the LLM).**
   - Safety allowlist and value ranges, plus rate limits.
   - Skip calls that are already satisfied (the fan is already on).
   - After each call, read the state back to confirm it worked; retry once.
   - Record the outcome.
6. **Explanation.** Taken from the `explanation` field, truncated to the display's AI-message
   limit, and published to `intellithings/<node_id>/display`.
7. **Optional escalation.** When the local model declines, fails validation twice, or the event
   is complex, hand it to the existing cloud model path (OpenRouter). This keeps the system
   local-first without making local the ceiling, and stays compatible with the current design.

AgentFloor's measured failure point (branching and multi-source work, at ≤4B) is exactly what D
moves into deterministic code.

---

## 5. Real-world feasibility on the Pi 5

### Speed limits

- **Generation is limited by memory bandwidth.** The Pi 5's LPDDR4X has ~17 GB/s theoretical
  bandwidth. The measured generation speeds imply ~11–13 GB/s effective. [estimate, from:
  1.56 GB × 8.5 t/s ≈ 13 GB/s; 1.88 GB × 5.7 t/s ≈ 11 GB/s; 4.36 GB × 2.7 t/s ≈ 12 GB/s]
  - **Rule of thumb:** generation ≈ 12 GB/s ÷ the bytes read per token, which is about the Q4
    file size for dense models and less for Gemma's per-layer embeddings or MoE active weights.
  - **Generation peaks at 2 threads** [measured: 8.5 at 2 threads vs. 7.5 at 4]. Using 2–3
    threads for generation costs nothing and leaves cores free for HA.
- **Prompt processing is limited by compute,** and it dominates agent latency [measured]:

  | Model | Prompt processing (pp512) |
  |---|---|
  | MiniCPM5-2B | 38.8 t/s |
  | Gemma 4 E2B | 29.7–45 t/s |
  | Llama 3.2 3B | ~27 t/s |
  | Gemma 4 E4B | 22.5 t/s |
  | 7–8B models | 11 t/s |

  A 3,000-token system prompt plus tool definitions would take **~80–140 s for a 2–4B model**
  [estimate].

### What makes it workable

**Prompt-prefix caching.** Keep the system prompt and tool definitions identical and first in
every request, so `llama-server` reuses their cached state and only the new event tokens are
processed. Prompt caching is reliable for pure-transformer models. For **hybrid/recurrent models**
(Nemotron's Mamba layers, Qwen3.5's DeltaNet, LFM's conv layers, Granite-H), reusing a partial
prefix is harder in llama.cpp. **Measure it**: it can decide between a dense and a hybrid model on
this hardware.

### Latency per event [estimate]

Assumes a cached prefix and ~400 new prompt tokens per round.

| Model class | One round (prompt + ~60-token tool call) | Typical event (2 rounds + explanation) |
|---|---|---|
| 2B dense, MiniCPM5-2B-like (39 pp / 8.5 tg) | ~10 s + ~7 s ≈ **17 s** | **~40 s** |
| 4B (~25 pp / ~5 tg) | ~16 s + ~12 s ≈ **28 s** | **~60–70 s** |

Acceptable for "the room got stuffy, turn on the fan". Slow for chat. If chat matters, keep chat
on the cloud path or accept roughly half-minute replies.

### Other limits

- **Context and KV cache.** Keep every event **stateless**: a fresh request with memory pulled from
  the database, never one long conversation. The KV cache stays small and nothing grows over 24/7
  operation. 8K context is plenty. Hybrid models have smaller KV caches but that isn't the
  bottleneck here.
- **RAM.** 16 GB is comfortable. Rough budget [estimate]:

  | Item | RAM |
  |---|---|
  | Home Assistant | 1–2 GB |
  | Desktop GUI | ~0.5–1 GB (recommend headless Raspberry Pi OS Lite) |
  | 4B Q4 model + 8K context | ~3–4 GB |

  That's enough to keep **two models resident**, e.g. a 1–2B fast model and a 4B model, so
  nothing is reloaded. Measured peaks: MiniCPM5-2B 2.9 GiB, Gemma 4 E4B 5.8 GiB.
- **Model loading and storage.** Measured cold loads on a Pi 5 took **17–135 s**. Use an **NVMe
  SSD**, keep models resident (`llama-server` as a systemd service; for Ollama, set
  `keep_alive: -1` because it unloads idle models by default), and never load per event.
- **Sharing the CPU with HA.**
  - Run one inference at a time from a queue.
  - Limit the LLM service with `CPUQuota`/`nice` (e.g. 3 cores); prompt processing would
    otherwise take all four.
  - HA stays responsive because the LLM is **off the critical path**: lights, safety
    automations and dashboards never wait for it.
  - Measure HA API latency during inference (§8).
- **Heat.** The Pi 5 throttles at 80 °C (soft) and 85 °C (hard). Use the **official Active
  Cooler** or better. One 16 GB Pi 5 benchmark peaked at 51–65 °C with no throttling
  (`get_throttled = 0x0`), but its cooling wasn't documented. Log temperature and
  `vcgencmd get_throttled` during the soak test.
- **24/7 operation.**
  - systemd restart-on-failure and a health probe.
  - Daily log rotation.
  - SQLite in WAL mode on the SSD, not the SD card.
  - A watchdog that falls back to "rules only" if the LLM service is down. The house must work
    without the LLM.

### Deployment and runtimes

- **llama.cpp (`llama-server`): recommended.**
  - Build natively on the Pi with dotprod enabled (Cortex-A76 has dotprod, no i8mm/SVE).
  - Provides an OpenAI-compatible endpoint, `--jinja` chat templates for tool calls, JSON-schema
    constrained output and prompt caching.
  - Supports the hybrid architectures (Granite-H, LFM2, Qwen3.5, Nemotron-H).
- **Ollama.** Easiest to start with, and it wraps llama.cpp. Measured speeds were similar but
  with less control over caching and threads. Fine for quick model comparisons.
- **Home Assistant's llama.cpp integration** (new in **HA 2026.8**). It makes a local
  OpenAI-compatible server an HA conversation agent with access to the **Assist API**: HA's own
  agent loop, no custom harness needed. That's a quick baseline to compare against, but it gives
  less control over gating, memory and checking than design D.
- **HA's MCP server.**
  - It exposes **Assist API tools only, for entities exposed to Assist**.
  - Its docs describe **OAuth 2.0 + PKCE** at `/api/mcp`, admin-only by default. In practice the
    project's harness connects with a **long-lived access token**, which works.
  - It offers **no history access**, so the memory database (§6) has to come from the harness's
    own logging or HA's recorder/history API.
- **Can't run on the Pi CPU:** vLLM and SGLang are GPU-oriented, so `d3-edge` only runs through
  Transformers/PyTorch (ARM speed unknown). MLX is Apple-only.
- **OS choice.** The Pi runs Raspberry Pi OS + Docker with HA Container (Project Guideline §5).
  Run `llama-server` as its own systemd service or container. Prefer the Lite (headless) edition
  of Raspberry Pi OS, which saves RAM and CPU.

---

## 6. Memory architecture

Everything lives in **SQLite (WAL mode) on the SSD**. Nothing changes the model's weights.

| Table | Content | Retention |
|---|---|---|
| `observations` | Readings per room, downsampled (1-min means, plus every presence change) | Raw 7 days → hourly aggregates 90 days → daily kept |
| `episodes` | Detected periods with start/end: occupancy sessions, "cooking-like" PM/VOC/heat events, CO₂ build-ups, sleep periods. Each has a `confidence` and `source` (rule name / model) | 1 year |
| `decisions` | Event, prompt hash, tools offered, model output, tool calls + arguments, validation result, execution result, check result, latency, model name | 1 year |
| `outcomes` | What happened next: did the reading improve, **did the user undo it within N minutes** | 1 year |
| `patterns` | Learned statistics per room and hour of the week: occupancy probability, typical temperature/CO₂, typical sleep window | Continuously updated (exponential moving averages) |
| `preferences` | Explicit ("I like 22 °C") and learned ("user turns the fan off below 24 °C"), each with evidence count and confidence | Decay unused entries |
| `summaries` | Daily ≤100-word summary per room, written by the LLM overnight or from templates | Keep |

- **Inferring activities.** Use rules first: the occupancy state machine, plus "PM2.5 + VOC +
  heat rising in the kitchen with presence" means likely cooking. Write episodes with confidence.
  The LLM may label ambiguous episodes overnight, off the hot path.
- **Retrieval.** Deterministic key lookups (room, hour of week, event type, recent decisions on
  the same device), not vector search. That's cheap, exact and debuggable. Each prompt gets a
  **fixed memory budget** (e.g. ≤300 tokens):
  - Latest summary.
  - Top 3 matching patterns.
  - Last 3 decisions in this room and their outcomes.
  - Relevant preferences.

  Add a small embedding model only if free-text notes appear.
- **Learning loop.** User undo → negative outcome → lower that preference's or pattern's
  confidence. Repeated manual settings → a new learned preference once evidence ≥ k. This is
  **database-backed learning** and takes effect immediately.
- **Real weight updates** would mean fine-tuning (e.g. a LoRA) on the `decisions` / `outcomes`
  logs, **done offline on a PC with a GPU**, then copying a new GGUF to the Pi. That's possible
  later, but not needed, and not something to do on the Pi.
- **Bounded growth.** Retention tiers, fixed prompt slots, stateless prompts, and summaries that
  replace raw history. The context never grows with uptime.

---

## 7. Deployment notes

- **Start with:**
  - `llama-server` (one systemd unit per resident model, `--jinja`, prompt cache on,
    `-t 3`, `-c 8192`, keep loaded).
  - The existing Python harness in `ai/` (the same MCP-client loop), pointed at
    `http://127.0.0.1:8080/v1`.
  - SQLite.
- **Sampling.** Use model-card recommendations in **non-thinking** mode. MiniCPM5 recommends
  `min_p=0.0` in llama.cpp (the 0.05 default can cause repetition); Qwen3.5 lists
  `temperature 0.7, top_p 0.8, top_k 20, presence_penalty 1.5` for non-thinking use. For tool
  calls, lower the temperature further and rely on schema constraints.
- **Obstacles to check early:**
  - Tool-call parsing for each model's format in llama.cpp (XML for MiniCPM5, Pythonic for LFM,
    `qwen3_coder`-style for Qwen3.5).
  - Prompt-cache reuse for the hybrid models.
  - The license terms for Nemotron (NVIDIA) and LFM (Liquid).
  - The text-only GGUF variant for vision-equipped models (Qwen3.5, Ministral 3), so no
    vision-encoder memory is wasted.

---

## 8. Benchmark plan on our Pi

**Setup:**
- Pi 5 16 GB, Active Cooler, NVMe.
- Home Assistant running with real integrations.
- **Our emulator ESP32** as the stimulus. Its scripted scenarios and MQTT command topic
  (`ai/emulator/README.md`) make every test repeatable.

**Phase 1: raw speed** (per model, ~1 hour).
- `llama-bench` prompt processing (pp512) and generation (tg128) at 2, 3 and 4 threads, idle and
  with HA under load.
- Peak RAM.
- Cold load time from NVMe.
- Prompt-cache hit timing: the same 3K-token prefix plus a 400-token suffix.

**Phase 2: agent accuracy** (per model, ~200 cases).
- **Scenarios:**
  - The five standard ones: hot + presence, rising CO₂, PM2.5 spike, empty room, nothing wrong.
  - Each **varied**: other rooms; device already on; device unavailable; conflicting sensors
    (presence on, distance far); a sensor missing a field; two simultaneous issues; a user
    preference in memory that contradicts the default.
- **Metrics:**
  - Valid tool calls (%).
  - Correct tool + arguments (%).
  - **Unneeded actions (%)** on "nothing wrong" and "already satisfied" cases.
  - Redundant or repeated calls.
  - Rounds used.
  - Explanation fits the display limit.
  - Hallucinated tools or entities.

**Phase 3: end-to-end and 24/7.**
- Replay one event every 5–10 minutes for **24 h** with HA running.
- **Metrics:**
  - Event → action → display latency (p50/p95).
  - HA API response time during inference vs. idle.
  - Temperature and `get_throttled`.
  - Memory growth.
  - Service restarts.
  - Database size.

**Suggested acceptance bar** (for the team to agree on):

| Metric | Target |
|---|---|
| Valid calls | ≥ 98% |
| Correct action | ≥ 90% |
| Unneeded actions | ≤ 5% |
| p95 event latency | ≤ 90 s |
| HA p95 API latency increase | ≤ 2× |
| Throttling events | None |

Pick the **fastest model that passes**, not the most capable one.

---

## 9. Sources

**Pi 5 measurements**
- MiniCPM5-2B vs. Gemma 4 E2B, llama-bench on a Pi 5: https://note.com/sakuya_mikagami/n/na61e0c077d17
- Pi 5 16 GB arena (Ollama + llama.cpp, Gemma 4, Llama 3.2, 7–8B): https://github.com/leemailto2008/pi5-llm-arena
- Jeff Geerling's AI benchmarks (Llama 3.2 3B on Pi 5 8/16 GB): https://github.com/geerlingguy/ai-benchmarks
- LLM inference on single-board computers (Ollama ranges): https://arxiv.org/abs/2511.07425

**Independent agentic evaluation**
- AgentFloor (small open-weight models on a 6-tier tool-use ladder, May 2026): https://arxiv.org/abs/2605.00334
- XDA test of 7 local models on Home Assistant: https://www.xda-developers.com/tested-local-llms-nas-smart-home-automation-one-didnt-fall-apart/
- Home Assistant + llama.cpp 24-command accuracy test: https://techfuelhq.com/tutorials/home-assistant-local-llm-voice-2026/

**Model cards and announcements**
- MiniCPM5-2B: https://huggingface.co/openbmb/MiniCPM5-2B · MiniCPM5 collection: https://huggingface.co/collections/openbmb/minicpm5 · GitHub: https://github.com/openbmb/minicpm
- Nemotron 3 Nano 4B: https://huggingface.co/blog/nvidia/nemotron-3-nano-4b · GGUF: https://huggingface.co/nvidia/NVIDIA-Nemotron-3-Nano-4B-GGUF
- Qwen3.5-4B: https://huggingface.co/Qwen/Qwen3.5-4B · Qwen3.5-0.8B: https://huggingface.co/Qwen/Qwen3.5-0.8B
- Gemma 4: https://huggingface.co/google/gemma-4-E2B · function calling: https://ai.google.dev/gemma/docs/capabilities/text/function-calling-gemma4
- Ministral 3 3B: https://huggingface.co/mistralai/Ministral-3-3B-Instruct-2512-BF16
- Granite 4.0 family: https://www.ibm.com/granite/docs/models/granite · Granite 4.0 Nano: https://huggingface.co/blog/ibm-granite/granite-4-nano
- LFM2.5-1.2B-Instruct: https://huggingface.co/LiquidAI/LFM2.5-1.2B-Instruct · LFM2-1.2B-Tool (deprecated): https://docs.liquid.ai/lfm/models/lfm2-1.2b-tool
- FunctionGemma 270M: https://huggingface.co/google/functiongemma-270m-it
- vLLM Semantic Router d3-edge: https://huggingface.co/vllm-sr/d3-edge
- Jev (TypeSafe), via OpenRouter docs: https://openrouter.ai/docs/guides/community/jev
- MiniCPM-V 4.7 release reports: https://aiweekly.co/alerts/openbmb-releases-minicpm-v-47-series-first-moe-model
- xLAM-2-3b-fc-r (license): https://featherless.ai/models/Salesforce/xLAM-2-3b-fc-r

**Home Assistant**
- MCP Server integration: https://www.home-assistant.io/integrations/mcp_server/
- llama.cpp integration (HA 2026.8): https://www.home-assistant.io/integrations/llama_cpp

**Not used as evidence:** sites whose Pi 5 figures are labelled as estimates (e.g. a
note.com "top 15" ranking), aggregator pages that conflicted with official model cards, and the
BFCL aggregator snapshots. Check the official leaderboard at https://gorilla.cs.berkeley.edu/leaderboard.html
before quoting BFCL numbers.
