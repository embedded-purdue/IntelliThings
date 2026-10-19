# Emulator ESP32

Owner: @spicybutter (PM) · AI subteam · firmware **v2.1.0** (in development; the board runs
`2.1.0-dev` builds) · plan: [`PLAN-v2.1.0.md`](PLAN-v2.1.0.md) · progress and test log:
[`TODO.md`](TODO.md)

One **LuatOS ESP32-C3 Core** board with no sensors attached, pretending to be **three
IntelliThings sensor nodes**: kitchen, bedroom and living room.
- **Same contract as real nodes:** it publishes on exactly the same MQTT topics and JSON schema,
  so the AI team can build and test the whole **MQTT → Home Assistant → agent → display**
  pipeline before the real nodes exist. When real nodes arrive, nothing in HA or the agent
  changes except which devices are enabled.
- **Demo mode (experimental):** it also stands in for the ECE SPARK **demo master**: a whole
  demo home, 50 preset scenarios and a *Roll the house* button.

You drive it from a **web control panel**:
- make a room hot, fill it with smoke, walk someone in or out, or pin CO₂ to an exact value;
- read the AI messages each room received;
- plot every reading over the last hour;
- watch the MQTT traffic live.

| Document | For |
|---|---|
| **[`GUIDE.md`](GUIDE.md)** | **The user guide.** Every part of the panel, settings, demo mode and the HTTP API. Also built into the board: **?** in the panel, or `/guide.md` |
| This README | Worked examples for the agent tests, scripting, MQTT topics, Home Assistant, building and flashing |
| [`PLAN-v2.1.0.md`](PLAN-v2.1.0.md) | Why v2.1.0 is built the way it is, and the capacity budget |
| [`TODO.md`](TODO.md) | What's done and tested, and what still needs testing |
| [`releases/v0.1.0/`](releases/v0.1.0/README.md) | The previous firmware, frozen, and how to fall back to it |

**Contents**

- [Quick start](#quick-start)
- [What's new in v2.1.0](#whats-new-in-v210)
- [Emulating situations: worked examples](#emulating-situations-worked-examples)
- [Scenario reference](#scenario-reference)
- [Automatic behaviour](#automatic-behaviour-what-happens-when-you-do-nothing)
- [Demo mode](#demo-mode)
- [Scripting and automation](#scripting-and-automation)
- [MQTT topics](#mqtt-topics)
- [Home Assistant](#home-assistant)
- [Build, configure and flash (maintainer)](#build-configure-and-flash-maintainer)
- [Testing](#testing)
- [Troubleshooting](#troubleshooting)
- [Mapping to the real nodes](#mapping-to-the-real-nodes)
- [Source layout](#source-layout)

---

## Quick start

1. **Power the board** from any USB-C port or charger. It joins the team Wi-Fi and the MQTT
   broker by itself in about 20 s and keeps its settings through power cycles.
2. **Open the control panel** from a laptop or phone on the same Wi-Fi:
   - **`http://192.168.1.4/`**: the board's fixed IP on the team network;
   - `http://intellithings-emu.local/`: some phones and networks block `.local` names.
3. **Check the badges at the top right:** panel ✓, Wi-Fi, **MQTT 3/3** and clock ✓ should all be
   green. Click any badge for the system status.
4. **The rooms are publishing:** each publishes a full snapshot every **3 seconds** on
   `intellithings/emu-kitchen/sensors`, `…/emu-bedroom/sensors` and `…/emu-living-room/sensors`.
   Build your HA entities and agent against those, then use the **Emulate** card to create the
   situations your automation or agent should react to.

> The panel has **no login**. Anyone on the team Wi-Fi can drive the emulator, so announce a
> long test in Discord so nobody changes the rooms under you.

**New network or broker?** Open **⚙ Settings → MQTT broker**, enter the Raspberry Pi's IP and
press **Apply & reconnect**. No reflashing is needed; see
[the guide](GUIDE.md#settings).

---

## What's new in v2.1.0

| Area | v0.1.0 | v2.1.0 |
|---|---|---|
| Broker | Compiled in | **Changeable from the panel** (host/IP, port, login, keepalive, publish interval), saved on the board; reconnects without a reboot |
| Panel | One long page | **Dev / Demo tabs**; every card can be hidden; settings and guide popups |
| Readings | Current values | **°C and °F side by side**, previous value and change, sparkline, **plots over 10 min / 30 min / 1 h** in six styles, last 20 values |
| AI messages | A list | **Three display-style boxes**, one per room, with the 60-character counter, a per-room test sender and the agent's **thinking** state; the overall summary |
| Diagnostics | Last command | **System log** (commands and firmware log, filter, download) and a **live MQTT route diagram** |
| Help | README | **Searchable guide built into the board**, with a download link |
| Demo | — | **Demo mode** stand-in: 61-value demo home, 50 presets, scenario loop, *Roll the house* with a rolling animation, on `intellithings/emulator/demo/#` |
| Firmware | 1.12 MB debug build | Size-optimised (1.07 MB of a 1.9 MB slot); two app slots; panel gzipped |

Every v0.1.0 command, HTTP endpoint and MQTT topic still works unchanged, so existing scripts
and `register_emulator.py` keep working.

---

## Emulating situations: worked examples

Each example gives:
- the panel steps;
- the same thing as a command (for scripts, see [Scripting](#scripting-and-automation));
- what appears on MQTT;
- what a well-behaved agent should do.

Together they cover the AI team's standard test set: hot room with presence, rising CO₂, PM2.5
spike, empty room, nothing wrong.

> **Before each run:** in the **Emulate** card, choose the room → **Reset room**, so earlier
> tests don't leak in.

### Example 1: Hot room with someone in it

*The agent should cool the room (fan or thermostat) and tell the occupant.*

**Panel:** Emulate → **Living Room** → **🥵 Hot & occupied**.

```json
{"node":"living-room","action":"scenario","name":"hot"}
```

Over the next few minutes, `intellithings/emu-living-room/sensors` shows:

```json
{"node_id":"emu-living-room","temperature_c":28.6,"humidity_pct":61.2,"voc_index":104,
 "co2_ppm":702,"lux":312.5,"distance_mm":640,"pm1_0_ugm3":3,"pm2_5_ugm3":5,
 "pm10_ugm3":7,"presence":true,"ts":"2026-09-27T16:05:00-04:00"}
```

Temperature climbs to 28–30 °C and humidity to 55–65 %, with presence `true`, for 30 minutes.
**Expected agent behaviour:** turn on the living-room fan or lower the thermostat, then send
something like *"29 °C here; fan on."* to the living room's display.

**Faster, exact version:** Emulate → **more: custom values** → Temperature 29/29, Humidity 60/60,
Presence *present*, tick **jump instantly**, Apply.

```json
{"node":"living-room","action":"set","ranges":{"temperature_c":29,"humidity_pct":60},
 "presence":true,"instant":true}
```

### Example 2: CO₂ building up while someone sleeps

*The agent should suggest ventilation, gently. It's night and someone is asleep.*

**Panel:** Emulate → **Bedroom** → **😴 Sleep**, then **😮‍💨 Stuffy room**. Or pin it under
custom values: CO₂ 1300/1600, Presence *present*, Apply (leave *jump instantly* off to watch it
rise).

```json
{"node":"bedroom","action":"scenario","name":"stuffy"}
{"node":"bedroom","action":"set","ranges":{"co2_ppm":{"min":1300,"max":1600}},"presence":true}
```

Starting *Stuffy room* replaces *Sleep*. To keep the room dark as well, use the custom-value
version and add Light 0/2.

**Expected:** CO₂ passes 1000 ppm within a couple of minutes. The agent should turn on
ventilation or the purifier, or suggest opening a window. It shouldn't turn lights on in a dark,
occupied bedroom at night.

### Example 3: Burnt food in the kitchen (PM2.5 spike)

*The agent should start the air purifier and/or range hood.*

**Panel:** Emulate → **Kitchen** → **🔥 Burnt food** (10 min).

```json
{"node":"kitchen","action":"scenario","name":"burnt_food"}
```

**Expected on MQTT:** over the 10 minutes:
- PM2.5 climbs to 150–300 µg/m³, and PM1.0 and PM10 follow;
- VOC Index rises to 400–500, CO₂ to about 1000–1400 ppm;
- temperature rises about 3 °C, and presence is `true`.

Afterwards the PM decays gradually, as smoke would.

**Expected agent behaviour:** purifier on high, maybe the range hood, plus a display message.

A good follow-up check: does it turn the purifier *off* again once PM2.5 is back to normal? For a
milder test, use **🌫 PM2.5 spike** (PM2.5 80–150 only).

### Example 4: Everyone leaves the room

*The agent should switch off lights and devices in an empty room.*

**Panel:** Emulate → **Living Room** → **🛋 Gathering** (4 people, lights on). Wait a minute, then
**🚪 Leave**.

```json
{"node":"living-room","action":"scenario","name":"gathering"}
{"node":"living-room","action":"scenario","name":"leave"}
```

**Expected on MQTT:**
- `presence` goes `false`;
- `distance_mm` returns to about 1100 (nobody near the unit);
- lux drops as the lamps go off;
- CO₂ decays slowly.

**Expected agent behaviour:** lights and fan off in the living room, and nothing done in the
other two rooms.

### Example 5: Nothing wrong (the "do nothing" test)

*The agent should NOT act.* This is as important as the others.

**Panel:** Emulate → **All rooms** → **Reset room**. All three rooms return to calm readings
(≈20–23 °C, CO₂ ~450–500 ppm, PM2.5 < 10).

```json
{"node":"all","action":"reset"}
```

**Expected:** no tool calls, or at most a short "all good" if your prompt asks for one.

At meal times the kitchen may start **cooking by itself** (the card then shows `auto: cooking`).
For a guaranteed quiet baseline, pin the values:
`{"node":"all","action":"set","ranges":{"co2_ppm":480,"pm2_5_ugm3":5,"voc_index":100},"instant":true}`.

### Example 6: Exact, repeatable scenario for comparing models

When you compare models on OpenRouter, every model should see **the same numbers**. Use custom
values with fixed values, **jump instantly**, and a duration that covers the run:

```json
{"node":"kitchen","action":"set","ranges":{"temperature_c":27.5,"humidity_pct":68,
 "co2_ppm":1250,"voc_index":180,"pm2_5_ugm3":12},"presence":true,"instant":true,"duration_s":600}
```

Every snapshot for the next 10 minutes carries exactly those values.
- **PM1.0 and PM10** follow the pinned PM2.5 (≈ 0.65× and 1.3–1.5×) unless you pin them too.
- **Lux and distance** keep living normally.

### Example 7: Test the display path without the agent

**Panel:** AI messages → under the **Bedroom** box type *"CO₂ is high, open the window."*, tick
**retained**, **Send**.

- **Where it shows up:** within a second in the Bedroom box, which also gives its character
  count against the 60-character room-message limit. It also prints as a banner on the serial
  monitor.
- **On the MQTT route diagram:** a dot goes from the control connection to the broker and back
  to the rooms.
- **What it proves:** this is exactly what happens when the pipeline publishes the agent's
  answer, so it verifies topic names, JSON format and the retained flag end to end.

To clear a retained message afterwards, publish an empty retained payload to the topic (see
[Scripting](#scripting-and-automation)).

### Quick map: agent test → emulator action

| Test scenario | Panel (Emulate → room → button) | Command |
|---|---|---|
| Hot room + presence | Living Room → 🥵 Hot & occupied | `{"node":"living-room","action":"scenario","name":"hot"}` |
| Rising CO₂ | Bedroom → 😮‍💨 Stuffy room | `{"node":"bedroom","action":"scenario","name":"stuffy"}` |
| PM2.5 spike | Kitchen → 🔥 Burnt food (or 🌫 PM2.5 spike) | `{"node":"kitchen","action":"scenario","name":"burnt_food"}` |
| VOC spike | Living Room → 🧴 VOC spike | `{"node":"living-room","action":"scenario","name":"voc_spike"}` |
| Empty room | any → 🚪 Leave | `{"node":"living-room","action":"scenario","name":"leave"}` |
| Someone arrives | any → 🚶 Enter | `{"node":"bedroom","action":"scenario","name":"enter"}` |
| Nothing wrong | All rooms → Reset room | `{"node":"all","action":"reset"}` |
| Stop a test | any → ↺ Back to normal | `{"node":"all","action":"clear"}` |

---

## Scenario reference

"Target" values are held inside the range for the scenario's duration; "+N" shifts the room's
normal value by N. Anything not listed keeps its normal behaviour.

| Shortcut | `name` | Typical room | Default length | What changes |
|---|---|---|---|---|
| 🍳 Cook | `cooking` | Kitchen | 20 min | Presence on, lights on (350–600 lux), temp +2.5 °C, humidity +15 %, VOC 250–350, CO₂ 900–1300, PM2.5 40–80 |
| 🔥 Burnt food | `burnt_food` | Kitchen | 10 min | Presence on, lights on, temp +3 °C, humidity +10 %, VOC 400–500, CO₂ 1000–1400, PM2.5 150–300 |
| 😴 Sleep | `sleep` | Bedroom | 8 h | 1 person present, lights off (0–3 lux); CO₂ builds slowly from breathing |
| 🛋 Gathering | `gathering` | Living room | 60 min | 4 people, lights on (250–450 lux), temp +1 °C; CO₂ rises from 4 people breathing |
| 🥵 Hot & occupied | `hot` | any | 30 min | Presence on, temperature 28–30 °C, humidity 55–65 % |
| 🌫 PM2.5 spike | `pm_spike` | any | 10 min | PM2.5 80–150 (PM1.0 and PM10 follow) |
| 🧴 VOC spike | `voc_spike` | any | 10 min | VOC Index 300–450 |
| 😮‍💨 Stuffy room | `stuffy` | any | 30 min | Presence on, CO₂ 1200–1800 |
| 🪟 Open window | `window_open` | any | 20 min | Temp −2 °C, humidity +5 %, CO₂ 420–500, PM2.5 8–15 |
| 🚶 Enter | `enter` | any | 15 min | Presence on (someone comes in; CO₂ starts rising) |
| 🚪 Leave | `leave` | any | 15 min | Presence off, lights off; CO₂ decays |
| 🌙 Lights off | `lights_off` | any | 30 min | Light 0–3 lux |

- **PM ratios:** PM1.0 is about 0.65 × PM2.5 and PM10 about 1.3–1.5 × PM2.5, as in real smoke
  and dust.
- **Distance:** when someone is present, `distance_mm` drops to 300–900 mm now and then (someone
  at the desk); otherwise it sits at ~1100 mm.
- **How fast values move:**
  - light, distance and presence: within seconds;
  - VOC and PM: ~3 min;
  - CO₂: ~6 min;
  - temperature and humidity: ~9 min.

  For an exact value right away, use custom values with **jump instantly**.

---

## Automatic behaviour (what happens when you do nothing)

Readings are **not** random numbers. Each room keeps its own state and behaves like a room,
following the real local time (US Eastern):

- **People come and go** with realistic stays (5–90 min, hours for sleep), more often at busy
  times of day. Presence stays on while someone sits still or sleeps, like the real mmWave
  sensor.
- **Kitchen:** often **cooks by itself at meal times** (6:30–9:00, 11:30–13:30, 17:00–20:00),
  shown as `auto: cooking` on the card: warmer, more humid, higher VOC/PM2.5/CO₂.
- **Bedroom:** calm; occupied and dark at night (23:00–6:30), CO₂ creeping up while someone
  sleeps.
- **Living room:** up to 4 people in the evening; occasional **candle/cleaning** VOC events and
  **dust** PM events.
- **CO₂** rises with the number of people and decays towards outdoor air (~420 ppm).
- **Light** follows daylight for the hour, plus lamps when someone is there.
- Small sensor noise on every reading; the three rooms never move in lockstep.

**Priority:** anything you start takes priority: **custom values > shortcut > automatic
behaviour**.
- Starting a shortcut cancels any automatic event in that room, and no new one starts until the
  shortcut ends.
- Custom values don't stop automatic events, but the readings you pinned stay pinned.

---

## Demo mode

**Experimental.** The **Demo** tab stands in for the ECE SPARK demo master, so the AI team can
build HA entities, automations and agent behaviour for demo mode before the real buzzer board
exists.

- **The demo home:** 61 values in 9 areas (home, outside, kitchen, living room, master bedroom,
  second bedroom, garage, patio, garden). It covers readings and device states: lights,
  windows, doors, the car, presence, weather and time of day. Each value has caps and a normal
  range.
- **50 presets:** 20 normal homes; 30 abnormal ones (air quality, heat and cold, security,
  energy waste, hazards).
- **ROLL THE HOUSE:**
  1. Every value spins.
  2. The values land on a random home within the caps (*lean to normal* or uniform) and are
     published once.
  3. Presses are ignored for the roll plus the hold (default 4 s + 120 s), like the buzzer.
- **Scenario loop:** presets, rolls or a mix, every *interval* seconds (default 90).
- **Publishing:** to `intellithings/emulator/demo/#`, **off after every boot**. The three Dev
  rooms keep publishing on their usual topics either way.

The entities, caps and presets are a **draft for the AI team** in
[`esp32_c3_emulator/demo/demo_home.json`](esp32_c3_emulator/demo/demo_home.json). The build
validates that file and turns it into C, so editing the demo home means editing the JSON,
rebuilding and flashing.

Topics, payloads and commands: [the guide](GUIDE.md#demo-mode-over-mqtt-and-http). Panel
controls: [the guide](GUIDE.md#demo-mode).

---

## Scripting and automation

Everything the panel does is a small JSON command, over **HTTP** or **MQTT**. Use it for
automated test runs and model comparisons.

### Commands

`node` is a node ID (`emu-kitchen`), a room name (`kitchen`, `bedroom`, `living-room`) or
`"all"`.

```jsonc
// Start a shortcut (duration_s optional: default per scenario, 0 = until cleared)
{"node":"emu-kitchen","action":"scenario","name":"cooking","duration_s":1200}

// Force values (a single number = fixed value, or {"min","max"}), plus presence
{"node":"emu-bedroom","action":"set","ranges":{"co2_ppm":{"min":1300,"max":1500},"temperature_c":26},
 "presence":true,"duration_s":600,"instant":false}

// Back to automatic behaviour / also reset readings to baseline
{"node":"all","action":"clear"}
{"node":"emu-kitchen","action":"reset"}

// Publish a test AI message to a room's display topic (single room only, ≤ 400 characters)
{"node":"emu-kitchen","action":"display","text":"Turn on the range hood.","retain":false}

// Demo mode (no "node"): publish, roll, preset, loop, settings — see GUIDE.md
{"action":"demo","op":"roll"}
{"action":"demo","op":"preset","id":35}
```

Metric keys are the payload field names: `temperature_c`, `humidity_pct`, `voc_index`,
`co2_ppm`, `lux`, `distance_mm`, `pm1_0_ugm3`, `pm2_5_ugm3`, `pm10_ugm3`. Invalid commands are
rejected with a reason (HTTP 400, `last_cmd` in the state, and the panel's command log) and never
change anything.

### Over HTTP

| Endpoint | Returns |
|---|---|
| `POST /api/cmd` | Run a command; `{"ok":true,"msg":"..."}` or HTTP 400 with the reason |
| `GET /api/state` | Status, each room's mode, occupants and last reading, settings summary, demo state |
| `GET /api/messages` | The last 20 AI messages |
| `GET /api/meta` | Metrics with ranges, scenarios, node IDs and topics, history layout, firmware version |
| `GET /api/poll`, `/api/history` | What the panel uses: state plus new log lines, commands, MQTT traffic and messages; reading history (binary). Formats in [the guide](GUIDE.md#http-api) |
| `GET`/`POST /api/config`, `POST /api/config/reset` | Broker and publish settings ([guide](GUIDE.md#settings-over-http)) |
| `GET /api/demo/meta`, `/api/demo`, `/api/demo/history` | Demo home ([guide](GUIDE.md#demo-mode-over-mqtt-and-http)) |

```bash
# macOS / Linux / Git Bash
curl -X POST http://192.168.1.4/api/cmd -H 'Content-Type: application/json' \
  -d '{"node":"kitchen","action":"scenario","name":"burnt_food"}'
```

```powershell
# Windows PowerShell
Invoke-RestMethod -Method Post -Uri http://192.168.1.4/api/cmd `
  -ContentType 'application/json' -Body '{"node":"kitchen","action":"scenario","name":"burnt_food"}'
```

A test script in Python (standard library only) that runs the standard scenarios one after
another:

```python
import json, urllib.request

EMU = "http://192.168.1.4"

def cmd(body):
    req = urllib.request.Request(EMU + "/api/cmd", data=json.dumps(body).encode(),
                                 headers={"Content-Type": "application/json"}, method="POST")
    with urllib.request.urlopen(req, timeout=5) as r:
        print(json.load(r)["msg"])

tests = [
    ("hot room + presence", {"node": "living-room", "action": "set", "instant": True,
                             "ranges": {"temperature_c": 29, "humidity_pct": 60}, "presence": True}),
    ("rising CO2",          {"node": "bedroom", "action": "set", "instant": True,
                             "ranges": {"co2_ppm": 1500}, "presence": True}),
    ("PM2.5 spike",         {"node": "kitchen", "action": "set", "instant": True,
                             "ranges": {"pm2_5_ugm3": 120}}),
    ("empty room",          {"node": "living-room", "action": "set", "instant": True,
                             "presence": False, "ranges": {"lux": 0}}),
]
for name, body in tests:
    cmd({"node": "all", "action": "reset"})
    cmd(body)
    input(f"--- {name}: check what the agent did, then press Enter")
cmd({"node": "all", "action": "clear"})
```

### Over MQTT

Publish the same JSON to **`intellithings/emulator/cmd`**, **not retained**. The result shows up
within ~0.5 s in the retained `intellithings/emulator/state` topic (same JSON as
`GET /api/state`, also refreshed every 3 s), and in the panel's command log with source `mqtt`.

```bash
B="-h 192.168.1.2 -u <user> -P <password>"
mosquitto_pub $B -t intellithings/emulator/cmd -q 1 \
  -m '{"node":"emu-kitchen","action":"scenario","name":"cooking"}'
```

From Home Assistant (e.g. a script button that sets up a test):

```yaml
action: mqtt.publish
data:
  topic: intellithings/emulator/cmd
  payload: '{"node":"bedroom","action":"scenario","name":"stuffy"}'
```

**Don't publish commands with `-r` / `retain: true`.**
- **What happens:** a retained command runs once immediately and is then ignored on every
  reconnect.
- **To clear one:** publish an empty retained payload:
  `mosquitto_pub $B -t intellithings/emulator/cmd -r -n`.

### Watching and sending AI messages from a terminal

```bash
mosquitto_sub $B -t 'intellithings/#' -v                      # everything

mosquitto_pub $B -t intellithings/emu-bedroom/display -r -q 1 \
  -m '{"text":"CO2 high. Open a window.","ts":"2026-10-18T21:30:00+00:00"}'

mosquitto_pub $B -t intellithings/emu-bedroom/display -r -n   # clear the retained message
```

---

## MQTT topics

Full contract: [`docs/interfaces/mqtt.md`](../../docs/interfaces/mqtt.md) (agreed Oct 4).
Broker: the Mosquitto container on the HA Pi, `192.168.1.2:1883` (changeable in ⚙ Settings).

| Topic | Direction | QoS / retained | Payload |
|---|---|---|---|
| `intellithings/<id>/sensors` | emulator → broker, **every 3 s even if unchanged** | 0 / no | Sensor snapshot JSON (every field, every time; `ts` once the clock has synced) |
| `intellithings/<id>/status` | emulator → broker | 1 / yes | `online`, or `offline` via Last Will ~20 s after the board drops |
| `intellithings/<id>/display` | pipeline (via HA) → emulator | 1 / yes | `{"text": "...", "ts": "..."}` |
| `intellithings/ai/summary`, `…/inference_status` | pipeline → emulator (read-only) | 1 / yes | Overall summary and thinking state, shown in the AI card |
| `intellithings/emulator/status` | emulator → broker | 1 / yes | `online` / `offline` for the control connection |
| `intellithings/emulator/state` | emulator → broker | 0 / yes | Status JSON |
| `intellithings/emulator/cmd` | you → emulator | not retained | Command JSON |
| `intellithings/emulator/demo/state`, `/home`, `/<id>/sensors` | emulator → broker | 1 / yes | Demo mode, *experimental* ([guide](GUIDE.md#demo-mode-over-mqtt-and-http)) |

- **`<id>`** is `emu-kitchen`, `emu-bedroom` or `emu-living-room`; the `emu-` prefix always
  marks emulated data.
- **Connections:** each room has **its own MQTT connection** (client ID `intellithings-<id>`) and
  its own Last Will, so HA sees three independent devices. They share one board, so if it loses
  power or Wi-Fi, all three go `offline` together.
- **Broker change from Settings:** each connection first publishes a retained `offline` on the
  old broker, then reconnects to the new one.
- **Reserved namespace:** `intellithings/emulator/#` belongs to the emulator; real nodes and HA
  must not depend on it.

---

## Home Assistant

**How HA gets the emulator's entities today:**
[`ai/deployment/home_assistant/register_emulator.py`](../deployment/home_assistant/register_emulator.py)
publishes retained MQTT discovery configs for the 30 sensor entities. Setup:
[`ai/deployment/mosquitto/README.md`](../deployment/mosquitto/README.md#emulator-readings-in-home-assistant).
- **What it reads:** the aggregate `intellithings/emulator/state` topic, not the per-node
  `…/sensors` topics. That's temporary; real nodes need discovery on their own topics.
- **Entity names:** [`docs/interfaces/ha-entities.md`](../../docs/interfaces/ha-entities.md).
- **Demo-mode entities aren't registered yet:** that's the AI team's next step, reading
  `intellithings/emulator/demo/home`.

For reference, this is how a per-node entity looks in plain MQTT YAML, the way real nodes will
be set up:

```yaml
mqtt:
  sensor:
    - name: "Kitchen CO2"
      unique_id: emu_kitchen_co2
      state_topic: intellithings/emu-kitchen/sensors
      value_template: "{{ value_json.co2_ppm }}"
      unit_of_measurement: ppm
      device_class: carbon_dioxide
      availability_topic: intellithings/emu-kitchen/status
```

**Rate and history:**
- **Throttle the trigger:** readings arrive every 3 s, so throttle the hand-off automation
  (thresholds plus a minimum time between calls) so the LLM isn't called on every snapshot.
- **Recorder:** consider excluding the emulator entities from HA's recorder.

---

## Build, configure and flash (maintainer)

You only need this to change the firmware. The broker address and publish interval are set from
the panel, so they no longer need a rebuild.

### Hardware

**LuatOS ESP32-C3 Core**: 4 MB flash, single-core RISC-V. No GPIOs are used, just USB-C. Both
variants work unchanged:
- **Classic:** CH343 USB-serial chip; install the CH343 driver on Windows.
- **Native USB:** shows up as *USB JTAG/serial debug unit*.

### 1. Configure (credentials are never committed)

```bash
cd ai/emulator/esp32_c3_emulator
cp sdkconfig.local.example sdkconfig.local     # gitignored
# edit sdkconfig.local: Wi-Fi SSID/password, broker host/port, broker username/password
rm -f sdkconfig                                # so the new values are picked up
```

- **What the broker values are:** the *defaults*. Settings saved from the panel override them,
  and **⚙ Settings → Restore defaults** goes back to them.
- **Broker host:** use the **Pi's IP address**; `homeassistant.local` doesn't resolve on the
  team network.
- **Login:** give the emulator its own MQTT login; today it still uses the broker's
  `homeassistant` user (see [`TODO.md`](TODO.md)).
- **Alternative:** `idf.py menuconfig` → **IntelliThings Emulator**.

| Setting | Default |
|---|---|
| `CONFIG_EMU_WIFI_SSID` / `CONFIG_EMU_WIFI_PASSWORD` | — (required, 2.4 GHz network) |
| `CONFIG_EMU_MQTT_HOST` / `CONFIG_EMU_MQTT_PORT` | `192.168.1.2` / `1883` |
| `CONFIG_EMU_MQTT_USERNAME` / `CONFIG_EMU_MQTT_PASSWORD` | empty (anonymous) |
| `CONFIG_EMU_MQTT_KEEPALIVE_S` | `15` |
| `CONFIG_EMU_PUBLISH_INTERVAL_MS` | `3000` |
| `CONFIG_EMU_SIM_TICK_MS` / `CONFIG_EMU_SIM_SPEED` | `1000` / `1` (raise the speed to fast-forward CO₂ build-up, decay and occupancy) |
| `CONFIG_EMU_NODE_ID_KITCHEN` / `_BEDROOM` / `_LIVING_ROOM` | `emu-kitchen` / `emu-bedroom` / `emu-living-room` |
| `CONFIG_EMU_TIMEZONE` | `EST5EDT,M3.2.0,M11.1.0` |
| `CONFIG_EMU_HOSTNAME` | `intellithings-emu` |
| `CONFIG_EMU_LOG_READINGS` | on (one serial line per room per publish) |

### 2. Build, flash, monitor

Requirements: **ESP-IDF v6.1**, and internet access on the first build (MQTT, cJSON and mDNS
come from the ESP component registry).

```bash
. $HOME/esp/esp-idf/export.sh        # Windows: the ESP-IDF v6.1 PowerShell profile (not Git Bash)
idf.py set-target esp32c3            # first time only
idf.py build
idf.py -p COM8 flash monitor         # macOS/Linux: -p /dev/cu.usbmodem… or /dev/ttyUSB0
```

Exit the monitor with `Ctrl+]`.

**What the build does besides compiling:**
- gzips the panel (`main/web/index.html`) and the guide (`GUIDE.md`) into the firmware;
- generates the demo-home tables from `demo/demo_home.json`, stopping with a message if the JSON
  is invalid.

**When a full flash is needed:**
- **From v0.1.0 to v2:** the partition table changed (two 1.9 MB app slots, `partitions.csv`).
  The first time you flash v2 over v0.1.0, use `idf.py -p COM8 flash -a`. `idf.py app-flash`
  alone leaves the old table, and the board won't boot.
- **After changing `sdkconfig.defaults`:** delete `sdkconfig` so the new defaults are picked up.

**Version:** `PROJECT_VER` in `CMakeLists.txt` is the only place it's set. It's `2.1.0-dev`
until the release.

### Expected serial output

```
I (414) emulator: IntelliThings emulator 2.1.0-dev booting: 3 virtual nodes
I (424) config: Broker mqtt://192.168.1.2:1883, publish every 3000 ms (defaults)
I (424) nodes: Kitchen      -> emu-kitchen
I (424) nodes: Bedroom      -> emu-bedroom
I (424) nodes: Living Room  -> emu-living-room
I (434) demo: 9 areas, 61 entities, 50 presets; publishing off until turned on
I (15834) time: SNTP started (pool.ntp.org)
I (15844) web: Control panel running on port 80
I (15854) emulator: Control panel: http://192.168.1.4/  (or http://intellithings-emu.local/)
I (15904) mqtt: emu-kitchen: connected; published online, subscribing to intellithings/emu-kitchen/display
I (15931) ctrl: emulator: connected; listening on intellithings/emulator/cmd and intellithings/ai/*
I (19571) node: emu-kitchen      T=22.2C RH=44.5% CO2=468 VOC=107 PM1/2.5/10=4/6/9 lux=378 dist=627mm PRESENT [auto]
```

**What the lines mean:**
- **Reading lines** end with the room's mode: `[auto]`, `[auto:cooking]`, `[scenario:stuffy]`
  or `[override]`. `(not sent: MQTT offline)` means the broker isn't connected.
- **Commands** are logged as `ctrl: [web] …` or `ctrl: [mqtt] …`.
- **AI messages** print as a banner. `[retained]` means the broker re-delivered a stored
  message after a (re)connect:

  ```
  ==================================================
  [AI MESSAGE] KITCHEN  (emu-kitchen)  [retained]
  Range hood on while cooking.
  ts: 2026-10-18T22:10:00+00:00
  ==================================================
  ```

The same log, minus the per-snapshot reading lines, is in the panel's **System log**.

### 3. Work on the panel without reflashing

```bash
python ai/emulator/tools/panel_dev_server.py      # then open http://localhost:8765/
```

It serves `main/web/index.html` and `GUIDE.md` from your checkout and forwards `/api/…` to the
board (`--board <ip>` for another address). Edit, reload, repeat; flash once you're done.

### 4. Release images and falling back

- **Release image:** make a single image with `idf.py merge-bin -o emulator-<version>-merged.bin`
  and keep it, with its SHA-256, in the **team Google Drive folder**.
  - **Never commit it:** it contains the Wi-Fi and broker passwords.
  - **Flashing it** needs only `esptool` (`write_flash 0x0 …`).
- **Fall back to v0.1.0** (frozen source, its guide and image): see
  [`releases/v0.1.0/README.md`](releases/v0.1.0/README.md#falling-back). The merged v0.1.0
  image also restores the old partition table.

---

## Testing

- **[`tests/api_check.py`](tests/api_check.py):** checks the whole HTTP API on a running board
  (standard library only). It changes the rooms while it runs and resets them at the end, so
  tell the team first.

  ```bash
  python ai/emulator/tests/api_check.py --host 192.168.1.4            # 66 checks
  python ai/emulator/tests/api_check.py --host 192.168.1.4 --demo     # + demo mode (~1 min)
  python ai/emulator/tests/api_check.py --host 192.168.1.4 --broker-test  # + switch brokers and back
  ```

- **Panel checklist:** [the guide's *How to test this portal*](GUIDE.md#how-to-test-this-portal).
- **Test log:** results and open checks are in [`TODO.md`](TODO.md).

---

## Troubleshooting

| Symptom | Check |
|---|---|
| Panel doesn't load at `intellithings-emu.local` | Use `http://192.168.1.4/`; some phones and networks block `.local` names. On another network, the IP is in the serial log and the router's client list |
| Panel shows **emulator unreachable** | Board unplugged, rebooting (~20 s), or your device is on another network |
| A button does nothing | Look at the toast, or **System log → Commands**: every rejection says why |
| Readings don't change after a shortcut | They move gradually (CO₂ ~6 min, temperature ~9 min). Use custom values + *jump instantly* for immediate values |
| Kitchen changes by itself | Automatic meal-time cooking (`auto: cooking`). Press **Reset room**, or pin values with custom values |
| Badge **MQTT 0/3** | Broker down, wrong address or wrong login. Check **⚙ Settings → MQTT broker** and **System log → System** (`transport error … is mqtt://… reachable?` = address; `broker refused connection (code 5)` = login) |
| Changed the broker to a wrong IP | The panel still works: fix it in ⚙ Settings, or **Restore defaults** |
| Cleared the username/password and the board restarted | Expected: a login can only be cleared with a reboot |
| `No Wi-Fi SSID configured` on serial | `sdkconfig.local` missing, or `sdkconfig` existed before you created it: delete `sdkconfig` and rebuild |
| `Association refused temporarily … comeback time` right after a reset | Normal with this router (PMF): it makes a just-rebooted board wait ~15 s; it retries and joins by itself |
| Wi-Fi keeps retrying (`reason 201/15/…`) | 2.4 GHz network? SSID/password correct? 15/204 = wrong password, 201 = SSID not found |
| AI message not shown | Topic must be exactly `intellithings/<id>/display`; payload `{"text":"..."}`. Other payloads show with a red **not {text} JSON** badge |
| AI box says **over 60 chars** | The pipeline's room messages are meant to be ≤ 60 characters; the board still shows up to 400 |
| AI message marked **truncated** | Payload over 512 bytes; keep display messages short |
| **ROLL THE HOUSE** is grey | It's locked during a roll and the hold after it (countdown on the button), like the buzzer |
| Demo values don't reach HA | Is **Publish to `intellithings/emulator/demo/#`** ticked? It's off after every boot |
| Build stops with `gen_demo.py: …` | The demo-home JSON has an error; the message names the preset and value |
| Board doesn't boot after flashing v2 over v0.1.0 | Do a full flash (`idf.py flash -a`); the partition table changed |
| No serial output (classic board) | CH343 driver; correct COM port |

---

## Mapping to the real nodes

| Emulator | Real node (ESP32-C6, Rust) |
|---|---|
| Three node IDs `emu-*` on one board | One node ID per board, derived from the MAC |
| Four MQTT connections (3 rooms + control) | One connection per node |
| Simulated readings every 3 s | Real sensor readings (interval to be agreed) |
| AI message in the panel's display-style box and on serial | AI message in the DWIN display's AI section |
| `intellithings/emulator/#` control and demo topics | Not used |

Topics, JSON fields, units, QoS, retained flags and Last Will behaviour are identical, so HA and
the agent don't care which one they're talking to.

---

## Source layout

```
ai/emulator/
├── README.md                 this file
├── GUIDE.md                  user guide (also built into the firmware, /guide.md)
├── PLAN-v2.1.0.md            v2.1.0 plan and capacity budget
├── TODO.md                   progress and test log
├── releases/v0.1.0/          frozen v0.1.0: source, guide, test log, fallback steps
├── tests/api_check.py        HTTP API checks against a running board
├── tools/panel_dev_server.py serve the panel from the checkout, API from the board
└── esp32_c3_emulator/
    ├── CMakeLists.txt            picks up sdkconfig.local; PROJECT_VER = firmware version
    ├── sdkconfig.defaults        target, flash, size optimisation, lwIP tuning (no secrets)
    ├── partitions.csv            nvs + two 1.9 MB app slots (ota_0, ota_1)
    ├── sdkconfig.local.example   credentials template
    ├── demo/demo_home.json       demo home: areas, entities, caps, 50 presets (draft)
    ├── tools/gzip_asset.py       gzips web assets at build time
    ├── tools/gen_demo.py         demo_home.json -> demo_data.c at build time
    └── main/
        ├── Kconfig.projbuild     "IntelliThings Emulator" menu
        ├── idf_component.yml     espressif/mqtt, cjson, mdns
        ├── main.c                boot, sim task, 3 s publish cycle
        ├── app_config.[ch]       runtime broker/publish settings in NVS
        ├── sim_model.[ch]        room profiles and environment model (plain C)
        ├── scenarios.c           shortcut scenario presets
        ├── payload.[ch]          topic and sensor-JSON formatting (plain C)
        ├── virtual_node.[ch]     node table and lock
        ├── history.[ch]          reading history for the plots (plain C)
        ├── demo_home.[ch]        demo home: preset fill, random roll, rules (plain C)
        ├── demo_engine.[ch]      demo loop / roll / hold task and demo topics
        ├── wifi_manager.[ch]     station mode, reconnect with backoff
        ├── time_sync.[ch]        SNTP, timezone, ts
        ├── mqtt_node.[ch]        one MQTT connection per node, Last Will, runtime broker change
        ├── emulator_ctrl.[ch]    commands, state JSON, control MQTT connection
        ├── ai_message.[ch]       AI messages, summary and thinking state
        ├── event_log.[ch]        system log hook, command log, MQTT route events
        ├── web_server.[ch]       control panel + HTTP API
        └── web/index.html        the panel (gzipped into the firmware)
```
