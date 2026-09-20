# Contributing to IntelliThings

Welcome aboard. This doc covers how we branch, commit, and review.

## Branching model

We use a two-tier model. Keep it simple, keep `main` demo-ready.

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

Prefix with your subteam so it's obvious who owns what:

```
software/i2c-sht40-driver
hardware/companion-pcb-rev-a
ai/mqtt-broker-setup
ai/status-statement-prompts
docs/architecture-update
```

Lowercase, hyphens, no spaces. Keep it short and descriptive.

## Day-to-day workflow

```bash
# 1. Start from an up-to-date dev
git checkout dev
git pull origin dev

# 2. Cut your branch
git checkout -b software/i2c-sht40-driver

# 3. Work, committing as you go
git add <files>
git commit -m "software: add SHT40 temperature/humidity driver"

# 4. Push and open a PR into dev
git push -u origin software/i2c-sht40-driver
```

Then open a pull request on GitHub targeting **`dev`**.

### Keeping your branch fresh

If `dev` has moved since you branched:

```bash
git checkout dev && git pull origin dev
git checkout software/i2c-sht40-driver
git merge dev          # resolve any conflicts here, not in the PR
```

## Commit messages

Format: `<area>: <what changed>`

```
software: add SHT40 temperature/humidity driver
hardware: route power plane on companion PCB rev A
ai: add MQTT broker config for Pi 5 hub
ai: tighten proactive status statement prompt
docs: correct BOM cost for CO2 sensor
```

Present tense, lowercase after the colon, no trailing period. Explain *why* in the
body if it isn't obvious from the diff.

## Pull requests

1. **Target `dev`**, not `main`.
2. Fill out the PR template — what changed, how you tested it, anything you're unsure about.
3. **At least one review** before merge. Tag your subteam lead if you're not sure who.
4. Keep PRs small. A 200-line PR gets reviewed today; a 2,000-line PR gets reviewed Sunday.
5. Draft PRs are welcome — open one early if you want eyes on a direction.

### Merging to `main`

Only at milestones, and only from `dev`, once the pipeline actually works end to end.
Subteam leads coordinate this — don't open a PR into `main` on your own.

## Hardware and binary files

- **Commit** schematics, PCB layouts, CAD source, BOMs. These are the deliverable.
- **Don't commit** generated artifacts: gerbers you can re-export, build output, STLs
  you can re-slice, autosave/backup files. `.gitignore` covers the common ones.
- Large binaries bloat clones permanently. If a file is over ~10 MB, ask in Discord first.

## Secrets — read this one

**Never commit API keys, Wi-Fi credentials, or tokens.** Not in code, not in config,
not "temporarily."

- Use `.env` files locally — they're gitignored.
- Home Assistant: use `secrets.yaml` (gitignored), reference with `!secret`.
- ESP-IDF: keep credentials in `sdkconfig` overrides or a gitignored header, never in source.

If you commit a secret by accident, tell a lead immediately and rotate it. Do not just
delete it in a follow-up commit — it stays in git history forever.

## Code style

- **Firmware (C):** follow the ESP-IDF style guide. 4-space indent, snake_case, no tabs.
- **Python:** PEP 8. Format with `black` if you have it.
- **YAML (Home Assistant):** 2-space indent.
- Comment the *why*, not the *what*. The code already says what it does.

## Getting unstuck

- Meetings: Sundays 1–4 PM. Recaps go to Discord.
- Blocked mid-week? Post in your subteam's Discord channel — don't sit on it until Sunday.
- Subteam assignments are advisory. Nine of eighteen of us have written firmware; if
  you want to help another team, do.
