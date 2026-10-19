# Emulator v2.1.0 — firmware update plan

Owner: @spicybutter (PM) · *Status: phases 0–1 done on branch `ai/emulator-v2` (Oct 18); results in [`TODO.md`](TODO.md#v210-progress). Still open from phase 0: uploading the merged v0.1.0 image to the team Drive* · current firmware: **v0.1.0**
([`README.md`](README.md), [`TODO.md`](TODO.md))

v2.1.0 keeps everything v0.1.0 does for the AI team (three emulated nodes on the real topics, the
same commands and HTTP API) and adds:

- an experimental **demo mode** that stands in for the demo master;
- runtime **MQTT settings**;
- **history plots**, a **system log** and a **live MQTT route diagram**;
- a **cleaner panel** where most sections can be hidden.

**Contents**

1. [Versions and fallback](#1-versions-and-fallback)
2. [Feature list](#2-feature-list)
3. [Dev mode and demo mode](#3-dev-mode-and-demo-mode)
4. [Demo mode in detail](#4-demo-mode-in-detail)
5. [MQTT topics](#5-mqtt-topics)
6. [Web panel layout](#6-web-panel-layout)
7. [Firmware architecture](#7-firmware-architecture)
8. [HTTP API](#8-http-api)
9. [Can the ESP32-C3 Core handle it?](#9-can-the-esp32-c3-core-handle-it)
10. [Implementation phases](#10-implementation-phases)
11. [Test plan](#11-test-plan)
12. [Open questions](#12-open-questions)

---

## 1. Versions and fallback

| Version | What | Where |
|---|---|---|
| **v0.1.0** | The firmware running on the board since Sep 27 (what's on `main` today) | Frozen; git tag + a copy of the source |
| **v2.1.0** | This plan | `ai/emulator/esp32_c3_emulator/` (keeps the current path, so links and scripts keep working) |

**Freezing v0.1.0 (phase 0, before any v2 code):**

- **Git tag** `emulator-v0.1.0` on the commit that holds today's firmware. This is the
  authoritative copy.
- **A frozen source copy** in `ai/emulator/releases/v0.1.0/`: the ESP-IDF project as-is, plus a
  short `README.md` covering what it does, how to build it and how to flash it back. Its full
  user guide is the current `README.md`, copied there unchanged. You can rebuild v0.1.0 without
  knowing git.
- **A prebuilt firmware image** made with `idf.py merge-bin`. It's a single file that includes
  the bootloader and partition table, so falling back is one `esptool.py write_flash 0x0 …`
  command, with no ESP-IDF needed. **It doesn't go in git**:
  - the image has the team Wi-Fi and broker passwords compiled in;
  - the repo rule is no build artifacts.

  It goes in the **team Google Drive folder**, next to the shared passwords, as
  `emulator-v0.1.0-merged.bin` with its SHA-256. The same applies to every v2 release image.
- **A version string from one place:** `PROJECT_VER` in `CMakeLists.txt`. It feeds
  `esp_app_get_description()`, the panel, `/api/state` and `/api/meta`. This replaces the
  hard-coded `FW_VERSION "0.1.0"` in `emulator_ctrl.c`.
- **Fallback is safe in both directions:**
  - v0.1.0 ignores the new NVS settings v2 stores.
  - If v2 changes the partition table (see §9), flashing the merged v0.1.0 image restores the old
    table too.
- **The v2 README gets a "Falling back to v0.1.0" section** with the exact commands.

---

## 2. Feature list

| # | Feature | Mode | Notes |
|---|---|---|---|
| F1 | **Broker (Raspberry Pi) IP changeable from the panel** | both | Part of F2 |
| F2 | **MQTT settings**: broker host/IP, port, username, password, keepalive, sensor publish interval | both | Saved in NVS. The `sdkconfig.local` values stay as defaults, with a "restore defaults" button. Applying reconnects all MQTT clients without a reboot. The password field can be set but never read back |
| F3 | **Dev / Demo tabs** | — | §3 |
| F4 | **"Roll the house" button** (stands in for the game buzzer) | demo | Triggers an actual random roll (F6) |
| F5 | **50 preset scenarios** | demo | §4.3 |
| F6 | **Actual random roll** within each entity's caps, with **number-rolling animation** | demo | §4.4 |
| F7 | **Scenario loop on/off**, plus its interval | demo | §4.5 |
| F8 | **Actual-random controls**: roll length, hold time, how values are spread | demo | §4.4. See open question Q1 |
| F9 | **°C and °F side by side** for every temperature | both | Display only; MQTT stays °C |
| F10 | **Three AI message boxes**, one per node, styled like the DWIN AI section | both | Each shows the latest `intellithings/<node_id>/display` message, its `ts`, the retained flag and a character count against the 60-character limit. Each has its own "send test message" field. There's also a "thinking…" indicator from `intellithings/ai/inference_status` and the overall summary from `intellithings/ai/summary` |
| F11 | **System log window** | both | Command log (who sent what: web or MQTT, accepted or rejected, why) plus the firmware's own log lines. Filter, pause, clear and download |
| F12 | **Live MQTT route diagram** | both | Publisher → broker → subscriber, with an edge that lights up on each message. §6.4 |
| F13 | **Last value per sensor**: current value, previous value, change ▲▼ and when it last changed. Click for the last 20 values | both | |
| F14 | **Plot per sensor** over the past **10 min / 30 min / 1 h** | dev (demo: step history) | **Chart style setting**: line, area, step, bar or min–max band, plus a table view. The choice is kept per browser |
| F15 | **On-page user guide**: popup, search box, a "How to test this portal" checklist, and a **download link** (`/guide.md`) | both | §6.5 |
| F16 | **Hideable sections**: every card gets `[hide]` / `[show]`, and advanced parts sit behind `[more]` | both | Kept per browser |
| F17 | **Demo publishes to `intellithings/emulator/demo/#`**; dev mode keeps `intellithings/<emu-node>/…` and `intellithings/emulator/{status,state,cmd}` | — | §5 |
| F18 | **Backward compatible**: every v0.1.0 command, endpoint and topic works unchanged | both | AI team scripts and `register_emulator.py` keep working |

---

## 3. Dev mode and demo mode

- **Dev mode** is v0.1.0's job. The three emulated nodes publish on the real node topics every
  3 s, driven by the room simulation, shortcuts and custom values. Nothing changes for HA or the
  agent.
- **Demo mode (experimental)** runs a **demo-home engine**. It holds the whole-home entities, the
  50 presets, the loop and the roll, and publishes only under `intellithings/emulator/demo/#`.
- **What the tabs switch:** the **panel view** and which engine you're controlling. **Demo
  publishing has its own on/off switch** in the Demo tab, off at boot.
- **Recommendation:** keep the three dev nodes publishing while the Demo tab is open. Then:
  - the existing HA entities never go unavailable;
  - someone testing demo mode can't silently break another teammate's dev test.

  A "pause dev nodes" option can be added if the AI team wants a quiet broker during demo tests
  (Q2).
- **The two engines share no state.** Demo values never leak into `intellithings/emu-*/sensors`.

---

## 4. Demo mode in detail

This follows [`demo/README.md`](../../demo/README.md). The emulator is the **test stand-in for the
demo master**, so the AI team can build HA demo entities, automations and agent behaviour before
the real buzzer board exists.

### 4.1 Reuse with the real demo master

- **One data file:** the demo-home entity table and the 50 presets are written as data, in
  `demo/demo_home.json` inside the emulator project. A small generator turns it into C tables at
  build time.
- **One logic module:** the entity model, randomizer and scenario selection go in plain C
  (`demo_home.[ch]`) with no ESP-IDF calls, like `sim_model.c`.
- **Moving to `demo/demo-master/` later** means copying the JSON, the generator and
  `demo_home.[ch]` over unchanged.
- **HA discovery for the demo entities** could be generated from the same JSON (AI team's call).

### 4.2 Demo-home entities (draft for the AI team)

Each entity has a **key**, area, label and type. Types:
- **number**, with unit, decimals, **low/high caps** and a "normal" range;
- **bool**;
- **enum**, with its options.

Caps are hard limits that no value ever crosses. Draft, about 60 entities:

| Area | Entities (draft) |
|---|---|
| **Kitchen, living room, master bedroom** (the three node rooms) | The full node sensor set: `temperature_c`, `humidity_pct`, `voc_index`, `co2_ppm`, `lux`, `distance_mm`, `pm1_0_ugm3`, `pm2_5_ugm3`, `pm10_ugm3`, `presence`; plus `light` (on/off) and `window` (open/closed) |
| **Second bedroom** | `temperature_c`, `humidity_pct`, `co2_ppm`, `presence`, `light`, `window` |
| **Garage** | `temperature_c`, `door` (open/closed), `car` (home/away), `light`, `presence` |
| **Patio** | `door` (open/closed), `light`, `motion` |
| **Garden** | `soil_moisture_pct`, `sprinkler` (on/off), `motion` |
| **Outside (weather)** | `temperature_c`, `humidity_pct`, `condition` (clear / cloudy / rain / snow / storm), `wind_kph`, `pm2_5_ugm3` |
| **Home** | `front_door` (locked/unlocked), `anyone_home`, `time_of_day` (morning / afternoon / evening / night) |

- **Example caps:**
  - indoor temperature **15.6–37.8 °C** (60–100 °F, as in the demo plan);
  - outdoor temperature −20…40 °C;
  - CO₂ 400–5000 ppm.

  The node-room sensors use the node contract's ranges as their outer limits.
- **Node-room rule:** the three node rooms use **exactly the node payload field names**, so their
  data can also go out on the per-node demo topic (§5).
- **Status:** this list is a **proposal**. It goes into
  [`docs/interfaces/ha-entities.md`](../../docs/interfaces/ha-entities.md) for @ProgrammingJohn to
  agree. Until then it lives only in the emulator, under the emulator's own namespace.

### 4.3 The 50 presets

- **Each preset sets every entity.**
- **Authoring:** a preset lists only the entities that matter to it. The generator fills the rest
  from each entity's normal range with a **fixed per-preset seed**, so a preset always gives the
  same complete home.
- **Mix:** about **20 normal, 30 abnormal**, so visitors see the agent both stay calm and react.

| Group | Count | Examples |
|---|---|---|
| Normal day / evening / night | 20 | Quiet weekday morning, everyone at work, family dinner, movie night, everyone asleep, weekend gardening |
| Air quality | 8 | Burnt toast (PM2.5 spike), stuffy bedroom overnight, cleaning-spray VOC, wildfire smoke outside with windows open |
| Heat / cold | 6 | Hot empty room with the AC off, cold snap with a window open, heat wave with nobody home |
| Security / doors | 6 | Garage door open at night, front door unlocked while away, patio motion at 3 am |
| Energy waste | 5 | Lights on in empty rooms, sprinkler running in the rain, window open with heating on |
| Hazard | 5 | Kitchen near 100 °F + smoke (fire hazard), car running in a closed garage (CO₂/VOC), very high humidity in the bedroom |

The full list is drafted in phase 4 and reviewed with the AI team before it's flashed.

### 4.4 Actual random roll ("Roll the house")

1. **Press** the panel's 🔴 **Roll the house** button (or send the `demo roll` command). The
   loop pauses.
2. **Draw:** the firmware picks the new values **at roll start**, so every open panel lands on
   the same numbers. Numbers stay within their caps; bools and enums are rolled too.
3. **Roll:** for about **4 s** (configurable), every panel plays the **number-rolling
   animation**. **Nothing is published while rolling**, so HA and the agent never see in-between
   values.
4. **Land:** the values are published once (§5). The state shows `landed`.
5. **Hold:** the result stays for a **fixed hold time** (configurable, e.g. 120 s). Further roll
   presses are **ignored** until roll + hold are over, as the buzzer notes in
   `demo/demo-master/README.md` describe.
6. **Resume:** after the hold, the loop resumes if it's on. Otherwise the rolled home stays.

**How values are spread** (an open question in the demo plan, so it's a setting):

- **uniform within caps** ("actually random");
- **lean to normal**: mostly inside the normal range, with an X % chance per entity of going
  anywhere within the caps.

Abnormal-but-realistic is the point, so near-cap values are allowed. For example, 37 °C in the
kitchen should read as a possible fire.

### 4.5 Scenario loop

- **On/off switch** plus an **interval** (default 90 s, adjustable; the right value is still an
  open discussion, and this is where it gets tried).
- **What the loop picks** (setting): **presets only** (default), **random rolls only**, or a
  **mix**. In a mix, each step is a roll with probability X %.
- **No immediate repeats:** the same preset is never picked twice in a row.
- **Manual pick:** clicking a preset in the library applies it now and holds it. The loop, if on,
  resumes after the hold time.

### 4.6 Demo engine states

```
off ──publish on──► idle/held ──loop on──► looping ──interval──► (next preset or roll)
                        ▲                     │
                        │        roll button  ▼
                        └──── hold ◄── landed ◄── rolling (≈4 s, nothing published)
```

The current state, the preset or roll ID, the time left in the step and the lockout are all in
`/api/state` and `intellithings/emulator/demo/state`.

---

## 5. MQTT topics

**Dev mode: unchanged from v0.1.0.**

| Topic | Notes |
|---|---|
| `intellithings/<emu-node>/sensors` · `/status` · `/display` | Real node contract, three nodes |
| `intellithings/emulator/status` · `/state` · `/cmd` | Control channel. `state` gains fields but keeps the existing ones, so `register_emulator.py` keeps working |

**Demo mode: new, under the emulator's reserved namespace.**

| Topic | QoS / retained | Payload (draft) |
|---|---|---|
| `intellithings/emulator/demo/state` | 1 / yes | `{"state":"looping","source":"preset","scenario_id":12,"scenario":"Garage door open at night","step_left_s":64,"loop":true,"ts":"…"}` |
| `intellithings/emulator/demo/home` | 1 / yes | `{"scenario_id":12,"ts":"…","areas":{"kitchen":{"temperature_c":24.1,"light":"on",…},"garage":{"door":"open","car":"home",…},…}}` (about 2 KB) |
| `intellithings/emulator/demo/<emu-node>/sensors` | 1 / yes | **Exactly the node `…/sensors` schema** (°C) for that node's demo room, so node display code can be tested with demo data |

- **When it publishes:** only when values change (preset applied or roll landed), and never
  during a roll. The retained flag means late subscribers get the current home.
- **Which connection:** demo traffic uses the **existing control connection**, so no fifth MQTT
  client (§9). `intellithings/emulator/status` already works as its Last Will.
- **New read-only subscriptions on the control connection:** `intellithings/ai/summary` and
  `intellithings/ai/inference_status`, for F10 and the route diagram. These are existing contract
  topics; subscribing changes nothing for anyone else.
- **AI messages:** they keep arriving on `intellithings/<emu-node>/display`, the same as in dev
  mode.
- **Contract step:** before v2.1.0 publishes them, these three topics go into
  [`docs/interfaces/mqtt.md`](../../docs/interfaces/mqtt.md) under "Reserved:
  `intellithings/emulator/#`", marked experimental. This needs @ProgrammingJohn's review (and
  @LukeTuthill's for the per-node demo topic).
- **Relation to the real demo master's topics:** the real demo master's
  `intellithings/demo/#` topics stay a separate proposal. When they're agreed, the emulator's
  demo payloads should match them, so switching is a topic-prefix change.

---

## 6. Web panel layout

**Goal: less on screen by default.** Each tab shows its three or four most-used things; the rest
sits behind `[more]`, inside collapsible cards, or in popups.

### 6.1 Frame (both tabs)

```
IntelliThings Emulator v2.1.0    [ Dev ] [ Demo (experimental) ]     ● Wi-Fi ● MQTT 4/4 ● clock   [?] Guide  [⚙] Settings
```

- **Badges:** clicking one opens the system status details (IP, broker, uptime, heap, version).
- **Guide (F15)** and **Settings (F2, F9, F14, F16)** open as popups.
- **Bottom dock with two collapsible panels:**
  - **System log** (F11): last command shown in one line when collapsed.
  - **MQTT route** (F12).

### 6.2 Dev tab

1. **Room cards** (three side by side; one column on a phone). Each sensor tile shows:
   - the current value, with °C / °F side by side for temperature;
   - the previous value and a ▲▼ change;
   - a small sparkline.

   Clicking a tile opens the **plot** with 10 min / 30 min / 1 h buttons and the chart style
   menu, plus the **last-20 table**. Mode badges and `↺ Normal` work as in v0.1.0.
2. **Emulate:** room target tabs and the 12 shortcuts stay visible. **Custom values** and
   duration options sit behind `[more]`.
3. **AI messages:** three DWIN-style boxes (F10). The full message history (last 20) sits behind
   `[more]`.

### 6.3 Demo tab

1. **Control bar:**
   - big 🔴 **Roll the house** button (shows the lockout countdown);
   - **Loop** on/off and interval;
   - **Publish demo data** on/off;
   - the current step (preset name or "random roll #7") and the time left.

   Roll settings (length, hold, spread, loop source) sit behind `[more]`.
2. **The demo home:** areas as columns and entities as rows, like the booth monitor will show
   them. Values play the **rolling animation** on every change; temperatures show °C / °F.
   Clicking an entity shows its last values (F13) and a step plot of the past hour (F14).
3. **Preset library** (collapsed): the 50 presets, filterable by group. Click one to apply it.
4. **AI messages:** the same three boxes plus the overall summary and the "thinking…" state.

### 6.4 Live MQTT route diagram (F12)

An SVG flowchart drawn in the browser from an event list the firmware keeps (the last 64
messages it sent or received: time, topic, direction, size).

```
[Emulator: 3 nodes] ──sensors──►            ──►  [Home Assistant] ──► [AI agent]
[Emulator: demo]    ──demo/*───►  [Broker   ]                            │
[Panel / scripts]   ──cmd──────►  [192.168.1.2] ◄── display / ai/* ◄─────┘
[Emulator: AI boxes] ◄─display───
```

- **Observed edges:** an edge the emulator **saw** (it published, or it received) pulses and
  shows its last topic and how long ago.
- **Unobserved edges:** the emulator can't see who else is subscribed, so broker → HA and
  HA → agent are drawn **dashed, labelled "expected"**. The exception is when
  `ai/inference_status` shows the agent thinking, which lights the agent box.
- **Click an edge** to see its last few messages.

### 6.5 User guide (F15)

- **One source file:** the guide is a Markdown file, `ai/emulator/GUIDE.md`, embedded in the
  firmware. The repo copy and the on-board copy are the same file.
- **Popup:** the panel renders it in a popup with a table of contents and a **search box** that
  filters sections and highlights matches.
- **`?` links:** each card has a small `?` that opens the guide at its section.
- **"How to test this portal" section:** a short checklist covering the badges, sending a
  command, watching it arrive in the log and the route diagram, sending a test AI message, and
  rolling the house.
- **Download:** `/guide.md` is served with `Content-Disposition: attachment`. The guide also
  links to the repo README for scripting details.

### 6.6 Settings popup

| Section | Items |
|---|---|
| **MQTT** | Broker host/IP, port, username, password (write-only), keepalive, sensor publish interval. **Apply & reconnect**, **Restore defaults**, connection result shown live |
| **Display** | Temperature: °C / °F / both · default plot window · chart style · refresh rate |
| **Layout** | Show or hide each card; reset layout |

- **Where settings live:** display and layout settings are stored in the **browser**
  (`localStorage`). Only MQTT settings are stored on the board.
- **No login (as in v0.1.0):** anyone on the Wi-Fi can now change the broker. A wrong broker
  doesn't lock you out, because the panel is HTTP and doesn't depend on MQTT. An optional
  **settings PIN** is Q5.

---

## 7. Firmware architecture

**Kept as-is:** `wifi_manager`, `time_sync`, `sim_model`, `scenarios`, `payload`,
`virtual_node`.

**Changed:**
- `mqtt_node` and `emulator_ctrl`: broker settings come from `app_config` instead of Kconfig, and
  `mqtt_reconfigure()` restarts all four clients with the new settings.
- `emulator_ctrl` also gets the demo commands, the extra state fields, and the new subscriptions
  and publishes.
- `ai_message`: latest message per node, plus the summary and thinking state.

**New modules:**

| Module | Plain C? | Purpose |
|---|---|---|
| `app_config.[ch]` | no | Runtime settings in NVS (`emu` namespace), Kconfig/`sdkconfig.local` as defaults, validation, write-only password |
| `history.[ch]` | **yes** | Ring buffers per node × metric (§9), packed 16-bit values |
| `event_log.[ch]` | no | (a) system log: hooks `esp_log_set_vprintf`, keeps an 8 KB text ring and still prints to serial; (b) command log; (c) MQTT route events. Each entry has a sequence number so the panel fetches only what's new |
| `demo_home.[ch]` | **yes** | Entity model, caps, presets, randomizer (lean-to-normal or uniform), loop picker |
| `demo_engine.[ch]` | no | Demo state machine (§4.6), timers, publishing over the control connection |
| `demo/demo_home.json` → generated `demo_data.c` | — | Entities and 50 presets as data. A Python generator runs from CMake at build time (ESP-IDF already ships Python) |
| `web/` | — | `index.html` with inline CSS and JS, gzipped at build time; plus `GUIDE.md`, gzipped and embedded |

**Rules carried over from v0.1.0:**
- one nodes lock;
- no MQTT calls while holding a lock;
- commands validated before anything changes.

**New rules:**
- **Large responses** (history, log) are **streamed in chunks** and never built as a cJSON tree.
- **The roll result is drawn under the lock;** it's published after the lock is released.

---

## 8. HTTP API

| Endpoint | v0.1.0 | v2.1.0 |
|---|---|---|
| `GET /` | panel | panel (gzipped) |
| `GET /api/state`, `/api/messages`, `/api/meta` | ✓ | unchanged fields + new ones (version, demo, config summary) |
| `POST /api/cmd` | ✓ | same commands + `demo` commands |
| `GET /api/poll?log=&ev=&msg=` | — | **One call per refresh**: state plus new log lines, route events and messages since the given sequence numbers. Keeps each browser to one or two sockets |
| `GET /api/history?tier=10m\|1h` | — | Binary, chunked; the browser decodes it once on load and then appends from polls |
| `GET /api/demo/history` | — | Last 30 demo-home snapshots |
| `GET/POST /api/config`, `POST /api/config/reset` | — | MQTT settings (password never returned) |
| `GET /guide.md` | — | Guide download |

**New commands** (HTTP and `intellithings/emulator/cmd`). `node` isn't needed for these:

```jsonc
{"action":"demo","op":"publish","on":true}                 // demo publishing on/off
{"action":"demo","op":"roll"}                              // "Roll the house"
{"action":"demo","op":"loop","on":true,"interval_s":90,"source":"presets"}  // presets | random | mix
{"action":"demo","op":"preset","id":12}                    // apply and hold one preset
{"action":"demo","op":"settings","roll_s":4,"hold_s":120,"spread":"normal","abnormal_pct":15}
```

---

## 9. Can the ESP32-C3 Core handle it?

**Short answer: yes, if the heavy work stays in the browser and three build settings change.**
The board only stores compact data and serves it. Plots, animations, the route diagram, guide
search and Markdown rendering all run in the browser.

### Board

| | |
|---|---|
| CPU | 1 × RISC-V core at 160 MHz |
| RAM | 400 KB SRAM |
| Flash | 4 MB |
| PSRAM | none |

### Measured on v0.1.0

| Item | Value |
|---|---|
| Free heap | ~137 KB (1 h soak; lowest 111 KB) |
| App image | **1.12 MB of the 1.5 MB app partition (71 %)** |
| Build settings behind that size | **debug optimisation (`-Og`)** and **MQTT over TLS and WebSocket enabled**; v0.1.0 uses neither |
| Panel page | 20 KB (6.5 KB gzipped) |
| Sockets | 16 (4 MQTT + 3 httpd internal + 7 panel clients) |

### RAM budget for v2.1.0

| New item | Size |
|---|---|
| Sensor history, 3 nodes × 10 series, 16-bit: **10 min at 3 s** (200 samples) + **1 h at 15 s** (240 samples) | 26 KB |
| Demo history (30 snapshots × ~60 entities × 16-bit) + demo state | ~6 KB |
| System + command log ring | 8 KB |
| MQTT route events (64) | 3 KB |
| Extra per-request peak (bigger state JSON, chunked streaming) | ~10–15 KB transient |
| **Total** | **~55 KB → ~80 KB free left, lowest ~55 KB** |

- **Target:** lowest free heap stays **above 50 KB** in a 2-hour soak with two browsers open.
- **If it gets tight:** shrink the 10-min tier to 5 min (−6 KB), or the log to 4 KB.
- **The 50 presets live in flash** (about 8 KB of `const` data), not RAM.

### Flash budget

- **Growth:** new code and data ~60–100 KB, plus the gzipped panel and guide ~60–100 KB.
- **Without changes**, that's about 1.3 MB of 1.5 MB. It fits, but with little room left.
- **Planned fixes:**
  - **Optimise for size (`-Os`):** typically saves 15–25 %.
  - **Turn off the MQTT TLS and WebSocket transports:** saves mbedTLS code and some RAM. The
    broker is plain MQTT on the LAN.
  - **Custom partition table:** two **1.9 MB OTA slots** (4 MB flash fits nvs + otadata + phy +
    2 × 1.9 MB). That leaves room to grow, and it makes the optional in-place fallback below
    possible. A single 3 MB app slot also works if OTA isn't wanted.

### CPU and network

- **Traffic:** about 1 sensor publish per second on average, demo publishes only on change, and
  one poll per second per browser. All of it is light work.
- **Polling vs WebSocket:** v2 keeps **HTTP polling** but merges it into one `/api/poll` call per
  refresh instead of several. WebSocket push is possible (`CONFIG_HTTPD_WS_SUPPORT`) but adds
  complexity. Revisit only if polling proves too slow.
- **Sockets stay at four MQTT connections**, because demo traffic reuses the control connection.
  The page is a single inlined file, so loading it takes one request. If more than about 3
  browsers are often open together, raise `LWIP_MAX_SOCKETS` to 20.

### Risks

| Risk | Mitigation |
|---|---|
| Log hook called from many tasks | Non-blocking: short critical section and `memcpy` only; never logs from inside itself |
| Applying new broker settings mid-publish | `mqtt_reconfigure()` runs on the HTTP task, stops the sim task's publishing with a flag, destroys and recreates the clients, then resumes |
| NVS wear | Writes only on Settings → Apply, never per command |
| Heap fragmentation from bursts of large JSON | Streamed responses; fixed-size rings allocated at boot |

---

## 10. Implementation phases

| Phase | Work | Done when |
|---|---|---|
| **0. Freeze v0.1.0** | Tag, frozen copy in `releases/v0.1.0/`, merged image + SHA-256 in Drive, `PROJECT_VER`, fallback section in README | Board reflashed from the merged image and still works |
| **1. Build and memory groundwork** | `-Os`, TLS/WS transports off, custom partition table, gzip step for web assets. Re-measure image and heap | Image and heap numbers recorded in `TODO.md`; v0.1.0 behaviour unchanged |
| **2. Back end** | `app_config` + runtime broker change, `event_log`, `history`, `/api/poll`, `/api/history`, `/api/config`, `/guide.md`; v0.1.0 API still answers the same | The Sep 27 API checks (46, see `TODO.md`) still pass, rerun as a committed script; broker change works without reboot |
| **3. Panel v2 frame + dev tab** | Tabs, hideable cards, settings and guide popups with search, sensor tiles (°C/°F, last value, plots, chart styles), three AI boxes, log window, route diagram | Every F-item for dev mode ticked in the browser (Chrome + a phone) |
| **4. Demo mode** | Entity JSON + generator, 50 presets (reviewed with the AI team), `demo_home` + `demo_engine`, demo topics, demo tab with rolling animation | Roll, loop and presets publish the agreed payloads; nothing published mid-roll |
| **5. Docs and contracts** | `mqtt.md` emulator demo topics (lead review), entity draft in `ha-entities.md`, README v2 + `GUIDE.md`, `TODO.md` test log, Guideline §3.1a and `demo/README.md` pointers | PR into `dev` reviewed |
| **6. Soak and release** | 2-hour soak with two browsers, a broker change mid-run and the demo loop on; merged v2.1.0 image to Drive | Lowest heap > 50 KB, no reboots |

**While touching the docs**, fix the stale v0.1.0 lines:
- `TODO.md` (dated Sep 27; "not pushed yet");
- README "draft until Oct 4" and its HA YAML example (HA actually uses `register_emulator.py`);
- Kconfig help "Home Assistant add-on" and the `homeassistant.local` default; default to
  `192.168.1.2` instead.

**Optional, later:** **firmware upload from the panel** (OTA) with a **"boot previous version"**
button. With the two-slot partition table from phase 1, the board could keep v0.1.0 (or the last
good v2) in the other slot and fall back without a USB cable.

---

## 11. Test plan

The existing v0.1.0 checks stay. New checks:

- **Broker settings:**
  - Change the broker to a wrong IP → MQTT badge goes red and the panel still works.
  - Change it back → all four clients reconnect, statuses `online`, retained `display` messages
    re-delivered.
  - The settings survive a reboot; "restore defaults" works.
  - The password never appears in any response.
- **History:**
  - The 10 min / 30 min / 1 h plots fill as expected.
  - Reloading the page restores the history from the board.
  - A reboot clears it, as documented.
- **Log and route diagram:**
  - A command sent over MQTT appears in the log with source `mqtt`.
  - The `cmd` edge pulses.
  - A test AI message lights the `display` edge and the right box.
- **AI boxes:**
  - A 61-character message shows the over-limit warning.
  - A retained message after reboot shows `retained`.
  - `inference_status` start/stop toggles "thinking…".
- **Demo mode:**
  - Publish switch off → nothing under `emulator/demo/#`.
  - Each preset publishes the full home once.
  - During a roll, nothing is published; afterwards, exactly one publish happens.
  - Presses during roll + hold are ignored.
  - The loop never repeats a preset back to back.
  - Every rolled value is within its caps (also checked by a host test over 10 000 rolls once a
    host compiler is set up).
- **Load:** two browsers plus one script polling, 2 hours; record the lowest heap and the socket
  errors in `TODO.md`.
- **Fallback:** flash the merged v0.1.0 image, check that it boots and publishes, then flash
  v2.1.0 back.

---

## 12. Open questions

**Decided Oct 18:** the suggested answers below were accepted.

| # | Question | Answer |
|---|---|---|
| Q1 | "Control for actual random": does this mean the roll settings, or a switch that makes the **loop** use random rolls instead of presets? | Both are planned: roll settings (length, hold, spread) and the loop source (presets / random / mix) |
| Q2 | Should the dev nodes keep publishing while demo mode is active? | Yes (§3), with an optional "pause dev nodes" switch |
| Q3 | Demo per-node topics: reuse `emu-kitchen` / `emu-bedroom` / `emu-living-room`? The demo home has a *master* bedroom; map it to `emu-bedroom`? | Yes, and yes |
| Q4 | Frozen v0.1.0: tag **and** a source copy in `releases/v0.1.0/`, or the tag only? | Both (no git knowledge needed to rebuild) |
| Q5 | Add a PIN to the Settings popup now that the broker can be changed from the page? | Optional PIN, off by default |
| Q6 | Two-slot OTA partition now (enables in-panel fallback later) or a single 3 MB slot? | Two slots |
| Q7 | Loop interval and roll/hold defaults to start testing with | 90 s loop, 4 s roll, 120 s hold |
