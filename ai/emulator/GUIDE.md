# IntelliThings emulator: user guide

*Firmware v2.1.0 (in development). This guide is built into the board: open it with **?** in
the panel, or download it from `/guide.md?download=1`. Full reference: `ai/emulator/README.md`
in the repo.*

The emulator is one ESP32-C3 board that pretends to be **three sensor nodes**: `emu-kitchen`,
`emu-bedroom` and `emu-living-room`. They publish on the real node topics with the real JSON
schema, so the AI team can test **MQTT → Home Assistant → agent → display** without real nodes.

## Quick start

1. Power the board from USB-C. It joins the Wi-Fi and the broker in about 20 s.
2. Open **`http://192.168.1.4/`** or `http://intellithings-emu.local/` on the same Wi-Fi.
3. Check the badges at the top: **panel**, **Wi-Fi**, **MQTT 3/3** and **clock** should all be
   green.
4. Each room publishes a full snapshot every **3 s** on `intellithings/emu-<room>/sensors`.

The panel has **no login**. Anyone on the Wi-Fi can drive it, so announce long tests in Discord.

## The panel

The panel has two tabs at the top:
- **Dev:** the three emulated rooms. This is the everyday tab.
- **Demo (experimental):** the demo-mode stand-in. It arrives in a later update.

Below the tabs are the **System log** and **MQTT route** cards. Every card has **hide** to fold
it away, and some parts sit behind **more**. The panel remembers what you hid, per browser.

### Header and badges

| Badge | Green means | Otherwise |
|---|---|---|
| panel | Your browser reaches the board | Red: board off, rebooting, or you're on another network |
| Wi-Fi | Board is on Wi-Fi (signal in dBm) | Red: it retries by itself |
| MQTT n/3 | All three rooms are connected to the broker | Amber: some rooms; red: none. Readings are still generated, just not sent |
| clock | Time came from the internet | Amber until synced; payloads have no `ts` until then |

- **Click any badge** to open **System status**: IP, broker, uptime, free memory, firmware
  version and the last command.
- **?** opens this guide. **⚙** opens Settings.

### Rooms

One card per room. Each tile shows a reading as the board last published it:
- **the current value.** Temperature shows **°C and °F side by side**; MQTT always carries °C.
- **the previous value** and the change (▲ / ▼).
- **a sparkline** of the last 10 minutes.
- **Blue values** are pinned with *Custom values*.

The badges under the room name show:
- the **mode**: `auto`, `scenario` or `override`;
- the running shortcut and its time left;
- automatic events such as `auto: cooking`;
- the number of people in the room.

The footer says how long ago the room published; it turns red if publishing stops. **↺ Normal**
returns just that room to automatic behaviour.

### Plots and last values

**Click a tile** to open its plot.

- **Window:** 10 min, 30 min or 1 h. The last 10 min has every snapshot; longer windows use
  15-second averages.
- **Chart style:** line, area, step, bars, min–max band, or a table. Pick the default in
  Settings.
- **Hover** (or tap) the chart to read a value.
- **Last values:** a table of the last 20 published values, with the time and the change.
- **History is kept on the board** (it survives reloading the page), but a reboot clears it.

### Emulate

Pick the target first: **Kitchen**, **Bedroom**, **Living Room** or **All rooms**.

- **Shortcuts:**
  - **Start one:** click it. Shortcuts typical for the chosen room are highlighted.
  - **Duration (s):** set it *before* clicking. `0` runs until stopped; empty uses the default.
  - **↺ Back to normal** stops shortcuts and custom values. Readings then drift back.
  - **Reset room** does the same and also snaps the readings back to baseline.
- **Custom values** (under **more**): enter a **Min** and **Max** per reading. Min = Max gives a
  fixed value; a range wanders like a live reading.
  - Optional: presence and a duration.
  - **Jump instantly** makes the next snapshot carry your value.
  - Temperature is entered in **°C**.

Shortcuts and slow custom values move like a real room:
- light, distance and presence change within seconds;
- VOC and PM in ~3 min;
- CO₂ in ~6 min;
- temperature and humidity in ~9 min.

For an exact, immediate value, use **jump instantly**.

### AI messages

Three boxes, one per room, styled like the AI section of the node's display.

- **What each box shows:** the latest message on that room's `…/display` topic, with:
  - its `ts` and how long ago it arrived;
  - **retained** if the broker re-delivered a stored message;
  - a **character count**, red when over the 60-character limit for room messages.
- **Problem badges:** **not {text} JSON** or **truncated**.
- **Agent status:** while the agent is running (`intellithings/ai/inference_status` says
  `start thinking`), every box shows **thinking…**. The agent's overall summary
  (`intellithings/ai/summary`) is shown under the boxes.
- **Send a test message:** type it under a box and press **Send**. It's published exactly as
  Home Assistant would. Tick **retained** to keep it on the broker.
- **History:** **more** shows the last 20 messages from all rooms.

### System log

- **Commands:** every command and settings change, from the panel or from MQTT. Each shows ✔ or
  ✖ with the reason. When the card is hidden, the last command still shows in its title.
- **System:** the board's own log lines (Wi-Fi, MQTT, AI messages, errors). Warnings are amber,
  errors red. The per-snapshot reading lines are left out.
- **Controls:** filter by text, **pause** scrolling, **clear** the view (the board keeps its
  copy), and **download** what's shown as a text file.

### MQTT route

A live diagram of who publishes and who receives:
- **left:** the board (the three rooms, its control connection, this panel);
- **middle:** the broker;
- **right:** Home Assistant and the AI agent.

How to read it:
- **Moving dots** show messages the board sent or received.
- **Each line** is labelled with its last topic and how long ago it was used.
- **Dashed lines are expected but not visible to the board.** The broker doesn't say who
  subscribes, so broker → Home Assistant is always dashed.
- **The agent box lights up** while it's thinking. Its messages reach the board through the
  broker.
- **Click a line** to list its last messages.

### Settings

Open with **⚙**.

- **MQTT broker:**
  - **Fields:** host (the Raspberry Pi's IP), port, username, password, keepalive and the
    sensor publish interval. The password is never shown; leave it empty to keep it.
  - **Apply & reconnect:** saves on the board and reconnects all four MQTT connections in a few
    seconds, without a reboot. The old broker is told the rooms went `offline`.
  - **Clearing the username or password** makes the board reboot to apply it.
  - **Restore defaults:** goes back to the values the firmware was built with.
- **Display:** temperature as °C, °F or both; the default plot window and chart style; how often
  the panel refreshes; light or dark theme. These are stored in your browser only.
- **Layout:** show or hide whole cards, or reset the layout.

**Typing a wrong IP doesn't lock you out.** The panel is plain HTTP and keeps working; only the
MQTT badge goes red. Fix the IP or restore the defaults.

## Emulating situations from scripts

Everything the panel does is a small JSON command. Send it to `POST /api/cmd` or to the MQTT
topic `intellithings/emulator/cmd` (never retained):

```jsonc
{"node":"kitchen","action":"scenario","name":"burnt_food"}            // a shortcut
{"node":"bedroom","action":"set","ranges":{"co2_ppm":1500},"instant":true}  // pin a value
{"node":"all","action":"clear"}                                      // back to automatic
{"node":"all","action":"reset"}                                      // also reset readings
{"node":"emu-kitchen","action":"display","text":"Test","retain":true} // test AI message
```

Shortcuts: `cooking`, `burnt_food`, `sleep`, `gathering`, `hot`, `pm_spike`, `voc_spike`,
`stuffy`, `window_open`, `enter`, `leave`, `lights_off`. Metric keys: `temperature_c`,
`humidity_pct`, `voc_index`, `co2_ppm`, `lux`, `distance_mm`, `pm1_0_ugm3`, `pm2_5_ugm3`,
`pm10_ugm3`.

## Settings over HTTP

- **Read:** `GET /api/config`. Returns the current values, `password_set`, `saved` (whether
  anything differs from the defaults) and `defaults`.
- **Change:** `POST /api/config` with only the fields to change:

  ```json
  {"host":"192.168.1.2","port":1883,"username":"emulator","password":"…",
   "keepalive_s":15,"publish_interval_ms":3000}
  ```
- **Back to defaults:** `POST /api/config/reset`.

Limits: host up to 63 characters (letters, digits, `.`, `-`, `_`), port 1–65535, username up to
32 characters, password up to 64, keepalive 5–120 s, publish interval 1000–60000 ms.

## Logs, history and AI topics on the board

All of this is kept in RAM, so a reboot clears it:
- **System log:** the last 80 lines.
- **Commands:** the last 32.
- **MQTT messages** the board sent or received: the last 64. Kinds: `sensors`, `status`,
  `state`, `display_out`, `display_in`, `cmd_mqtt`, `cmd_http`, `ai_in`.
- **AI messages:** the last 20, plus the latest per room and the latest summary and thinking
  state.
- **Reading history** in two tiers:

  | Tier | Resolution | Length |
  |---|---|---|
  | `10m` | every snapshot (3 s) | 200 snapshots, about 10 min |
  | `1h` | 15 s averages (presence: "present at any point") | 240 buckets, 1 h |

  A longer publish interval stretches the `10m` tier over more time.

Room messages from the pipeline are at most 60 characters. The board accepts up to 400
characters of text in a 512-byte payload.

## HTTP API

| Endpoint | Returns |
|---|---|
| `GET /` | The control panel |
| `GET /guide.md` | This guide (`?download=1` saves it as a file) |
| `GET /api/state` | Status, each room's mode and last reading (v0.1.0 fields, plus `config_saved` and `mqtt.reconfiguring`) |
| `GET /api/meta` | Metrics, scenarios, nodes, topics, the history layout and the firmware version |
| `GET /api/messages` | The last 20 AI messages, newest first |
| `POST /api/cmd` | Run a command: `{"ok":true,"msg":"…"}` or HTTP 400 with the reason |
| `GET /api/poll?log=&cmd=&ev=&msg=` | Everything the panel needs in one call (below) |
| `GET /api/history?tier=10m\|1h` | Reading history, binary (below) |
| `GET /api/config`, `POST /api/config`, `POST /api/config/reset` | Settings |

### `/api/poll`

Pass the `next` values from your previous poll. Start with `0`, which returns everything still in
memory:

```jsonc
{"state":{…same as /api/state…},
 "log":{"items":[{"seq":12,"t_ms":20512,"text":"I (20512) mqtt: …"}],"next":12,"gap":false},
 "cmds":{"items":[{"seq":3,"t_ms":…,"ok":true,"source":"web","msg":"…"}],"next":3,"gap":false},
 "routes":{"items":[{"seq":40,"t_ms":…,"kind":"sensors","bytes":231,"topic":"…"}],"next":40,"gap":false},
 "messages":{"items":[{"seq":1,"node_id":"emu-kitchen","text":"…",…}],"next":1,"gap":false},
 "ai":{"latest":[{…}|null, …],"summary":{"text":"…","ts":"…","age_s":5,"retained":true,"raw":false}|null,
       "inference":{"state":"stop thinking",…}|null}}
```

- **`t_ms`** is milliseconds since boot.
- **`gap: true`** means some entries were overwritten before you fetched them.
- **After a reboot** the numbers start again from 1. The board notices numbers it hasn't reached
  yet and starts over.

### `/api/history` binary layout (little-endian)

| Offset | Type | Field |
|---|---|---|
| 0 | 4 × char | `EMUH` |
| 4 | u8 | format version (1) |
| 5 | u8 | tier (0 = `10m`, 1 = `1h`) |
| 6 | u8 | nodes N (3, in `/api/meta` order) |
| 7 | u8 | series S (10: the nine metrics in `/api/meta` order, then presence 0/1) |
| 8 | u16 | sample count C |
| 10 | u16 | nominal interval, seconds |
| 12 | u32 | board uptime now, seconds |
| 16 | u32 | Unix time now (0 if the clock isn't synced) |
| 20 | u32 | reserved (0) |
| 24 | S × u16 | divisor per series (value = stored ÷ divisor; 10 for temperature and humidity) |
| … | C × u32 | sample times, seconds since boot, oldest first |
| … | N × S × C × u16 | values: node by node, series by series, oldest first |

## How to test this portal

Work through these after flashing a new build.

**In the panel:**
1. All four badges are green (MQTT needs the broker up). Click one: System status shows the
   expected firmware version.
2. Each room's footer says *published 0–3 s ago*, and the values change every few seconds.
3. Click a temperature tile. The plot opens with a few minutes of history. Switch to 1 h and
   try each chart style. *Last values* lists recent readings.
4. Choose **Kitchen**, click **🔥 Burnt food**. A green toast appears, the card shows
   `scenario`, and the **System log → Commands** tab shows the command with ✔.
5. **more → Custom values:** CO₂ min 1500 max 1500, tick *jump instantly*, Apply. Within 3 s the
   CO₂ tile shows 1500 in blue.
6. Enter CO₂ 6000 and Apply. You get a red toast with the allowed range, and nothing changes.
7. Under the Bedroom AI box, type a message and **Send**. It appears in that box within a second.
   The MQTT route shows a dot from the control connection to the broker and back to the rooms.
8. **⚙ → Display:** switch temperature to °F only; the tiles follow. Hide a card and reload; it
   stays hidden. **Layout → Reset** brings it back.
9. **⚙ → MQTT:** change the publish interval to 5000 ms and Apply; the room footers count up to
   5 s. Set it back to 3000.
10. Shrink the window to phone width: the cards stack and nothing scrolls sideways.

**From a script:** `ai/emulator/tests/api_check.py` checks the whole HTTP API
(`python api_check.py --host 192.168.1.4`; add `--broker-test` to also switch brokers and back).

## Falling back to v0.1.0

If a new build misbehaves, flash the frozen v0.1.0 image. Steps are in
`ai/emulator/releases/v0.1.0/README.md`.
