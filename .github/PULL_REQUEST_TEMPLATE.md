Closes #

## What changed

<!-- One or two sentences. What does this PR do? -->

## Why

<!-- What problem does it solve, or which session milestone does it move? -->

## How I tested it

<!-- Be specific. "Flashed to Rig S and confirmed the SHT41 reads 22.3 °C against a
     reference thermometer" or "Replayed the emulator's CO₂-spike scenario and the agent
     turned on the fan" beats "works on my machine". -->

## Subteam

- [ ] Software
- [ ] Hardware
- [ ] AI
- [ ] Docs

## Checklist

- [ ] Targeting `dev` (not `main`)
- [ ] Branch name follows `software/…`, `hardware/…`, `ai/…` or `docs/…`
- [ ] Issue linked above
- [ ] No secrets in the diff (Wi-Fi credentials, HA tokens, OpenRouter / AWS keys, bot tokens)
- [ ] No generated artifacts committed (build output, gerbers, STLs)
- [ ] Firmware: `cargo fmt` and `cargo clippy` clean, tested on Rig S
- [ ] Changes a contract in `docs/interfaces/` (MQTT schema, pin map, `/ha-event`, …)? If so, each affected lead is tagged for review

## Anything you're unsure about

<!-- Flag it here. Much cheaper to ask now than to find out at final assembly. -->
