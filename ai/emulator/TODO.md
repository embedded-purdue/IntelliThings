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

> The board currently has the offline build flashed (no credentials). Reflash after creating
> `sdkconfig.local`.

## To test at the next team session

### Setup

- [ ] Home Assistant Mosquitto add-on running; create an MQTT login for the emulator
- [ ] Copy `esp32_c3_emulator/sdkconfig.local.example` → `sdkconfig.local`; fill in the
  2.4 GHz Wi-Fi and broker host/port/user/password
- [ ] `del sdkconfig` (or `rm sdkconfig`), then `idf.py build` and `idf.py -p COM8 flash monitor`
- [ ] On a laptop: `mosquitto_sub -h <broker> -u <user> -P <pass> -t 'intellithings/#' -v`

### Wi-Fi

- [ ] Connects; serial shows IP and the control-panel URL
- [ ] Reconnect: turn the AP off/on (or move out of range) → retries with backoff (1 s → 30 s),
  reconnects by itself; panel's "Wi-Fi reconnects" goes up
- [ ] Clock syncs (serial `Clock synced: …`; payloads gain `ts`; panel shows local time)

### MQTT — node contract

- [ ] Four connections come up: three nodes + emulator (`connected` lines on serial)
- [ ] `intellithings/<id>/status` = `online` (retained) for all three, and
  `intellithings/emulator/status` = `online`
- [ ] Each node publishes `…/sensors` every 3 s, even when readings haven't changed
- [ ] Payload fields and types match `docs/interfaces/mqtt.md`
- [ ] Last Will: unplug the board → all four statuses flip to `offline` within ~25 s
- [ ] Broker restart: nodes reconnect, re-publish `online`, resubscribe; retained `display`
  messages are re-delivered and print with `[retained]`

### AI messages (`…/display`)

- [ ] `mosquitto_pub -t intellithings/emu-bedroom/display -r -q 1 -m '{"text":"…","ts":"…"}'`
  → banner under **BEDROOM** on serial and in the panel; repeat for kitchen and living room
- [ ] Message to one node does **not** show under the others
- [ ] Non-JSON payload → printed raw with a note
- [ ] Payload over 512 bytes → truncated with a note, no crash
- [ ] Empty retained payload (`-r -n`) → "display topic cleared" log

### Control panel

- [ ] Loads at the IP and at `http://intellithings-emu.local/` (check a phone too; some
  Android/Windows setups don't resolve `.local`)
- [ ] **JavaScript never ran yet** (no Node.js on the dev PC) — check the browser console for
  errors in Chrome, Safari and a phone
- [ ] System status values correct (IP, SSID/RSSI, broker, uptime, heap, clock, last command)
- [ ] Every shortcut on its default room: mode badge + time left shown; values move as
  expected; "published N s ago" stays under ~3 s
- [ ] Shortcut with a custom duration; "All rooms"; ↺ Back to normal; Reset room
- [ ] Custom values: fixed value (min = max) becomes exact; range + "jump instantly"; duration
  expiry returns to auto; presence present/empty
- [ ] Invalid input (min > max, out of sensor range) → red toast, nothing changes
- [ ] Send test AI message → appears in the panel list and on serial under the right room

### MQTT control topic

- [ ] `mosquitto_pub -t intellithings/emulator/cmd -q 1 -m '{"node":"emu-kitchen","action":"scenario","name":"cooking"}'`
  works; `intellithings/emulator/state` updates within ~1 s with `last_cmd`
- [ ] Invalid JSON / unknown node / unknown scenario → rejected in `last_cmd`
- [ ] Retained command (`-r`) is ignored and logged (then clear it with `-r -n`)

### Home Assistant

- [ ] Three devices via the YAML example in `README.md`; availability follows `status`
- [ ] A cooking / stuffy / PM2.5 scenario triggers the HA → agent pipeline as expected

### Soak

- [ ] Leave it running 1 h+; panel's "Free heap (min)" stays stable, no reboots on serial

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
