# Emulator ESP32

Owner: @spicybutter (PM) · AI subteam · status and test plan: [`TODO.md`](TODO.md)

One **LuatOS ESP32-C3 Core** board with no sensors attached that pretends to be **three
IntelliThings sensor nodes** — kitchen, bedroom and living room. It publishes on exactly the
same MQTT topics and JSON schema as the real nodes, so the AI team can build and test the
whole **MQTT → Home Assistant → cloud agent → display** pipeline before the real PCBs exist.
When real nodes arrive, nothing in HA or the agent has to change except which devices are
enabled.

It also serves a **control panel** at `http://intellithings-emu.local/` where the AI team can
trigger scenarios (cooking, stuffy room, PM2.5 spike…), force any value into a range, watch
system status and see every AI message the nodes receive.

## What it emulates

| Node ID | Room | Personality |
|---|---|---|
| `emu-kitchen` | Kitchen | Busy at meal times; cooking raises temperature, humidity, VOC, PM2.5 and CO₂; lights on while cooking |
| `emu-bedroom` | Bedroom | Calm and stable; occupied and dark at night, with CO₂ building slowly while someone sleeps; low PM/VOC |
| `emu-living-room` | Living room | Variable occupancy (up to 4 people) in the evening; changing light; occasional candle/cleaning VOC and dust PM events |

Each node reports the full real-node sensor set:

| Sensor | Fields |
|---|---|
| SHT41 | `temperature_c`, `humidity_pct` |
| SGP40 | `voc_index` (1–500, ~100 is normal) |
| SCD41 | `co2_ppm` |
| BH1750 | `lux` |
| VL53L0X | `distance_mm` (30–1200) |
| PMS5003 | `pm1_0_ugm3`, `pm2_5_ugm3`, `pm10_ugm3` |
| LD2410C | `presence` |

### How the simulation behaves

Values are **not** random each cycle. Every room keeps its own state, updated every second:

- **Smooth drift** toward room- and time-of-day targets, plus small sensor noise on each
  reading. Temperature and humidity move over minutes; light changes within seconds.
- **Occupancy** — people enter and leave with realistic dwell times (5–90 min, a few hours
  for sleep), weighted by the hour. Presence stays on while someone sits still or sleeps,
  like the LD2410C.
- **CO₂** rises with the number of occupants and decays toward outdoor air (~420 ppm) through
  ventilation.
- **PM** rises during events and decays gradually afterwards. PM1.0 ≈ 0.65 × PM2.5 and PM10 ≈
  1.3–1.5 × PM2.5.
- **Light** follows daylight for the local hour plus lamps when someone is there (bedroom
  lamps go off at night).
- **Distance** sits at the background (~1.1 m) and drops to 0.3–0.9 m when someone is near
  the unit.
- **Automatic events** — cooking at meal times in the kitchen, candles/cleaning in the living
  room, dust from activity.
- Each room has its own random seed, so the three nodes never move in lockstep.

Time of day comes from NTP (US Eastern by default); before the clock syncs it assumes noon.

## Hardware

**LuatOS ESP32-C3 Core** — 4 MB flash, single-core RISC-V. Two variants exist; both work with
this firmware unchanged, because ESP-IDF sends logs to UART0 *and* native USB by default:

- **Classic** — CH343 USB-serial chip. Install the CH343 driver on Windows.
- **Native USB** — no bridge chip; shows up as *USB JTAG/serial debug unit*. GPIO18/19 are
  taken by USB.

No GPIOs are used. Just plug in USB-C.

## Build

Requirements: **ESP-IDF v6.1** (same version as the node firmware) and internet access on
the first build — MQTT, cJSON and mDNS come from the ESP component registry.

### 1. Configure credentials (never committed)

```bash
cd ai/emulator/esp32_c3_emulator
cp sdkconfig.local.example sdkconfig.local     # gitignored
# edit sdkconfig.local: Wi-Fi SSID/password, broker host/port, broker username/password
rm -f sdkconfig                                # so the new values are picked up
```

Or use `idf.py menuconfig` → **IntelliThings Emulator**. Both `sdkconfig` and
`sdkconfig.local` are gitignored.

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
| `CONFIG_EMU_LOG_READINGS` | on (one serial line per node per publish) |

The HA Mosquitto add-on usually requires a login: create an HA user (or an add-on login)
for the emulator.

### 2. Build, flash, monitor

```bash
. $HOME/esp/esp-idf/export.sh        # Windows: run the ESP-IDF v6.1 PowerShell profile
idf.py set-target esp32c3            # first time only
idf.py build
idf.py -p COM8 flash monitor         # macOS/Linux: -p /dev/cu.usbmodem… or /dev/ttyUSB0
```

Exit the monitor with `Ctrl+]`.

## Expected serial output

```
I (292) emulator: IntelliThings emulator booting: 3 virtual nodes, publish every 3000 ms
I (302) nodes: Kitchen      -> emu-kitchen
I (302) nodes: Bedroom      -> emu-bedroom
I (302) nodes: Living Room  -> emu-living-room
I (412) wifi: Connecting to "home-wifi"...
I (2890) wifi: Connected. IP 192.168.1.42  gateway 192.168.1.1  RSSI -52 dBm
I (2900) emulator: Control panel: http://192.168.1.42/  (or http://intellithings-emu.local/)
I (3010) mqtt: emu-kitchen: connected; published online, subscribing to intellithings/emu-kitchen/display
I (3060) mqtt: emu-kitchen: subscribed to intellithings/emu-kitchen/display
I (3040) ctrl: emulator: connected; listening on intellithings/emulator/cmd
I (4312) emulator: Example payload on intellithings/emu-kitchen/sensors:
{"node_id":"emu-kitchen","temperature_c":22.2,"humidity_pct":44.5,"voc_index":107,"co2_ppm":468,"lux":378.5,"distance_mm":627,"pm1_0_ugm3":4,"pm2_5_ugm3":6,"pm10_ugm3":9,"presence":true,"ts":"2026-09-27T12:05:04-04:00"}
I (4322) node: emu-kitchen      T=22.2C RH=44.5% CO2=468 VOC=107 PM1/2.5/10=4/6/9 lux=378 dist=627mm PRESENT [auto]
I (4352) node: emu-bedroom      T=20.7C RH=46.9% CO2=458 VOC=90 PM1/2.5/10=2/2/4 lux=76 dist=1144mm empty [auto]
I (4392) node: emu-living-room  T=22.6C RH=42.0% CO2=458 VOC=99 PM1/2.5/10=3/6/6 lux=220 dist=1049mm empty [auto]
```

Each line ends with the node's mode: `[auto]`, `[auto:cooking]` for an automatic event,
`[scenario:stuffy]` or `[override]`. `(not sent: MQTT offline)` means the reading was
generated but the broker isn't connected.

An AI message arriving on a node's `display` topic prints as a banner:

```
==================================================
[AI MESSAGE] KITCHEN  (emu-kitchen)
Remember to turn on the range hood while cooking.
ts: 2026-09-27T18:10:00-04:00
==================================================
```

`[retained]` after the node ID means the broker re-delivered the last retained message
after a (re)connect.

## Control panel

Open `http://intellithings-emu.local/` (or the IP printed on serial) from any device on the
same network. It shows:

- **System status** — IP, hostname, Wi-Fi SSID and signal, broker, MQTT state, uptime, free
  memory, clock, firmware version and the result of the last command.
- **Rooms** — live readings per node, MQTT state, mode (auto / scenario / override) with time
  left, occupants and "published N s ago" (red if a node stops publishing). Forced values are
  highlighted.
- **Emulate** — pick a room (or *All rooms*), then:
  - **Shortcuts**: 🍳 Cook · 🔥 Burnt food · 😴 Sleep · 🛋 Gathering · 🥵 Hot & occupied ·
    🌫 PM2.5 spike · 🧴 VOC spike · 😮‍💨 Stuffy room · 🪟 Open window · 🚶 Enter · 🚪 Leave ·
    🌙 Lights off. Optional duration override. **↺ Back to normal** clears everything;
    **Reset room** also resets readings to the room's baseline.
  - **Custom values**: tick any metrics, enter a min/max (min = max for a fixed value),
    presence, duration (0 = until cleared) and optionally *jump instantly*. Values move
    smoothly into the range, then wander inside it like live readings.
- **AI messages received** — everything that arrived on the three `display` topics.
- **Send a test AI message** — publishes `{"text","ts"}` to a room's `display` topic, like
  HA would, to test the display path without the agent.

Precedence per value: **custom range > scenario > automatic behaviour**. When a scenario or
override ends, values drift back to normal (PM and CO₂ decay gradually).

## MQTT

Full contract: [`docs/interfaces/mqtt.md`](../../docs/interfaces/mqtt.md) (draft until Oct 4).

### Node topics (same as real nodes)

| Topic | Direction | QoS / retained |
|---|---|---|
| `intellithings/<id>/sensors` | emulator → broker, **every 3 s even if unchanged** | 0 / no |
| `intellithings/<id>/status` | emulator → broker, `online` / `offline` (Last Will) | 1 / yes |
| `intellithings/<id>/display` | HA → emulator, `{"text","ts"}` | 1 / yes |

Each virtual node has **its own MQTT connection** (client ID `intellithings-<id>`) with its
own Last Will, so HA sees three independent devices. They share one Wi-Fi link, so if the
board loses power or Wi-Fi, all three go `offline` together.

### Emulator control topics

A fourth connection (`intellithings-emulator`) carries the control channel, kept apart from
the node topics:

| Topic | Direction | Payload |
|---|---|---|
| `intellithings/emulator/status` | emulator → broker (retained, Last Will) | `online` / `offline` |
| `intellithings/emulator/state` | emulator → broker (retained) | status JSON, every 3 s and right after a command — same as `GET /api/state` |
| `intellithings/emulator/cmd` | you → emulator (**not retained**) | command JSON — same as `POST /api/cmd` |

### Commands

`node` is a node ID, a room name (`kitchen`, `bedroom`, `living-room`) or `"all"`.

```jsonc
// Start a scenario (duration_s optional: default per scenario, 0 = until cleared)
{"node":"emu-kitchen","action":"scenario","name":"cooking","duration_s":1200}

// Force values into ranges (a single number = fixed value), plus presence
{"node":"emu-bedroom","action":"set","ranges":{"co2_ppm":{"min":1300,"max":1500},"temperature_c":26},
 "presence":true,"duration_s":600,"instant":false}

// Back to automatic behaviour / also reset readings to baseline
{"node":"all","action":"clear"}
{"node":"emu-kitchen","action":"reset"}

// Publish a test AI message to a node's display topic
{"node":"emu-kitchen","action":"display","text":"Turn on the range hood.","retain":false}
```

Scenario names: `cooking`, `burnt_food`, `sleep`, `gathering`, `hot`, `pm_spike`,
`voc_spike`, `stuffy`, `window_open`, `enter`, `leave`, `lights_off`. Metric keys are the
payload field names. Invalid commands are rejected with a reason in the state's `last_cmd`
(and in the HTTP response); they never crash the emulator.

### Testing from a terminal

```bash
B="-h homeassistant.local -u <user> -P <password>"

# Watch everything the emulator publishes
mosquitto_sub $B -t 'intellithings/#' -v

# Send an AI message to the bedroom (retained, as HA does) — it prints on serial
mosquitto_pub $B -t intellithings/emu-bedroom/display -r -q 1 \
  -m '{"text":"CO2 is elevated. Consider opening the window.","ts":"2026-09-27T21:30:00-04:00"}'

# Clear that retained message
mosquitto_pub $B -t intellithings/emu-bedroom/display -r -n

# Trigger cooking in the kitchen
mosquitto_pub $B -t intellithings/emulator/cmd -q 1 \
  -m '{"node":"emu-kitchen","action":"scenario","name":"cooking"}'

# Same through the HTTP API
curl -X POST http://intellithings-emu.local/api/cmd -H 'Content-Type: application/json' \
  -d '{"node":"emu-kitchen","action":"scenario","name":"cooking"}'
```

HTTP API: `GET /api/meta` (metrics, scenarios, nodes, topics) · `GET /api/state` ·
`GET /api/messages` · `POST /api/cmd`.

## Home Assistant example

MQTT YAML for one node (repeat per node). This is a reference for the Local AI team, not the
project's HA config:

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

Readings arrive every 3 s; consider excluding the emulator entities from the recorder, or
throttling automations, so the LLM isn't called on every snapshot.

## Troubleshooting

| Symptom | Check |
|---|---|
| `No Wi-Fi SSID configured` | `sdkconfig.local` missing, or `sdkconfig` existed before you created it — delete `sdkconfig` and rebuild |
| Wi-Fi keeps retrying (`reason 201/15/…`) | 2.4 GHz network? SSID/password correct? Reason 15/204 = wrong password, 201 = SSID not found |
| `transport error … is mqtt://… reachable?` | Broker host/port; `homeassistant.local` needs mDNS on the network — use the Pi's IP if it doesn't resolve |
| `broker refused connection (code 5)` | Wrong MQTT username/password |
| Panel doesn't load at `.local` | Use the IP from the serial log (some networks/Android block mDNS) |
| Panel says "emulator unreachable" | Board rebooted or lost Wi-Fi — check serial |
| Nodes show "MQTT offline", readings not published | Broker down or credentials wrong; the simulation keeps running and publishing resumes on reconnect |
| No serial output (classic board) | CH343 driver; correct COM port |
| AI message not printed | Topic must be exactly `intellithings/<id>/display`; payload should be `{"text":"..."}` (other payloads print raw with a note) |
| A command keeps re-running after reconnect | Don't publish to `emulator/cmd` with `-r` — retained commands are ignored and logged |

## Mapping to the real nodes

| Emulator | Real node (ESP32-C6, Rust) |
|---|---|
| Three node IDs `emu-*` on one board | One node ID per board, derived from the MAC |
| Four MQTT connections (3 nodes + control) | One connection per node |
| Simulated readings every 3 s | Real sensor readings (interval to be agreed) |
| AI message printed on serial and the panel | AI message shown in the DWIN display's AI section |
| `intellithings/emulator/#` control topics | Not used |

Topics, JSON fields, units, QoS, retained flags and Last Will behaviour are identical, so HA
and the agent don't care which one they're talking to.

## Source layout

```
esp32_c3_emulator/
├── CMakeLists.txt            picks up sdkconfig.local if present
├── sdkconfig.defaults        target, flash, partitions (no secrets)
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
