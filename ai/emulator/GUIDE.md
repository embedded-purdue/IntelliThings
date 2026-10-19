# IntelliThings emulator: user guide

*Firmware v2.1.0 (in development) · this file is built into the board and served at
`/guide.md`. Full reference: `ai/emulator/README.md` in the repo.*

The emulator is one ESP32-C3 board that pretends to be **three sensor nodes**: `emu-kitchen`,
`emu-bedroom` and `emu-living-room`. They publish on the real node topics with the real JSON
schema, so the AI team can test **MQTT → Home Assistant → agent → display** without real nodes.

## Quick start

1. Power the board from USB-C. It joins the Wi-Fi and the broker in about 20 s.
2. Open **`http://192.168.1.4/`** or `http://intellithings-emu.local/` on the same Wi-Fi.
3. Check the badges at the top: panel connected, Wi-Fi, MQTT 3/3 nodes, clock synced.
4. Each room publishes a full snapshot every **3 s** on `intellithings/emu-<room>/sensors`.

The panel has **no login**. Anyone on the Wi-Fi can drive it, so announce long tests in Discord.

## Settings: broker address and publish interval

The broker (the Raspberry Pi) address, port, login, keepalive and the publish interval can be
changed **without reflashing**. They're saved on the board and survive a reboot. The values in
`sdkconfig.local` stay as the defaults.

- **Read:** `GET /api/config`. Returns the current values, `password_set` (the password itself
  is never sent back), `saved` (whether anything differs from the defaults) and `defaults`.
- **Change:** `POST /api/config` with only the fields you want to change:

  ```json
  {"host":"192.168.1.2","port":1883,"username":"emulator","password":"…",
   "keepalive_s":15,"publish_interval_ms":3000}
  ```

  - **Broker change:** all four MQTT connections leave the old broker (they publish `offline`
    first) and reconnect to the new one in a few seconds. No reboot is needed.
  - **Clearing the username or password:** the board saves the setting and **reboots** to apply
    it.
  - **Publish interval:** applies at once.
- **Back to defaults:** `POST /api/config/reset`.
- **Typing a wrong IP doesn't lock you out.** The panel is plain HTTP and keeps working; only the
  MQTT badge goes red. Fix the IP or reset.

Limits: host up to 63 characters (letters, digits, `.`, `-`, `_`), port 1–65535, username up to
32 characters, password up to 64, keepalive 5–120 s, publish interval 1000–60000 ms.

## Emulating situations

Shortcuts and custom values work as in v0.1.0. They're available from the panel, from
`POST /api/cmd` and from the MQTT topic `intellithings/emulator/cmd`:

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

## AI messages

The board listens on each room's `intellithings/<node>/display` topic, as a real node does, and
keeps:
- the **latest message per room**;
- the **last 20 messages**.

It also listens, read-only, to the agent's **overall summary** (`intellithings/ai/summary`) and
**thinking state** (`intellithings/ai/inference_status`). Room messages from the pipeline are at
most 60 characters. The board accepts up to 400 characters of text in a 512-byte payload.

## Logs and MQTT traffic

The board keeps three logs in memory (they're cleared by a reboot):
- **System log:** the last 80 firmware log lines. The per-snapshot reading lines are left out.
- **Command log:** the last 32 commands and settings changes, with source (`web` / `mqtt`),
  result and reason.
- **MQTT traffic:** the last 64 messages the board sent or received. Kinds: `sensors`, `status`,
  `state`, `display_out`, `display_in`, `cmd_mqtt`, `cmd_http`, `ai_in`. The board only sees its
  own traffic, so it can't tell whether HA received a message.

## Reading history

The board remembers what it published, for the panel's plots:

| Tier | Resolution | Length |
|---|---|---|
| `10m` | every snapshot (3 s) | 200 snapshots, about 10 min |
| `1h` | 15 s averages (presence: "present at any point") | 240 buckets, 1 h |

History lives in RAM and is cleared by a reboot. A longer publish interval stretches the `10m`
tier over more time.

## HTTP API

| Endpoint | Returns |
|---|---|
| `GET /` | The control panel |
| `GET /guide.md` | This guide (`?download=1` saves it as a file) |
| `GET /api/state` | Status, each room's mode and last reading (same as v0.1.0, plus `config_saved`) |
| `GET /api/meta` | Metrics, scenarios, nodes, topics, the history layout and the firmware version |
| `GET /api/messages` | The last 20 AI messages, newest first |
| `POST /api/cmd` | Run a command: `{"ok":true,"msg":"…"}` or HTTP 400 with the reason |
| `GET /api/poll?log=&cmd=&ev=&msg=` | Everything the panel needs in one call (below) |
| `GET /api/history?tier=10m\|1h` | Reading history, binary (below) |
| `GET /api/config`, `POST /api/config`, `POST /api/config/reset` | Settings (above) |

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

- **`t_ms`** is milliseconds since boot. Wall time is `state.time.now` minus
  (`state.uptime_s` × 1000 − `t_ms`).
- **`gap: true`** means some entries were overwritten before you fetched them.
- **After a reboot** the sequence numbers start again from 1. The board notices numbers it hasn't
  reached yet and starts over.

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

Work through these after flashing a new build:

1. `GET /api/state`: `fw` is the expected version, `wifi.connected` is true and `time.synced`
   is true.
2. With the broker up, every `nodes[].mqtt` is `connected` and `publish_count` grows by 1 every
   3 s.
3. Send `{"node":"kitchen","action":"scenario","name":"burnt_food","duration_s":60}`. The reply
   is `ok`. `/api/poll` shows it under `cmds` and a `cmd_http` entry under `routes`.
4. Send an invalid command (e.g. `co2_ppm` 5001). You get HTTP 400 with the reason, and nothing
   changes.
5. Send a test AI message to one room. It appears under `messages` and in that room's
   `ai.latest` entry, with `display_out` and `display_in` entries under `routes`.
6. Fetch `/api/history?tier=10m` and check the header: `EMUH`, N = 3, S = 10, C > 0.
7. Change the publish interval to 5000 ms with `POST /api/config`. Snapshots slow down; set it
   back to 3000.
8. Set the broker host to an unused IP. MQTT goes `disconnected` and the panel still answers.
   Set it back: all four connections come back within ~10 s.

`ai/emulator/tests/api_check.py` runs these checks automatically:
`python api_check.py --host 192.168.1.4`.

## Falling back to v0.1.0

If a new build misbehaves, flash the frozen v0.1.0 image. Steps are in
`ai/emulator/releases/v0.1.0/README.md`.
