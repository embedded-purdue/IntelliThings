# Emulator — progress and test plan

Owner: @spicybutter · last updated 2026-09-27

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
> `homeassistant.local`, which doesn't exist until the Mosquitto add-on is set up; if the Pi
> doesn't resolve by that name, put its IP in `sdkconfig.local` and rebuild.

## To test at the next team session

### Setup

- [x] Home Assistant Mosquitto add-on running on the Pi at `192.168.1.2:1883`
  (`homeassistant.local` doesn't resolve on the team network — use the IP). The emulator
  currently logs in as the add-on's internal `homeassistant` user — [ ] create a dedicated
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
- [x] Broker restart (Mosquitto add-on restarted 15:50): all four connections reconnected by
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
