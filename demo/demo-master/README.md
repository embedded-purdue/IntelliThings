# Demo master (ESP32-C3 + game buzzer)

Owner: **@spicybutter (PM)** · firmware **v1.0.0** · see
[the demo mode plan](../README.md).

The **demo master** sits on the ECE SPARK booth table between the Raspberry Pi 5 and the monitor.
It holds the demo home's **scenario library** and **randomizer**, reads the **game buzzer**,
publishes the demo data over MQTT and serves the **web interface** shown on the booth monitor.

## Hardware

| Part | Notes |
|---|---|
| **LuatOS ESP32-C3 Core** | Same board family as the emulator ESP32; Wi-Fi on the booth network. The dev board is the native-USB variant (USB-Serial/JTAG) |
| **Game buzzer** | An Amazon game buzzer on **2 × AA batteries**; a press makes its LED blink and plays a sound. Two wires go to the ESP32-C3: signal and GND (see below) |

**Pins used:**

| GPIO | Use |
|---|---|
| **3** | Buzzer input. Pressed = **high**, internal pull-down. **Bench test:** short **3V3 to IO03** |
| 12 | Onboard LED D4: blinks while the house is rolling (`CONFIG_DM_LED_GPIO`, -1 = off) |

## What it does

| Feature | Behaviour |
|---|---|
| **Demo home** | [`demo/demo_home.json`](demo/demo_home.json): the 9 areas of the AI team's [demo device inventory](../../ai/docs/demo-device-inventory.md) (home, master and second bedroom, kitchen, living room, bathroom, garage, office, laundry), 127 values: device and household states (lights, fans, AC, blinds, humidifier, purifier, oven, TV, vacuum, washer, dryer, garage door, car, front door…) and the full sensor set in every room |
| **Scenario library** | Starts from 56 built-in scenarios (the emulator's 50, adapted to the new rooms and devices, plus a few that use them). Up to **200**; add, edit and delete them on the admin page. Edits are saved in flash (`demolib` partition) |
| **Scenario loop** | Picks a random scenario (never the same twice), publishes it, waits the interval (default 90 s), picks the next. The source can also be random rolls, or a mix |
| **Buzzer press** | Stops the loop; every value rolls for **4 s** on the demo page, then lands on a **random home within the caps** and is published. Then a **hold** (default **30 s**, set on the admin page) while the AI reacts: presses are ignored. Then the loop resumes |
| **Checking the AI** | Every scenario has an **expected AI response** ("Turn on the kitchen purifier and fan…"), editable on the admin page. The admin page's **Now playing** card shows it next to a checklist worked out from the current values (also for random rolls) and to **what the AI actually said**: the demo master listens, read-only, to `intellithings/ai/summary`, `…/inference_status` and each node's `…/display`, and times each message against the scenario's start. The expectations are never published, so the agent can't see them |
| **Three ways to press** | The game buzzer (GPIO3), the **red button** on the demo page, and the **Space** key on the demo page |
| **Buzzer handling** | A quick tap counts: the press is caught on the edge and confirmed after **2 ms** of contact in total (or a burst of pulses, when the buzzer's signal follows its LED or sound). **One press per touch:** someone holding the buzzer rolls once; it re-arms after **200 ms** of quiet. While it's held, the red button on the demo page shows as pressed. A press during the lock makes the page shake and say how long until it unlocks, the same as a web press |
| **Random limits** | Every number has caps (never crossed) and a normal range; every bool a chance of true. Editable on the admin page, saved in flash. Published in °C |
| **MQTT** | One retained topic per area under `intellithings/demo/` (the prefix is a setting), plus the doorbell event. **Never calls the LLM**: HA automations decide when the agent runs |

## The web interface

Open `http://<board IP>/` (the IP is in the serial log and on the admin page). The board's
hostname is `intellithings-demo`.

- **`/` — demo page**, for the booth monitor (public):
  - every area of the demo home as a card: device states as chips (lit when on), sensor
    readings in a compact grid, abnormal values in amber, the three sensor node rooms marked.
    The whole home fits a 1920×1080 screen;
  - the current scenario, its group, the time of day, and a countdown to the next one;
  - a "Someone rang the doorbell" notice when the doorbell rings;
  - the big red **Roll the house** button (and **Space**); a rolling slot-machine animation,
    then the values land one after another;
  - after a roll: "Watch the AI react" and how long the button stays locked.
- **`/admin` — admin page**, behind a PIN (**default `1234`**). It's a speed bump for visitors,
  not security: the page and its API are on the open LAN.
  - **Now playing** (top of the overview): the running scenario, its expected AI response, a
    checklist from the current values (urgent / act / calm), the AI's thinking status, overall
    summary and room messages, each timed from the scenario's start (amber = written before it).
  - **Overview:** demo state, buzzer presses (total, kept across reboots, and by source since
    boot), doorbell rings, MQTT status and topics, device (uptime, memory, Wi-Fi), recent events.
  - **Demo control:** publishing on/off, loop on/off, interval and source; roll length, hold,
    spread and abnormal share; Roll now, Next step, Ring doorbell.
  - **Scenarios:** search, filter by group, play, edit (the expected AI response, "ring the
    doorbell", the values), delete, add; reset to the built-in library.
  - **Random limits:** caps and normal ranges per entity; reset all.
  - **Settings:** light/dark theme and animations for the demo page, °F/°C, the MQTT broker and
    topic prefix, the admin PIN, reboot.

Every setting is saved in NVS and survives a reboot; the loop and publishing settings are the
state the board boots into.

## MQTT

Follows the AI team's [demo device inventory](../../ai/docs/demo-device-inventory.md) (Nov 8):
**one topic per area**, under the prefix (default `intellithings/demo`). Still a **proposal**:
the proposed demo section of [`docs/interfaces/mqtt.md`](../../docs/interfaces/mqtt.md) predates
the inventory and needs syncing, with the Software and AI leads' review.

| Topic | QoS / retained | Payload |
|---|---|---|
| `<prefix>/status` | 1 / yes | `online` / `offline` (Last Will). Not in the inventory; kept so HA can tell the demo master is down |
| `<prefix>/home` | 1 / yes | Household states: `light_on`, `fan_on`, `front_door_locked`, `anyone_home`, `front_door_light_on`, `doorbell_motion`, `ts` |
| `<prefix>/<room>` | 1 / yes | Device states, then sensor readings, then `ts`. Rooms: `master_bedroom`, `second_bedroom`, `kitchen`, `living_room`, `bathroom`, `garage`, `office`, `laundry` |
| `<prefix>/home/doorbell` | 1 / **no** | `{"event":"doorbell_pressed","event_id":"boot-7-press-12","ts":"…"}`, once per ring |

Example, `intellithings/demo/kitchen`:

```json
{"light_on":true,"fan_on":false,"blinds_position_pct":80,"purifier_on":false,"window_open":false,
 "oven_on":false,"microwave_on":false,"temperature_c":22.4,"humidity_pct":41.2,"voc_index":112,
 "co2_ppm":684,"lux":312.5,"distance_mm":1180,"pm1_0_ugm3":3,"pm2_5_ugm3":5,"pm10_ugm3":7,
 "presence":true,"ts":"2026-11-15T12:00:00-05:00"}
```

- **When:** every area is published whenever the values change (a scenario step, or a roll
  landing), and again after a reconnect. Nothing is published during a roll.
- **`ts`** is left out until the clock has synced, as in the node contract.
- **Time of day** (`morning` / `afternoon` / `evening` / `night`) is internal: scenarios set it
  and it drives the fill rules (who's asleep, lights, blinds), and the demo page shows it, but
  it isn't published (it isn't in the inventory).
- **Doorbell:** rung when a scenario marked "doorbell" starts (built in: *Package at the door*,
  *Doorbell at night*), or with **Ring doorbell** on the admin page. `event_id` stays unique
  across reboots (boot count + press count).
- **Not published any more:** the emulator-style `state` and whole-house `home` payloads and the
  per-node `<node_id>/sensors` copies. The emulator's own demo stand-in still uses them.
- **Still open (inventory, "Integration still to settle"):** command topics and applying the
  AI's actions to the simulated home, target limits (AC, humidifier), timing, and how the room
  topics map to the bedroom / kitchen / living-room displays.

## Build and flash

**C on ESP-IDF v6.1**, like the emulator.

1. Copy `sdkconfig.local.example` to **`sdkconfig.local`** (gitignored) and fill in the Wi-Fi
   and broker login. **Never commit it.**
2. Build and flash (ESP-IDF v6.1 environment):

   ```bash
   idf.py build
   idf.py -p COM4 flash monitor
   ```

   After changing `sdkconfig.local` or `sdkconfig.defaults`, delete `sdkconfig` so the new
   defaults are picked up. The first flash (or a partition table change) needs `flash`, not
   `app-flash`.

**Version:** `PROJECT_VER` in `CMakeLists.txt` is the only place it's set.

### Editing the pages without reflashing

```bash
python tools/page_dev_server.py                      # built-in mock of the board's API
python tools/page_dev_server.py --board 192.168.1.5  # forward /api/* to a real board
```

Then open `http://localhost:8766/` and `/admin` (PIN `1234` on the mock). The pages are read from
`main/web/` on every request.

## HTTP API

| Endpoint | |
|---|---|
| `GET /api/state[?since=seq]` | State for the pages; `v` (values in entity order) only when they changed after `seq` |
| `GET /api/meta` | Areas, entities (limits, options), groups, scenario list |
| `POST /api/press` | A press from the page: `{"src":"web"}` or `{"src":"key"}` |
| `POST /api/admin/login` | Checks the PIN |
| `GET /api/admin/status[?after=id]` | Overview data and events after `id` |
| `POST /api/admin/cmd` | `{"op":"publish","on":true}`, `loop` (`on`, `interval_s`, `source`, `mix_pct`), `settings` (`roll_s`, `hold_s`, `spread`, `abnormal_pct`), `roll`, `play` (`id`), `next`, `doorbell`, `republish`, `reset_counts` |
| `GET /api/admin/scenario?id=n` | One scenario with its values |
| `POST /api/admin/scenario` | `{"id":n or null,"name":"…","group":"air","expect":"…","doorbell":false,"set":{"kitchen.pm2_5_ugm3":160,…}}` |
| `POST /api/admin/scenario/delete` | `{"id":n}` |
| `POST /api/admin/library/reset` | Built-in library and limits |
| `POST /api/admin/limit` | `{"key":"kitchen.temperature_c","min":15,"max":38,"nmin":20,"nmax":25}` |
| `POST /api/admin/limits/reset` | Built-in limits |
| `POST /api/admin/settings` | Any of `host`, `port`, `username`, `password`, `keepalive_s`, `prefix`, `theme`, `animation`, `fahrenheit`, `pin` |
| `POST /api/admin/settings/reset`, `/api/admin/reboot` | |

Admin endpoints need the header `X-Admin-Pin`.

## Game buzzer wiring

The buzzer is a self-contained, battery-powered toy, so the plan is to **find a point on its PCB
that carries a voltage when it's pressed** and wire it to the ESP32-C3: **signal → GPIO3,
buzzer GND → ESP32 GND**. Things to check when testing the board:

- **Measure the voltage first** with a multimeter. 2 × AA is about **3.0–3.2 V** with fresh
  cells, just inside the ESP32-C3's 3.3 V logic. **Never feed it more than 3.3 V.** If the point
  you pick is higher, use a resistor divider.
- **Connect the grounds** (buzzer GND to ESP32 GND), or the signal has no reference.
- **Put a ~1 kΩ resistor in series** with the signal wire, to protect both boards if something
  is wired wrong.
- **Don't connect the buzzer's battery + to the ESP32's 3.3 V,** so the two supplies never fight.
- **The press may look like several pulses** if the tapped point follows the blinking LED or the
  sound, and **a quick tap may be only a few milliseconds long** (a multimeter averages, so it
  can show well under 3 V for a signal that peaks at 3 V). The firmware catches the edge with an
  interrupt, counts the press after 2 ms of contact in total within 20 ms (or 2 more pulses),
  and re-arms only after 200 ms of quiet, so a long hold counts once. Presses during the roll and
  the hold are ignored. Tune with `CONFIG_DM_BUTTON_MIN_PRESS_MS` (raise it if noise on the wire
  starts rolls) and `CONFIG_DM_BUTTON_RELEASE_MS`. The admin page shows the input (held, for how
  long) and how many edges were too short to count.
- **The ESP32-C3 reads high above about 2.5 V** (0.75 × 3.3 V). The buzzer's ~3 V is enough with
  fresh cells; if presses go missing as the batteries run down, measure the peak (not the
  average) and consider a transistor or comparator stage.
- **The best point is usually the switch contact itself** (a clean high/low), rather than the
  LED or speaker drive, which pulse.
- If the point that works is **low when pressed**, turn off `CONFIG_DM_BUTTON_ACTIVE_HIGH`
  (internal pull-up instead).

Avoid the C3's strapping pins (GPIO2, 8, 9), the onboard LEDs (GPIO12/13) and, on the native-USB
board variant, the USB pins (GPIO18/19).

## Contracts

The demo topics and room payloads follow the AI team's
[demo device inventory](../../ai/docs/demo-device-inventory.md). They're still a proposal: the
demo section of [`docs/interfaces/mqtt.md`](../../docs/interfaces/mqtt.md) needs syncing with the
inventory, and the caps and normal ranges here are a draft.

## Layout

```
demo/demo-master/
├── CMakeLists.txt          project; PROJECT_VER
├── partitions.csv          4 MB: nvs, factory app (2 MB), demolib (64 KB)
├── sdkconfig.defaults      tracked defaults (no secrets)
├── sdkconfig.local.example template for the gitignored sdkconfig.local
├── demo/demo_home.json     areas, entities, caps and the built-in scenarios (per the inventory)
├── tools/
│   ├── gen_demo.py         demo_home.json -> C tables at build time (validates them)
│   ├── gzip_asset.py       gzips the pages at build time
│   └── page_dev_server.py  serves the pages with a mock API, or forwards to a board
└── main/
    ├── main.c              start-up
    ├── demo_home.c         fill, roll and derive rules; runtime limits
    ├── library.c           the scenario library and limits in flash
    ├── engine.c            loop, roll, hold, doorbell, publishing per area
    ├── mqtt_link.c         the MQTT connection (Last Will on <prefix>/status)
    ├── ai_watch.c          read-only view of the AI's summary, thinking status and room messages
    ├── button.c            buzzer input and LED
    ├── settings.c          NVS settings and the press counter
    ├── web_server.c        pages and the JSON API
    ├── event_log.c         recent events for the admin page
    ├── wifi_manager.c, time_sync.c, json_writer.c   from the emulator
    └── web/demo.html, web/admin.html
```
