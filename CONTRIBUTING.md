# Contributing to IntelliThings

How we branch, commit and review. How the subteams split work and hand off to each other
is in [`docs/Collaboration_Guidelines.md`](docs/Collaboration_Guidelines.md).

## Workflow

**Issue → Branch → Change → Test → Pull Request → Review → Merge**

Before starting, check Issues so nobody else is on the same task and assign yourself.
Tasks are assigned by subteam leads through GitHub Issues.

## Branching model

Two-tier model. Keep `main` demo-ready.

```
main      ●────────────────●────────────────●      protected, always demo-ready
           ╲              ╱                ╱
dev         ●──●──●──●──●──●──●──●──●──●──●        integration branch
             ╲    ╱   ╲       ╱
feature       ●──●     ●─────●                     your work
```

| Branch | Purpose | Who pushes |
|---|---|---|
| `main` | Stable, working, demo-ready. Tagged at milestones. | Nobody directly — PR from `dev` only |
| `dev` | Integration. Everything merges here first. | Via PR from feature branches |
| `<team>/<desc>` | Your actual work | You |

### Naming feature branches

Prefix with your team: `software/`, `hardware/`, `ai/` or `docs/`.

```
software/baseline
software/sht41-driver
software/dwin-dev-gui
hardware/power-input
hardware/mcu-bare-chip
ai/ha-mqtt-entities
ai/lambda-agent
docs/mqtt-schema
```

Lowercase, hyphens, no spaces. Keep it short and descriptive.

## Day-to-day

```bash
# 1. Start from an up-to-date dev
git checkout dev
git pull origin dev

# 2. Cut your branch
git checkout -b software/sht41-driver

# 3. Work, committing as you go
git add <files>
git commit -m "software: add SHT41 temperature/humidity driver"

# 4. Push and open a PR into dev
git push -u origin software/sht41-driver
```

If `dev` has moved since you branched, merge it into your branch and resolve conflicts
there, not in the PR:

```bash
git checkout dev && git pull origin dev
git checkout software/sht41-driver
git merge dev
```

## Commit messages

Format: `<area>: <what changed>`, where the area is `software`, `hardware`, `ai` or `docs`.

```
software: add SHT41 temperature/humidity driver
hardware: fix I2C pull-up values on sensor sheet
ai: add MQTT entities for emulator nodes
docs: fix MQTT payload field names
```

Present tense, lowercase after the colon, no trailing period. Small commits. Explain *why*
in the body if it isn't obvious from the diff.

## Pull requests

1. **Target `dev`**, not `main`.
2. Link the issue ("Closes #12") and fill out the PR template — what changed, how you
   tested it, anything you're unsure about.
3. **At least one review** before merge. CODEOWNERS requests the subteam lead
   automatically; the two PMs, @spicybutter and @rakkicow, are code owners on every
   path and can approve any PR. Who else can review is in the Collaboration Guidelines (e.g. sensor
   changes can be reviewed by a teammate who has worked on sensors).
4. **Contract changes** (anything in `docs/interfaces/`) need a review from **each affected
   team's lead**.
5. Keep PRs small. Draft PRs are welcome — open one early to get eyes on a direction.

### Merging to `main`

Only at milestones, only from `dev`, once the pipeline works end to end. Subteam leads
coordinate this — don't open a PR into `main` on your own.

## Repository layout

```
hardware/         KiCad project(s) + project-local libraries, enclosure CAD, datasheets,
                  Hardware_BOM_Candidates.md, Full_Parts_List.md
software/         Firmware_Architecture.md
└── firmware/     Rust (ESP-IDF) node firmware
ai/               AI_Agent_Notes.md
├── emulator/     Emulator ESP32 — 3 simulated nodes for early AI testing
├── ha/           Home Assistant config: MQTT entities, automations, dashboards (no secrets)
└── cloud/        Lambda agent harness, chat bot, deployment config
docs/             Project_Guideline.md, Collaboration_Guidelines.md
└── interfaces/   Shared contracts between teams — one file each
```

## Shared contracts

A contract (pin map, MQTT schema, `/ha-event` format, AI message format, …) is **written in
`docs/interfaces/` before anyone builds against it**, and changed only after the teams on
both sides agree. The full list and due dates are in the Collaboration Guidelines §0.

## Code style

- **Rust (firmware):** run `cargo fmt` and `cargo clippy` before opening a PR. Test on Rig S.
- **Python (cloud agent):** PEP 8, formatted with `black`.
- **YAML (Home Assistant):** 2-space indent.
- Comment the *why*, not the *what*.

## Hardware files

- Each KiCad hierarchical sheet (Power / MCU / I/O) has one owner at a time — **never two
  people editing the same sheet at once**; schematic files don't merge well.
- Keep every symbol and footprint the project uses in `hardware/`, so nobody's personal
  library breaks the build.
- **Commit** schematics, layouts, CAD source, BOMs. **Don't commit** re-exportable output:
  gerbers, STLs, build output, autosave/backup files. `.gitignore` covers the common ones.
- Anything over ~10 MB, ask in Discord first.

## Secrets

**Never commit** Wi-Fi passwords, HA tokens, OpenRouter / AWS keys, chat bot tokens or
webhook secrets — not in code, not in config, not "temporarily."

- **ESP32:** Wi-Fi credentials and broker address live in NVS, never hardcoded.
- **Cloud:** AWS Secrets Manager / Lambda environment config.
- **Home Assistant:** `secrets.yaml` (gitignored), referenced with `!secret`.
- **Local:** gitignored config files.

If you commit a secret by accident, tell a lead immediately and rotate it. Deleting it in a
follow-up commit doesn't help — it stays in git history.

## Getting unstuck

- Work sessions: Sundays, 1:00–4:00 PM. Recaps go to Discord.
- Questions that affect another team go in **that team's Discord channel**, not DMs.
- **When unsure, ask — don't assume.** A wrong guess about a pin, voltage or JSON field
  costs far more than a question.
