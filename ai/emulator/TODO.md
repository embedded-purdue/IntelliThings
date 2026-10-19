# Emulator — progress and test plan

Owner: @spicybutter · last updated 2026-10-18

## v2.1.0 progress

Plan: [`PLAN-v2.1.0.md`](PLAN-v2.1.0.md) · branch `ai/emulator-v2` · the board still runs
**v0.1.0**. The rest of this file below is the v0.1.0 log; its frozen copy is in
[`releases/v0.1.0/`](releases/v0.1.0/README.md).

### Phase 0: freeze v0.1.0 (Oct 18)

- [x] Tag `emulator-v0.1.0` on `967fe5e`; source copy + `dependencies.lock` in
  `releases/v0.1.0/` (identical to the tag apart from the lock file)
- [x] Rebuilt from the copy (0 warnings), merged image made, flashed to the board, boots
- [x] `PROJECT_VER` in `CMakeLists.txt` is the only version (`2.1.0-dev`); `/api/state` reads it
- [ ] Upload `emulator-v0.1.0-merged.bin` + `.sha256` to the team Drive

### Phase 1: build and memory groundwork (Oct 18)

Changes: `-Os`, MQTT TLS and WebSocket transports off, `partitions.csv` with two 1.9 MB app
slots, panel gzipped at build time (`tools/gzip_asset.py`) and served with
`Content-Encoding: gzip`. One `snprintf` bound tightened for `-Os`'s stricter truncation check.

| | v0.1.0 | Phase 1 |
|---|---|---|
| App image | 1 122 260 B | **997 554 B** (−11 %) |
| App slot / free | 1.5 MB / 27 % | **1.94 MB / 51 %** |
| Flash code / data | 860 630 / 175 200 B | 765 884 / 152 888 B |
| Static DRAM | 125 242 B | **117 578 B** (−7.7 KB) |
| Panel download | 20 492 B | **6 512 B** (byte-identical after unzip) |
| Free heap, Wi-Fi up, broker down, ~50 s | 139 640 B (lowest 120 084) | **146 536 B (lowest 126 680)** |

Tested on the board (Oct 18, team Wi-Fi, **Pi was off**):
- [x] Boots from `ota_0` with the new table; version `2.1.0-dev`; simulation and 3 s cycle run
- [x] Wi-Fi joins (−38 dBm), clock syncs, panel loads gzipped, HTTP commands accepted/rejected
  as before (scenario, out-of-range `set` → 400, reset, `/api/meta`, `/api/messages`)
- [x] Fallback: the merged v0.1.0 image restores the old table; v0.1.0 boots and serves the panel
- [ ] **MQTT with the new build** (Pi/broker unreachable on Oct 18): 4 connections, statuses,
  publishing, display messages, then heap with MQTT up (v0.1.0 had ~137 KB)
- [x] Panel in a real browser with the gzipped page (in-app Chromium, during phase 2: renders,
  no console errors) — [ ] still to check on a phone

### Phase 2: back end (Oct 18)

New modules: `app_config` (runtime broker/publish settings in NVS, Kconfig as defaults),
`event_log` (system log via a log hook, command log, MQTT route events), `history` (plain C,
10 min fine + 1 h coarse tiers). New endpoints: `/api/poll`, `/api/history` (binary),
`/api/config` (+ `/reset`), `/guide.md` (from `GUIDE.md`, gzipped). The control connection also
subscribes (read-only) to `intellithings/ai/summary` and `…/inference_status`. Broker changes
stop, reconfigure and restart the existing MQTT clients (handles are never freed, so no locks
around publishing); clearing a login reboots, since esp-mqtt can't drop it from a live client.
Kconfig default broker is now `192.168.1.2`, with a new keepalive option.

| | Phase 1 | Phase 2 |
|---|---|---|
| App image | 997 554 B | 1 017 202 B (50 % of the slot free) |
| Static DRAM | 117 578 B | 167 226 B (+49.6 KB: history 28 KB, logs 18 KB, AI state) |
| Free heap, broker down, ~1 min | 146 536 B (lowest 126 680) | **~90 000 B (lowest ~70 900)** |

Tested on the board (Oct 18, Pi still off), `tests/api_check.py`:
- [x] 66/66 checks (v0.1.0 commands and validation, poll, history, guide, config) and 70/70 with
  `--broker-test`: switching to an unreachable broker and back without a reboot, 3.3 s per switch,
  panel answering throughout; settings left as found
- [x] Bug found and fixed: `on_ip` still called `ctrl_start()` before the broker URI was set
  (the control client was created twice, first with an empty URI)
- [x] Bug found and fixed: `/api/state` showed the old broker until every client had stopped (up
  to a 10 s connect timeout). Now the new broker shows at once, `mqtt.reconfiguring` covers the
  switch, and the network timeout is 5 s
- [ ] **With the broker up:** reconnection after a broker change, the retained `offline` left
  on the old broker, display messages and AI summary/thinking via `/api/poll`, heap with 4 live
  connections (target: lowest > 50 KB)
- [ ] Clearing the login (reboot path)

---

## v0.1.0 log

Implementation is finished and builds cleanly. The offline boot test passed on the board.
**Everything that needs Wi-Fi and the MQTT broker is still untested** and waits for a team
session with the network and the Home Assistant Pi.

## Done

- [x] Firmware in `esp32_c3_emulator/`: C on ESP-IDF v6.1 for the LuatOS ESP32-C3 Core
  - three virtual nodes `emu-kitchen`, `emu-bedroom`, `emu-living-room`, each with its own
    MQTT connection and Last Will
  - stateful per-room simulation (occupancy, CO₂ mass balance, PM/VOC events with decay,
    daylight + lamps, distance), automatic events, 12 shortcut scenarios, custom range overrides
  - sensor snapshot published **every 3 s even if unchanged**
  - `display` subscription per node → serial banner + message history
  - control channel on its own MQTT connection (`intellithings/emulator/{status,state,cmd}`)
  - web control panel + HTTP API served by the board (`http://intellithings-emu.local/`)
  - credentials only in gitignored `sdkconfig.local` / `sdkconfig`
- [x] Build: `idf.py build` with ESP-IDF v6.1 — **0 warnings, 0 errors**; image 313 KB of a
  1.5 MB app partition
- [x] Flashed to the board (COM8) and ran **offline** (no Wi-Fi configured):
  - boots, sets up the three nodes, runs the sim, 3 s publish cycle is exact
  - example payload JSON is well-formed; floats format correctly with picolibc
  - rooms behave independently (different readings, occupancy and light)
- [x] Code review: string termination and truncation of inbound payloads, lock ordering (fixed
  a possible deadlock between HTTP and MQTT command paths), no MQTT calls while holding locks
- [x] Docs: `README.md` (full guide), `docs/interfaces/mqtt.md` (draft contract), plus updates to
  `ai/README.md`, Project Guideline §3.1a, Firmware Architecture §6, Collaboration Guidelines
  mock snapshot, `docs/interfaces/README.md`

> The board now runs a build with the team Wi-Fi from `sdkconfig.local`. The broker is still
> `homeassistant.local`, which doesn't exist until the Mosquitto broker is set up; if the Pi
> doesn't resolve by that name, put its IP in `sdkconfig.local` and rebuild.

## To test at the next team session

### Setup

- [x] Mosquitto broker (its own Docker container next to HA Container) running on the Pi at `192.168.1.2:1883`
  (`homeassistant.local` doesn't resolve on the team network — use the IP). The emulator
  currently logs in as the broker's `homeassistant` user — [ ] create a dedicated
  MQTT login for it
- [x] `sdkconfig.local` filled in (team Wi-Fi + broker), `sdkconfig` deleted, rebuilt, flashed
- [x] Laptop subscriber: no Mosquitto clients on the dev PC, so the tests used a small
  paho-mqtt script reading the broker settings from `sdkconfig.local`

### Wi-Fi

- [x] Connects; serial shows IP and the control-panel URL — 2026-09-27 on the team network,
  RSSI −20…−33 dBm. Boot backoff seen working (1 s → 2 s → 4 s): after a reset while
  associated, the AP (PMF) refuses re-association for ~15 s ("comeback time too long",
  reason 208); the board retries and joins by itself
- [ ] Reconnect: turn the AP off/on (or move out of range) → retries with backoff (1 s → 30 s),
  reconnects by itself; panel's "Wi-Fi reconnects" goes up
- [x] Clock syncs (serial `Clock synced: …`; panel shows local time) — synced 40 ms after
  Wi-Fi came up. **Bug found and fixed:** with the broker unreachable it never synced (4 failing
  mDNS broker lookups held all 4 lwIP DNS request slots, starving SNTP's lookup);
  `CONFIG_LWIP_SNTP_STARTUP_DELAY=n` in `sdkconfig.defaults` sends the NTP request before the
  MQTT clients start. Payload `ts` confirmed once the broker was up

### MQTT — node contract

- [x] Four connections come up: three nodes + emulator, all within 100 ms of Wi-Fi
- [x] `intellithings/<id>/status` = `online` (retained, QoS 1) for all three, and
  `intellithings/emulator/status` = `online`
- [x] Each node publishes `…/sensors` every 3 s (measured gaps 2.95–3.07 s), QoS 0, not retained
- [x] Payload fields, types and ranges match `docs/interfaces/mqtt.md`; `ts` present and valid
  ISO 8601 with offset in every message
- [x] Last Will: board held in reset (radio off, same as unplugging) → all four statuses flipped
  to `offline` after 22 s (keepalive 15 s × 1.5); back `online` 2 s after release. A quick
  reset/reflash also publishes `offline`, then `online` once it reconnects
- [x] After a reboot: resubscribes; retained `display` messages re-delivered and printed with
  `[retained]`; a cleared topic stays cleared
- [x] Broker restart (Mosquitto container restarted 15:50): all four connections reconnected by
  themselves (`mqtt_reconnects` = 1 each), statuses back to retained `online`, publishing
  resumed, retained `display` messages re-delivered with `[retained]`; no reboot

### AI messages (`…/display`)

- [x] Retained `{"text","ts"}` to each of the three `display` topics → banner under the right
  room on serial and in the panel's list, with `ts`; UTF-8 (°C, —) intact
- [x] Message to one node does **not** show under the others
- [x] Non-JSON payload → printed raw with a note
- [x] Payload over 512 bytes (711 B) → truncated, no crash. **Fixed:** the banner used to say
  "not JSON" (the cut breaks the JSON); it now says "payload over 512 bytes; truncated"
- [x] Empty retained payload (`-r -n`) → "display topic cleared" log

### Control panel

Tested 2026-09-27 from a laptop with a scripted run of the panel's HTTP API (46/46 passed) and
the page rendered in headless Edge.

- [x] Loads at the IP (74 ms) and at `http://intellithings-emu.local/` (Windows laptop, ~2.4 s
  for the mDNS lookup) — [ ] still to check on a phone
- [x] JavaScript runs in Chromium (headless Edge): no console errors; status, three room cards,
  12 shortcuts, custom-values table, message list and test form all render with live data —
  [ ] still to check Safari and a phone, and click through a few buttons by hand
- [x] System status values correct (IP, SSID/RSSI, broker, uptime, heap, clock, last command)
- [x] Every shortcut on its default room (rooms with no default tested on the kitchen): mode
  + time left correct; values move as expected (PM2.5 spike 6 → 45 in 30 s, hot 21.3 → 22.5 °C
  with presence, lights off 203 → 3 lux); "published N s ago" stays at 0–3 s
- [x] Shortcut with a custom duration; "All rooms" (until cleared); ↺ Back to normal; Reset room
  (also by room name)
- [x] Custom values: fixed value (min = max) becomes exact (CO₂ 1500); range + instant stays
  inside the range; 5 s duration expires back to auto; presence present/empty
- [x] Invalid input → HTTP 400 with a reason, nothing changes (min > max, out of sensor range,
  unknown metric/node/action/scenario, missing fields, negative duration, bad JSON, empty and
  over-2 KB bodies — the last gets its 400, then the socket is reset because httpd doesn't
  drain the body; the panel can't send that much). Red toast itself not seen yet (no clicks)
- [x] Send test AI message (HTTP `display` command) → published, appears in the panel list and
  on serial under the right room with a `ts`; with MQTT down it's rejected with "MQTT not
  connected; message not sent"
- [x] **Bug found and fixed:** once MQTT was up the panel stopped accepting connections
  (`httpd: error in accept (23)`). The 4 MQTT sockets left only ~3 of lwIP's default 10 sockets
  for HTTP, so idle browser keep-alive connections ran it out before httpd's LRU purge (at 7)
  could act. `CONFIG_LWIP_MAX_SOCKETS=16` in `sdkconfig.defaults`; with 12 idle keep-alive
  connections held open, fresh requests still answer in 27–57 ms

### MQTT control topic

- [x] Scenario command on `intellithings/emulator/cmd` works; `intellithings/emulator/state`
  updates 0.4–0.5 s later with `last_cmd` (source `mqtt`)
- [x] Invalid JSON / unknown node / unknown scenario → rejected in `last_cmd`
- [x] Retained command: re-delivered after a reconnect → ignored and logged. Note: a retained
  publish is still **executed once** by an already-connected emulator, because the broker
  forwards it to existing subscribers with the retain flag cleared (MQTT 3.1.1 §3.3.1.3)
- [x] Clearing it with `-r -n` is silent. **Fixed:** it used to be logged as "rejected: invalid
  JSON" and show up as the last command

### Home Assistant

- [ ] Three devices via the YAML example in `README.md`; availability follows `status`
- [ ] A cooking / stuffy / PM2.5 scenario triggers the HA → agent pipeline as expected

### Soak

- [x] Ran 1 h (15:36–16:36) with MQTT live, polled every 30 s: no reboots (uptime continuous to
  3631 s), panel answered all 121 polls, no Wi-Fi reconnects, free heap flat at ~137 KB, minimum
  free heap 111 KB (set in the first minute, never lower — including through the broker restart)

## Open items / known limitations

- MQTT schema is a **draft until Oct 4**. If field names change: update `METRIC_INFO` keys in
  `main/sim_model.c`, `docs/interfaces/mqtt.md` and `README.md`.
- AI message max length is part of the Oct 18 contract; the emulator currently accepts 400
  characters from the panel and truncates inbound payloads over 512 bytes.
- The real-node publish interval is still open; 3 s is emulator-only.
- No host unit tests (no host C compiler on the dev PC). The simulation is checked by review
  and on-device logs; `sim_model.c` and `payload.c` have no ESP-IDF calls, so host tests can
  be added later.
- The panel has no login; anyone on the LAN can drive the emulator. Fine for a test tool on
  the team network.
- Readings every 3 s × 3 nodes: consider excluding emulator entities from HA's recorder.
- Optional: use the onboard LEDs (GPIO12/13) for Wi-Fi/MQTT status.

## Git

- Committed on branch `ai/emulator`, cut from a local `dev` that was fast-forwarded to
  `main` (`origin/dev` was 3 commits behind and lacked PR #4). Not pushed yet. A PR from
  `ai/emulator` into `dev` also brings `dev` up to date with `main`.
- After the network tests pass, tick the boxes above and open the PR into `dev`.
- Gitignored and not to be committed: `sdkconfig`, `sdkconfig.local`, `build/`,
  `managed_components/`, `dependencies.lock`, `build.log`, `.vscode/`.
