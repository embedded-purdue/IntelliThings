----

# **Placeholder**

----

# Onboarding

Start here on your first day.

## Everyone

### 1. Get access

You should have a GitHub invite in your inbox — accept it. If not, ping a lead.

### 2. Clone and get on `dev`

```bash
git clone https://github.com/embedded-purdue/IntelliThings.git
cd IntelliThings
git checkout dev
```

### 3. Read these, in order

1. [`../README.md`](../README.md) — what we're building
2. [`architecture.md`](architecture.md) — how the pieces fit
3. [`../CONTRIBUTING.md`](../CONTRIBUTING.md) — how we branch, commit, and review
4. Your subteam's README (below)

### 4. Set your git identity

```bash
git config user.name "Your Name"
git config user.email "your@purdue.edu"
```

### 5. Copy the env template

```bash
cp .env.example .env
```

Fill in what you need. `.env` is gitignored — keep it that way.

---

## Software

**Stack:** Embedded C · ESP-IDF · FreeRTOS · ESP32-C5

```bash
# macOS
brew install cmake ninja dfu-util python3

# ESP-IDF (v5.x)
mkdir -p ~/esp && cd ~/esp
git clone -b v5.3 --recursive https://github.com/espressif/esp-idf.git
cd esp-idf && ./install.sh esp32c5
. ./export.sh    # run this in every new shell, or alias it
```

VS Code: install the **Espressif IDF** extension and point it at `~/esp/esp-idf`.

Verify:
```bash
idf.py --version
```

Then see [`../software/README.md`](../software/README.md).

---

## Hardware

**Stack:** KiCad or Altium · Fusion 360 / SolidWorks · 3D printing

- **KiCad** — free, cross-platform: https://www.kicad.org/download/
- **Altium** — check Purdue licensing before buying anything
- **Fusion 360** — free with a student license
- Datasheets for every BOM part live in the shared Drive folder

Before you touch a layout, read [`../hardware/README.md`](../hardware/README.md) —
especially the design review rule. We have five prototype fabs, total.

---

## AI

**Stack:** Home Assistant OS · Raspberry Pi 5 · MQTT · Matter · cloud LLM · MCP

This subteam spans the hub and the agent. Pick whichever end you're starting on —
most people end up touching both.

### Home Assistant side

You don't need physical hardware to start. Run HA locally:

```bash
docker run -d --name homeassistant \
  --privileged --restart=unless-stopped \
  -e TZ=America/Indiana/Indianapolis \
  -v "$(pwd)/ai/home-assistant:/config" \
  --network=host \
  ghcr.io/home-assistant/home-assistant:stable
```

Open http://localhost:8123.

Worth reading early:
- Home Assistant MQTT integration
- Home Assistant **AI Task** (newer feature — docs are thin, expect to experiment)
- Home Assistant **MCP Server** integration

### Agent side

```bash
cd ai
python3 -m venv .venv && source .venv/bin/activate
pip install -r requirements.txt   # once it exists
```

You'll need an API key in `.env` — ask a lead, don't buy your own.

Useful background:
- Model Context Protocol: https://modelcontextprotocol.io
- Home Assistant MCP Server integration docs

Then see [`../ai/README.md`](../ai/README.md).

---

## Stuck?

Post in your subteam's Discord channel. Don't sit on a blocker until Sunday — a
five-minute answer from someone who's hit it before beats three days of solo debugging.

Meeting recaps are posted to Discord after every session. Missed one? It's there.
