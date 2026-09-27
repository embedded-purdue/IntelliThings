# Emulator ESP32

Owner: @spicybutter (PM) · AI subteam · status and test log: [`TODO.md`](TODO.md)

One **LuatOS ESP32-C3 Core** board with no sensors attached that pretends to be **three
IntelliThings sensor nodes**: kitchen, bedroom and living room. It publishes on exactly the
same MQTT topics and JSON schema as the real nodes, so the AI team can build and test the
whole **MQTT → Home Assistant → cloud agent → display** pipeline before the real nodes exist.
When real nodes arrive, nothing in HA or the agent changes except which devices are enabled.

You drive it from a **web control panel**: make a room hot, fill it with smoke, walk someone
in or out, pin CO₂ to an exact value, and read the AI messages each room received.

**Contents**

- [Quick start](#quick-start)
- [User guide: the control panel](#user-guide-the-control-panel)
- [Emulating situations: worked examples](#emulating-situations-worked-examples)
- [Scenario reference](#scenario-reference)
- [Automatic behaviour](#automatic-behaviour-what-happens-when-you-do-nothing)
- [Scripting and automation (HTTP API and MQTT)](#scripting-and-automation)
- [MQTT topics](#mqtt-topics)
- [Home Assistant example](#home-assistant-example)
- [Build, configure and flash (maintainer)](#build-configure-and-flash-maintainer)
- [Troubleshooting](#troubleshooting)

---

## Quick start

1. **Power the board** from any USB-C port or charger. It joins the team Wi-Fi and the MQTT
   broker by itself in about 20 s. Nothing else is needed; it keeps its settings through power
   cycles.
2. **Open the control panel** from a laptop or phone on the same Wi-Fi, using either address:
   - **`http://intellithings-emu.local/`**
   - **`http://192.168.1.8/`** (the board's IP address on the team network)

   Both open the same panel. Use the IP if the name doesn't load: some Android phones and
   networks block `.local` names. If the IP ever stops working (the router hands out
   addresses, so it can change), the current one is printed on the serial monitor at boot and
   listed in the router's client list.
3. **Check the badges at the top right:** `panel connected`, `Wi-Fi … dBm`, `MQTT 3/3 nodes`
   and `clock synced` should all be green.
4. The three rooms are now publishing a full sensor snapshot every **3 seconds** on
   `intellithings/emu-kitchen/sensors`, `…/emu-bedroom/sensors` and `…/emu-living-room/sensors`.
   Build your HA entities and agent against those. Then use the panel to create the situations
   your automation or agent should react to.

> The panel has **no login**. Anyone on the team Wi-Fi can drive the emulator, so tell people
> in Discord before running a long test, so nobody changes the rooms under you.

---

## User guide: the control panel

The panel is one page with five parts, top to bottom. It refreshes itself every 1.5 s, and
you never need to reload it.

### 1. Header badges

| Badge | Green means | If it isn't green |
|---|---|---|
| `panel connected` | Your browser can reach the board | Red `emulator unreachable`: board off, rebooting or on another network |
| `Wi-Fi -35 dBm` | Board is on Wi-Fi (signal strength shown) | Red `Wi-Fi down`: it retries by itself |
| `MQTT 3/3 nodes` | All three rooms are connected to the broker | Amber = some rooms, red = none. Readings keep being generated but aren't sent |
| `clock synced` | Time came from the internet (US Eastern) | Amber until synced; payloads have no `ts` field until then |

### 2. System status

IP address, hostname, Wi-Fi network and signal, how many times Wi-Fi reconnected, the MQTT
broker address, uptime, free memory, local time, publish interval, firmware version and
**Last command**: the result of the last thing anyone did (✔ or ✖ with the reason). If a
button seems to do nothing, look here.

### 3. Room cards

One card per room: **Kitchen** (`emu-kitchen`), **Bedroom** (`emu-bedroom`), **Living Room**
(`emu-living-room`).

```
● Kitchen  emu-kitchen
[scenario] [Cook · 18 min left] [override · until cleared] [2 occupants]
Temperature   24.1 °C     Humidity   58.3 %
VOC Index     301         CO₂        1105 ppm      ← forced values are shown in blue
Light         512.0 lux   Distance   640 mm
PM1.0         38 µg/m³    PM2.5      58 µg/m³
PM10          79 µg/m³    Presence   present
published 1s ago · 412 sent                    [↺ Normal]
```

- **Dot** next to the name: green = this room's MQTT connection is up.
- **Mode badge:** `auto` (normal simulated life), `scenario` (a shortcut is running) or
  `override` (you forced custom values). Extra badges show the running shortcut and its time
  left, automatic events like `auto: cooking`, and the number of people in the room.
- **Readings** are exactly what was last published to MQTT. **Blue** numbers are values you
  forced with *Custom values*.
- **Footer:** `published Ns ago` should stay at 0–3 s. It turns **red** if the room stops
  publishing. `· MQTT offline` means readings are generated but not sent.
- **↺ Normal** on a card returns just that room to automatic behaviour.

### 4. Emulate

This is where you create situations. Always **pick the target first** with the tabs at the top
of this section: **Kitchen**, **Bedroom**, **Living Room** or **All rooms**. The page starts on
**All rooms**, so check the tab before you click anything.

#### Shortcuts (left side)

Twelve one-click situations: 🍳 Cook · 🔥 Burnt food · 😴 Sleep · 🛋 Gathering · 🥵 Hot &
occupied · 🌫 PM2.5 spike · 🧴 VOC spike · 😮‍💨 Stuffy room · 🪟 Open window · 🚶 Enter ·
🚪 Leave · 🌙 Lights off. What each one changes is in the
[scenario reference](#scenario-reference).

- Click a shortcut → it starts on the selected tab's room(s), and a green toast confirms it.
  Shortcuts that are typical for the selected room are **highlighted** (e.g. Cook on Kitchen).
- Each button shows its default length (e.g. `20 min`). To change it, type seconds into
  **Duration (s)** *before* clicking: `120` = 2 minutes, `0` = run until you stop it, empty =
  the default.
- When the time runs out, the room drifts back to normal by itself.
- **↺ Back to normal** stops every shortcut and custom value on the selected tab. Readings
  then drift back gradually, the way a real room airs out.
- **Reset room** does the same *and* snaps the readings straight back to the room's baseline.
  Use it between test runs.
- Starting a new shortcut replaces the one that was running in that room.

#### Custom values (right side)

For when you need **specific numbers**, e.g. "CO₂ exactly 1500 ppm" for a repeatable test.

1. For each reading you want to control, enter a **Min** and **Max** (typing ticks its
   checkbox automatically). **Min = Max** (or fill only one of them) gives a fixed value; a
   range makes the value wander inside it like a live reading.
2. Optionally set **Presence** to *present* or *empty* (*no change* leaves it alone).
3. **Duration (s):** `0` (default) = until you press *Back to normal*; otherwise it expires
   by itself.
4. **Jump instantly:** ticked = the next published reading already has your value; unticked
   = the value moves there smoothly (see the timing table below).
5. Press **Apply**. A red toast means something was rejected: min above max, a value outside
   what the real sensor can report (range shown in the last column), or nothing ticked.

Custom values win over a running shortcut for the readings you set, so you can combine
them: e.g. run *Gathering* but pin temperature at 27 °C.

#### How fast do values change?

Shortcuts, and custom values without *jump instantly*, move readings like a real room would.
The table gives the time until a reading is about 95 % of the way to its target:

| Reading | Time to reach the target |
|---|---|
| Light, distance, presence | seconds |
| VOC Index, PM1.0 / PM2.5 / PM10 | ~3 min |
| CO₂ | ~6 min |
| Temperature, humidity | ~9 min |

So after clicking **Stuffy room**, CO₂ passes 1000 ppm within a couple of minutes and settles
around 1200–1800 ppm. For a quick demo or a repeatable test, use **Custom values + jump
instantly** instead.

### 5. AI messages

- **AI messages received** lists the last 20 messages that arrived on any room's
  `…/display` topic, i.e. what the real node would show on its screen. Each shows the room,
  the message's `ts`, how long ago it arrived, a `retained` badge if the broker re-delivered
  a stored message, and a red `not {text} JSON` badge if the payload wasn't the agreed
  `{"text": "...", "ts": "..."}` format. The same messages print as a banner on the serial
  monitor.
- **Send a test AI message** publishes a message to a room's `display` topic exactly like
  Home Assistant would. Use it to test the feedback path, or your dashboard, without the
  agent. Max 400 characters. **Retained** keeps it on the broker, so it's re-delivered after a
  reconnect (that's how HA publishes the real ones).

---

## Emulating situations: worked examples

Each example gives the panel steps, the same thing as a command (for scripts, see
[Scripting](#scripting-and-automation)), what appears on MQTT, and what a well-behaved
agent should do. They cover the AI team's standard test set (hot room + presence, rising
CO₂, PM2.5 spike, empty room, nothing wrong).

> **Before each run:** select the room's tab → **Reset room**, so earlier tests don't leak in.

### Example 1: Hot room with someone in it

*The agent should cool the room (fan or thermostat) and tell the occupant.*

**Panel:** tab **Living Room** → click **🥵 Hot & occupied**.

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
something like *"It's 29 °C in here. I turned on the fan."* to the living room's display.

**Faster, exact version** (Custom values): Temperature min 29 max 29, Humidity 60/60,
Presence *present*, tick **jump instantly**, Apply.

```json
{"node":"living-room","action":"set","ranges":{"temperature_c":29,"humidity_pct":60},
 "presence":true,"instant":true}
```

### Example 2: CO₂ building up while someone sleeps

*The agent should suggest ventilation, gently. It's night and someone is asleep.*

**Panel:** tab **Bedroom** → click **😴 Sleep**, then **😮‍💨 Stuffy room**. Or pin it:
CO₂ min 1300 max 1600, Presence *present*, Apply (leave *jump instantly* off to watch it rise).

```json
{"node":"bedroom","action":"scenario","name":"stuffy"}
{"node":"bedroom","action":"set","ranges":{"co2_ppm":{"min":1300,"max":1600}},"presence":true}
```

(Starting *Stuffy room* replaces *Sleep*; use the custom-value version if you need the room to
stay dark as well: add Light 0/2.)

**Expected:** CO₂ rises past 1000 ppm within a couple of minutes. The agent should turn on
ventilation or the purifier, or suggest opening a window. It shouldn't turn lights on in a
dark, occupied bedroom at night.

### Example 3: Burnt food in the kitchen (PM2.5 spike)

*The agent should start the air purifier and/or range hood.*

**Panel:** tab **Kitchen** → click **🔥 Burnt food** (10 min).

```json
{"node":"kitchen","action":"scenario","name":"burnt_food"}
```

**Expected on MQTT:** PM2.5 climbs to 150–300 µg/m³ (PM1.0 and PM10 follow), VOC Index to
400–500, CO₂ ~1000–1400, temperature +3 °C, presence `true`. After 10 minutes the PM decays
back gradually, as smoke would. **Expected agent behaviour:** purifier on high, maybe the
range hood, plus a display message. A good follow-up check: does it turn the purifier *off*
again once PM2.5 is back to normal?

For a milder test, use **🌫 PM2.5 spike** (PM2.5 80–150 only, nothing else changes) on any room.

### Example 4: Everyone leaves the room

*The agent should switch off lights and devices in an empty room.*

**Panel:** tab **Living Room** → **🛋 Gathering** (4 people, lights on). Wait a minute, then
**🚪 Leave**.

```json
{"node":"living-room","action":"scenario","name":"gathering"}
{"node":"living-room","action":"scenario","name":"leave"}
```

**Expected on MQTT:** `presence` goes `false`, `distance_mm` returns to ~1100 (nobody near the
unit), lux drops as the lamps go off, and CO₂ decays slowly. **Expected:** lights and fan off
in the living room. It shouldn't act on the other two rooms.

### Example 5: Nothing wrong (the "do nothing" test)

*The agent should NOT act.* This is as important as the others.

**Panel:** tab **All rooms** → **Reset room**. All three rooms return to calm, normal readings
(≈20–23 °C, CO₂ ~450–500 ppm, PM2.5 < 10).

```json
{"node":"all","action":"reset"}
```

**Expected:** no tool calls, or at most a short "all good" if your prompt asks for one. Note
that at meal times the kitchen may start **cooking by itself** (see
[Automatic behaviour](#automatic-behaviour-what-happens-when-you-do-nothing)); the room card
then shows `auto: cooking`. For a guaranteed quiet baseline, pin the values, e.g.
`{"node":"all","action":"set","ranges":{"co2_ppm":480,"pm2_5_ugm3":5,"voc_index":100},"instant":true}`.

### Example 6: Exact, repeatable scenario for comparing models

When you compare models on OpenRouter, every model should see **the same numbers**. Use
custom values with fixed values, **jump instantly**, and a duration that covers the run:

```json
{"node":"kitchen","action":"set","ranges":{"temperature_c":27.5,"humidity_pct":68,
 "co2_ppm":1250,"voc_index":180,"pm2_5_ugm3":12},"presence":true,"instant":true,"duration_s":600}
```

Every snapshot for the next 10 minutes then carries exactly those values. PM1.0 and PM10
follow the pinned PM2.5 (≈ 0.65× and 1.3–1.5×) unless you pin them too; lux and distance
keep living normally.

### Example 7: Test the display path without the agent

**Panel:** *Send a test AI message* → Room *Bedroom* → type *"CO₂ is high, open the window."*
→ tick *retained* → **Send**.

The message appears within a second under **AI messages received** and as a banner on the
serial monitor. This is exactly what happens when HA publishes the agent's answer, so it
verifies topic names, JSON format and the retained flag end to end.

To clear a retained message afterwards, publish an empty retained payload to the topic
(see [Scripting](#scripting-and-automation)).

### Quick map: agent test scenario → emulator action

| Test scenario | Panel (tab → button) | Command |
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

PM1.0 is about 0.65 × PM2.5 and PM10 about 1.3–1.5 × PM2.5, as in real smoke and dust. When
someone is present, `distance_mm` drops to 300–900 mm now and then (someone at the desk);
otherwise it sits at ~1100 mm.

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
- Small sensor noise on every reading, and the three rooms never move in lockstep.

Anything you start from the panel takes priority: **custom values > shortcut > automatic
behaviour**. Starting a shortcut cancels any automatic event in that room, and no new one
starts until the shortcut ends. Custom values don't stop automatic events, but the readings
you pinned stay pinned.

---

## Scripting and automation

Everything the panel does is available as a small JSON command, over **HTTP** or **MQTT**.
Use it for automated test runs and model comparisons.

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
```

Metric keys are the payload field names: `temperature_c`, `humidity_pct`, `voc_index`,
`co2_ppm`, `lux`, `distance_mm`, `pm1_0_ugm3`, `pm2_5_ugm3`, `pm10_ugm3`. Invalid commands are
rejected with a reason (HTTP 400, and `last_cmd` in the state) and never change anything.

### Over HTTP

| Endpoint | Returns |
|---|---|
| `POST /api/cmd` | Run a command; `{"ok":true,"msg":"..."}` or HTTP 400 with the reason |
| `GET /api/state` | Everything the panel shows: status, each room's mode, occupants, last reading |
| `GET /api/messages` | The last 20 AI messages received |
| `GET /api/meta` | Metric list with ranges, scenario list, node IDs and topics |

The examples use `intellithings-emu.local`; `192.168.1.8` works the same everywhere
(e.g. `http://192.168.1.8/api/cmd`).

```bash
# macOS / Linux / Git Bash
curl -X POST http://intellithings-emu.local/api/cmd -H 'Content-Type: application/json' \
  -d '{"node":"kitchen","action":"scenario","name":"burnt_food"}'
```

```powershell
# Windows PowerShell
Invoke-RestMethod -Method Post -Uri http://intellithings-emu.local/api/cmd `
  -ContentType 'application/json' -Body '{"node":"kitchen","action":"scenario","name":"burnt_food"}'
```

A test script in Python (standard library only) that runs the standard scenarios one after
another:

```python
import json, time, urllib.request

EMU = "http://intellithings-emu.local"   # or "http://192.168.1.8"

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
    print(f"--- {name}: now check what the agent did, then press Enter")
    input()
cmd({"node": "all", "action": "clear"})
```

### Over MQTT

Publish the same JSON to **`intellithings/emulator/cmd`**, **not retained**. The result shows
up within ~0.5 s in the retained `intellithings/emulator/state` topic (same JSON as
`GET /api/state`, also refreshed every 3 s).

```bash
B="-h <broker-ip> -u <user> -P <password>"
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

Don't publish commands with `-r` / `retain: true`. A retained command runs once immediately
and is then ignored on every reconnect; clear it with an empty retained payload
(`mosquitto_pub $B -t intellithings/emulator/cmd -r -n`).

### Watching and sending AI messages from a terminal

```bash
mosquitto_sub $B -t 'intellithings/#' -v                      # everything

mosquitto_pub $B -t intellithings/emu-bedroom/display -r -q 1 \
  -m '{"text":"CO2 is elevated. Consider opening the window.","ts":"2026-09-27T21:30:00-04:00"}'

mosquitto_pub $B -t intellithings/emu-bedroom/display -r -n   # clear the retained message
```

---

## MQTT topics

Full contract: [`docs/interfaces/mqtt.md`](../../docs/interfaces/mqtt.md) (draft until Oct 4).
Broker: the Mosquitto add-on on the HA Pi, port 1883.

| Topic | Direction | QoS / retained | Payload |
|---|---|---|---|
| `intellithings/<id>/sensors` | emulator → broker, **every 3 s even if unchanged** | 0 / no | Sensor snapshot JSON (every field, every time; `ts` once the clock has synced) |
| `intellithings/<id>/status` | emulator → broker | 1 / yes | `online`, or `offline` via Last Will ~20 s after the board drops |
| `intellithings/<id>/display` | HA → emulator | 1 / yes | `{"text": "...", "ts": "..."}` |
| `intellithings/emulator/status` | emulator → broker | 1 / yes | `online` / `offline` for the control connection |
| `intellithings/emulator/state` | emulator → broker | 0 / yes | Status JSON |
| `intellithings/emulator/cmd` | you → emulator | not retained | Command JSON |

`<id>` is `emu-kitchen`, `emu-bedroom` or `emu-living-room`; the `emu-` prefix always marks
emulated data. Each room has **its own MQTT connection** (client ID `intellithings-<id>`) and
its own Last Will, so HA sees three independent devices. They share one board, so if it loses
power or Wi-Fi all three go `offline` together. `intellithings/emulator/#` is reserved for the
emulator; real nodes and HA must not depend on it.

---

## Home Assistant example

MQTT YAML for one room and two readings; repeat per room and per field. This is a reference
for the Local AI team, not the project's HA config (that lives in `ai/ha/`).

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
      device: {identifiers: [emu-kitchen], name: "Emulator – Kitchen"}
  binary_sensor:
    - name: "Kitchen presence"
      unique_id: emu_kitchen_presence
      state_topic: intellithings/emu-kitchen/sensors
      value_template: "{{ 'ON' if value_json.presence else 'OFF' }}"
      device_class: occupancy
      availability_topic: intellithings/emu-kitchen/status
      device: {identifiers: [emu-kitchen], name: "Emulator – Kitchen"}
```

Readings arrive every 3 s. Throttle the hand-off automation (thresholds plus a minimum time
between calls) so the LLM isn't called on every snapshot, and consider excluding the
emulator entities from HA's recorder.

---

## Build, configure and flash (maintainer)

You only need this to change the firmware or its Wi-Fi/broker settings. The board keeps
running what was last flashed.

### Hardware

**LuatOS ESP32-C3 Core**: 4 MB flash, single-core RISC-V. No GPIOs are used; just USB-C.
Two variants exist and both work unchanged:

- **Classic**: CH343 USB-serial chip; install the CH343 driver on Windows.
- **Native USB**: shows up as *USB JTAG/serial debug unit*.

### 1. Configure (credentials are never committed)

```bash
cd ai/emulator/esp32_c3_emulator
cp sdkconfig.local.example sdkconfig.local     # gitignored
# edit sdkconfig.local: Wi-Fi SSID/password, broker host/port, broker username/password
rm -f sdkconfig                                # so the new values are picked up
```

Use the **Pi's IP address** as the broker host: `homeassistant.local` doesn't resolve on
every network (it didn't on the team network, where the Pi was `192.168.1.2`). Give the
emulator its own MQTT login (an HA user); don't reuse the add-on's internal `homeassistant`
account. Or configure everything with `idf.py menuconfig` → **IntelliThings Emulator**.

| Setting | Default |
|---|---|
| `CONFIG_EMU_WIFI_SSID` / `CONFIG_EMU_WIFI_PASSWORD` | — (required, 2.4 GHz network) |
| `CONFIG_EMU_MQTT_HOST` / `CONFIG_EMU_MQTT_PORT` | `homeassistant.local` / `1883` |
| `CONFIG_EMU_MQTT_USERNAME` / `CONFIG_EMU_MQTT_PASSWORD` | empty (anonymous) |
| `CONFIG_EMU_PUBLISH_INTERVAL_MS` | `3000` |
| `CONFIG_EMU_SIM_TICK_MS` | `1000` |
| `CONFIG_EMU_SIM_SPEED` | `1` (raise to speed up CO₂ build-up, decay and occupancy) |
| `CONFIG_EMU_NODE_ID_KITCHEN` / `_BEDROOM` / `_LIVING_ROOM` | `emu-kitchen` / `emu-bedroom` / `emu-living-room` |
| `CONFIG_EMU_TIMEZONE` | `EST5EDT,M3.2.0,M11.1.0` |
| `CONFIG_EMU_HOSTNAME` | `intellithings-emu` |
| `CONFIG_EMU_LOG_READINGS` | on (one serial line per room per publish) |

### 2. Build, flash, monitor

Requirements: **ESP-IDF v6.1** and internet access on the first build (MQTT, cJSON and mDNS
come from the ESP component registry).

```bash
. $HOME/esp/esp-idf/export.sh        # Windows: run the ESP-IDF v6.1 PowerShell profile
idf.py set-target esp32c3            # first time only
idf.py build
idf.py -p COM8 flash monitor         # macOS/Linux: -p /dev/cu.usbmodem… or /dev/ttyUSB0
```

Exit the monitor with `Ctrl+]`.

### Expected serial output

```
I (420) emulator: IntelliThings emulator booting: 3 virtual nodes, publish every 3000 ms
I (430) nodes: Kitchen      -> emu-kitchen
I (430) nodes: Bedroom      -> emu-bedroom
I (430) nodes: Living Room  -> emu-living-room
I (570) wifi: Connecting to "team-wifi"...
I (18841) wifi: Connected. IP 192.168.1.8  gateway 192.168.1.1  RSSI -30 dBm
I (18841) time: SNTP started (pool.ntp.org)
I (18851) web: Control panel running on port 80
I (18851) emulator: Control panel: http://192.168.1.8/  (or http://intellithings-emu.local/)
I (18891) time: Clock synced: 2026-09-27T15:29:53-04:00
I (18901) mqtt: emu-bedroom: connected; published online, subscribing to intellithings/emu-bedroom/display
I (18931) ctrl: emulator: connected; listening on intellithings/emulator/cmd
I (18931) mqtt: emu-kitchen: connected; published online, subscribing to intellithings/emu-kitchen/display
I (18941) mqtt: emu-living-room: connected; published online, subscribing to intellithings/emu-living-room/display
I (19571) node: emu-kitchen      T=22.2C RH=44.5% CO2=468 VOC=107 PM1/2.5/10=4/6/9 lux=378 dist=627mm PRESENT [auto]
I (19581) node: emu-bedroom      T=20.7C RH=46.9% CO2=458 VOC=90 PM1/2.5/10=2/2/4 lux=76 dist=1144mm empty [auto]
I (19591) node: emu-living-room  T=22.6C RH=42.0% CO2=458 VOC=99 PM1/2.5/10=3/6/6 lux=220 dist=1049mm empty [auto]
```

Each reading line ends with the room's mode: `[auto]`, `[auto:cooking]`, `[scenario:stuffy]`
or `[override]`. `(not sent: MQTT offline)` means the broker isn't connected. Commands are
logged as `ctrl: [web] …` or `ctrl: [mqtt] …`, and AI messages print as a banner:

```
==================================================
[AI MESSAGE] KITCHEN  (emu-kitchen)  [retained]
Remember to turn on the range hood while cooking.
ts: 2026-09-27T18:10:00-04:00
==================================================
```

`[retained]` means the broker re-delivered a stored message after a (re)connect.

---

## Troubleshooting

| Symptom | Check |
|---|---|
| Panel doesn't load at `intellithings-emu.local` | Use `http://192.168.1.8/` instead; some phones/networks block `.local` names. If the IP changed, the current one is on the serial log and in the router's client list |
| Panel shows **emulator unreachable** | Board unplugged, rebooting (~20 s) or your device is on another network |
| A button does nothing | Look at **Last command** in System status, or the toast; the reason for a rejection is shown there |
| Readings don't change after a shortcut | They move gradually (CO₂ ~6 min, temperature ~9 min). Use Custom values + *jump instantly* for immediate values |
| Kitchen changes by itself | Automatic meal-time cooking (`auto: cooking`). Press **Reset room**, or pin values with Custom values |
| Room card says **MQTT offline** / badge `MQTT 0/3` | Broker down or credentials wrong; readings resume on reconnect |
| `transport error … is mqtt://… reachable?` on serial | Broker host/port; `homeassistant.local` needs mDNS — use the Pi's IP |
| `broker refused connection (code 5)` | Wrong MQTT username/password |
| `No Wi-Fi SSID configured` | `sdkconfig.local` missing, or `sdkconfig` existed before you created it — delete `sdkconfig` and rebuild |
| `Association refused temporarily … reason 208` right after a reset | Normal with this router (PMF): it makes a just-rebooted board wait ~15 s; it retries and joins by itself |
| Wi-Fi keeps retrying (`reason 201/15/…`) | 2.4 GHz network? SSID/password correct? 15/204 = wrong password, 201 = SSID not found |
| AI message not shown | Topic must be exactly `intellithings/<id>/display`; payload should be `{"text":"..."}` (other payloads show with a red `not {text} JSON` badge) |
| AI message marked "payload over 512 bytes; truncated" | Messages are cut at 512 bytes; keep display messages short (the length limit is part of the Oct 18 AI-message contract) |
| A command re-runs after reconnect | It won't — retained commands are ignored on reconnect — but don't publish commands with `-r` anyway |
| No serial output (classic board) | CH343 driver; correct COM port |

---

## Mapping to the real nodes

| Emulator | Real node (ESP32-C6, Rust) |
|---|---|
| Three node IDs `emu-*` on one board | One node ID per board, derived from the MAC |
| Four MQTT connections (3 rooms + control) | One connection per node |
| Simulated readings every 3 s | Real sensor readings (interval to be agreed) |
| AI message shown in the panel and on serial | AI message shown in the DWIN display's AI section |
| `intellithings/emulator/#` control topics | Not used |

Topics, JSON fields, units, QoS, retained flags and Last Will behaviour are identical, so HA
and the agent don't care which one they're talking to.

## Source layout

```
esp32_c3_emulator/
├── CMakeLists.txt            picks up sdkconfig.local if present
├── sdkconfig.defaults        target, flash, partitions, lwIP tuning (no secrets)
├── sdkconfig.local.example   credentials template
└── main/
    ├── Kconfig.projbuild     "IntelliThings Emulator" menu
    ├── idf_component.yml     espressif/mqtt, cjson, mdns
    ├── main.c                boot, sim task, 3 s publish cycle
    ├── sim_model.[ch]        room profiles and environment model (plain C)
    ├── scenarios.c           shortcut scenario presets
    ├── payload.[ch]          topic and sensor-JSON formatting (plain C)
    ├── virtual_node.[ch]     node table and lock
    ├── wifi_manager.[ch]     station mode, reconnect with backoff
    ├── time_sync.[ch]        SNTP, timezone, ts
    ├── mqtt_node.[ch]        one MQTT connection per node, Last Will, display subscription
    ├── emulator_ctrl.[ch]    commands, state JSON, control MQTT connection
    ├── ai_message.[ch]       serial banner and message history
    ├── web_server.[ch]       control panel + HTTP API
    └── web/index.html        the panel (embedded in the firmware)
```
