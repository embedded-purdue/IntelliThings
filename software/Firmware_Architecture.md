# IntelliThings — ESP32-C6 Firmware Architecture (`no_std` Rust)
*Software subteam (lead @LukeTuthill) | Draft rewrite, Oct 18, 2026 — **for @LukeTuthill's review***

> Detailed firmware design for the sensor nodes, rewritten for the bare-metal `no_std` stack the
> team chose on Sep 27 (the earlier version described Rust `std` on ESP-IDF).
> - **Current** = what's in `software/firmware/` today.
> - **Proposed** = a suggestion for the Software lead to accept, change or drop.
>
> Current decisions are summarized in `docs/Project_Guideline.md` §4; workflow in
> `docs/Collaboration_Guidelines.md` §2; build steps in `software/README.md` and
> `software/firmware/README.md`. If anything here disagrees with the Project Guideline, the
> guideline wins.
>
> **Node hardware:**
> - ESP32-C6-DevKitC-1 (on sockets in the final PCB).
> - 5 I2C sensors (SHT41, SGP40, SCD41, BH1750, VL53L0X).
> - PMS5003 on UART; LD2410C OUT pin on GPIO.
> - **DWIN DMG80480T050_09WN 5" display over UART** (custom DGUS UI, VP map).
> - **WS2812B LED bar** driven by the ESP32.
> - MQTT only (no Matter).

---

## 1. Toolchain (current)

| Item | Version / setting |
|---|---|
| Runtime | Bare-metal **`no_std`**, not an ESP-IDF/FreeRTOS application; generated from the `esp-generate` template |
| HAL | `esp-hal` ~1.2.2 (`unstable` features enabled — experimental APIs, not a nightly compiler) |
| RTOS / async | `esp-rtos` 0.4.0 with Embassy (`embassy-executor` 0.10.0, `embassy-time` 0.5.0) |
| Networking | `embassy-net` 0.9.1 on `smoltcp` 0.14; Wi-Fi through `esp-radio` 1.0.0-beta.1 |
| Heap | `esp-alloc` 0.11 (64 KiB heap in `main`) |
| Rust | **stable** + `rust-src`, minimum **1.95**, edition 2024 (`rust-toolchain.toml`) |
| Target | `riscv32imac-unknown-none-elf` |
| Flash / monitor | `cargo run` → `espflash flash --monitor --chip esp32c6` |
| Package | `intelli_things` |

- **Check every new crate** for `no_std`, the ESP32-C6 and compatibility with the current
  `esp-hal` and Embassy versions before adding it.
- **MQTT:** no client is configured yet. **Proposed:** pick a `no_std` async MQTT client that
  runs on `embassy-net` (for example `rust-mqtt`), and check it supports Last Will, retained
  publishes and QoS 1 subscribe, which the [MQTT contract](../docs/interfaces/mqtt.md) needs.
- **Socket budget:** `StackResources<2>` today (DHCP + one more). MQTT plus DNS/SNTP will need a
  larger value.

---

## 2. Tasks (current → proposed)

**Current:** one `main` task initializes the hardware, then reads the five I2C sensors in turn
once a second and prints them. `wifi.rs` provides Wi-Fi + DHCP + a ping smoke test (with a
background `network_task`), but the call in `main` is commented out while sensor work goes on.

**Proposed:** one Embassy task per job. The C6 has a single core, so the rule is that **no task
blocks for long**; every wait is an `.await`.

| Task | Job | Period | Owns |
|---|---|---|---|
| `main` | Init, spawn the tasks, then supervise (Wi-Fi/MQTT state) | once, then loop | — |
| `network_task` (exists) | Runs the `embassy-net` stack | continuous | Wi-Fi interface |
| `i2c_sensors` | SHT41 → SGP40 (with SHT41 compensation) → SCD41 → BH1750 → VL53L0X | 1–5 s (SCD41 updates ~every 5 s) | I2C bus |
| `pms5003` | Read and checksum PMS5003 frames (`0x42 0x4D`) | 10–30 s | its UART |
| `presence` | LD2410C OUT pin, edge-triggered with debounce | on change | its GPIO |
| `mqtt` | Connect with Last Will, publish `…/sensors` + `…/status`, subscribe to `…/display` | interval TBD (contract: not yet agreed) | the MQTT socket |
| `display` | Write the VP map: sensor block every 0.5–1 s, AI message only when it changes | 0.5–1 s | the display UART |
| `led_bar` | Patterns with a global brightness cap | ~30–50 ms tick | the LED data pin |

Peripherals that only one task uses (UARTs, GPIOs, the LED pin) **move into that task**.

---

## 3. Shared state (proposed)

Tasks never call each other; they share the **latest values** through a few
`embassy_sync` primitives in `static`s:

- **Latest reading of each sensor:** e.g. a `Mutex<CriticalSectionRawMutex, SensorSnapshot>`,
  written by the sensor tasks and read by `mqtt` and `display`. A missing or failed sensor is
  `None`, so the display can show *Waiting* (VP value `0x8000`) and MQTT can skip or flag it.
- **Connection state** (Wi-Fi, MQTT): a `Watch` or `Signal` that `display` and `led_bar` follow.
- **Latest AI message** (`text`, `ts`): set by `mqtt` when a `…/display` message arrives,
  read by `display`. It is **kept until a different message arrives** (no timeout), as the
  layout rule requires.

```
Sensor tasks ──► latest readings ──┬──► mqtt (publish …/sensors)
                                   └──► display (VP sensor block)
mqtt (…/display arrives) ──► latest AI message ──► display (VP message block)
Wi-Fi / MQTT state ──► display (VP connection block) · led_bar
```

**Current:** the I2C sensors share the bus through a `RefCell` inside the one `main` task, and
each borrow is released before the next driver call. The VL53L0X driver holds a bus handle
for its whole lifetime, which is why the `RefCell` approach was needed. **Proposed:** if the
sensors move into their own task, keep the bus in that one task, or use an
`embassy-embedded-hal` shared-bus wrapper.

---

## 4. Display — DWIN over UART

- **Contract:** [`docs/interfaces/display-vp-map.md`](../docs/interfaces/display-vp-map.md)
  (v1.0, display UI by @rakkicow).
- **Wiring:** UART at **115200 8N1, 3.3 V TTL**. The display must be in **TTL mode** (done on the
  rigs).
- **No graphics on the ESP32.** The UI is custom DGUS structures by @rakkicow (not DWIN's DGUS
  tool; LVGL dropped). The ESP32 only writes VP frames:
  `5A A5 <len> 82 <VP hi> <VP lo> <data…>`. The display answers `5A A5 03 82 4F 4B`.
- **Minimum for a working node:**

  | Block | VPs | When |
  |---|---|---|
  | Sensor readings | `0x1000–0x1009`, one 10-word frame | Every 0.5–1 s. `0x8000` = no reading; temperature/humidity × 10 |
  | Connection | `wifi`, `hub`, `synced_ago` (`0x1010–0x1012`) | On change |
  | Clock | `unix_time` + `utc_offset` (`0x1020–0x1022`) | Once after SNTP |
  | AI message | `message_text` (`0x1100`, up to 512 bytes in 3 frames) + `message_ts` (`0x10FE`) | Only when a different message arrives |
  | Reset | `inputs_reset` (`0x103F` = `0x5AA5`) | At boot |

- **Not used:** the rooms page and the Wi-Fi setup page. Whether to drive the other pages
  (alert, night, away, boot, toast, action chips, "thinking") is an **open discussion** with
  @rakkicow.
- **AI message length:** still to be **tested on the real display**. The pipeline sends room
  messages of at most 60 characters today.
- **Units:** MQTT carries °C. The display's `units` VP (`0x1030`) selects °F/°C on screen.

---

## 5. LED bar — WS2812B

- **The ESP32 drives the bar** on one GPIO (not a strapping pin). How (e.g. the RMT peripheral
  through `esp-hal`) is still being worked out with Hardware.
- **Global brightness cap in firmware:** each LED can draw ~60 mA at full white, and the bar
  dominates the 5 V budget.
- **What the bar shows is a 🗳 team decision** (status / AI feedback / air-quality gauge / mix).
  **Proposed:** a small pattern API (`solid`, `pulse`, `gauge(level)`, `flash`), so any choice is
  configuration, not a rewrite.

---

## 6. MQTT (contract summary)

Full list of topics and payloads: [`docs/interfaces/mqtt.md`](../docs/interfaces/mqtt.md).

| Topic | Direction | Content |
|---|---|---|
| `intellithings/<node_id>/sensors` | publish, QoS 0 | One JSON snapshot of all readings (°C), `ts` once SNTP has synced |
| `intellithings/<node_id>/status` | publish, retained, **Last Will** | `online` / `offline` |
| `intellithings/<node_id>/display` | subscribe, QoS 1 (published retained) | `{"text": "...", "ts": "..."}` |

- **`<node_id>` comes from the chip's MAC,** so one firmware image fits every node. Each real
  node ID must be added to the AI pipeline's `ai/config/display_nodes.json` to receive room
  messages.
- **Broker:** `192.168.1.2:1883`, login required.

---

## 7. Boot sequence (proposed)

1. `esp_hal::init`, heap, `esp_rtos::start`.
2. Display UART up; write `inputs_reset`; boot checklist steps (optional, see §4).
3. Wi-Fi (`esp-radio`) + DHCP; spawn `network_task`.
4. SNTP for the clock (needed for `ts` and the display clock).
5. MQTT connect with Last Will → publish `online` → subscribe to `…/display`.
6. Spawn the sensor, display, LED and presence tasks.
7. `main` supervises: reconnects Wi-Fi/MQTT and updates the connection state.

**Wi-Fi settings:** compiled in from the local, gitignored `.cargo/esp-config.toml` (`WIFI_SSID`,
`WIFI_PASSWORD`). Accepted for the prototype; no on-device Wi-Fi setup is planned.

---

## 8. Error handling

- **Current:** every sensor read returns a `Result`; errors are printed and the loop continues.
  The SCD41 start is retried each cycle until it succeeds.
- **Proposed:** tolerate N consecutive failures (small retry with backoff) before marking a
  sensor as faulted, so one dropped transaction doesn't flap the reading; a faulted sensor shows
  as `0x8000` on the display.
- **Proposed:** Wi-Fi and MQTT drops are handled by `main`'s supervisor loop with backoff.
- 8 MB flash leaves room for OTA; whether OTA is in scope is still open.

---

## 9. Code layout

```
software/firmware/src/
├── main.rs            init, (today) the sensor loop
├── lib.rs             module exports
├── wifi.rs            Wi-Fi, DHCP, network_task, ping smoke test
└── sensors/           one file per sensor: sht41.rs  sgp40.rs  scd41.rs  bh1750.rs  vl53l0.rs
                       (+ mod.rs, sensor.rs: a shared Sensor trait, work in progress)
```

**Proposed additions:** `mqtt.rs`, `display.rs` (VP writes; the drop-in `dwin_vp.rs` constants
from @rakkicow), `led.rs`, `state.rs` (the shared state in §3), and `pms5003.rs` / `ld2410c.rs`
under `sensors/`.

---

## 10. Open items

| Item | Status |
|---|---|
| MQTT client crate for `no_std`/Embassy | To choose and verify on the rig |
| Pin assignments for the display UART, PMS5003, LD2410C, LED data | TBD with Hardware ([`pin-map.md`](../docs/interfaces/pin-map.md)) |
| How to drive the LED strip | Hardware/Software to-do |
| Which display pages to drive | Open discussion with @rakkicow |
| AI message length on the display | Needs testing |
| Publish cadence (interval vs. on-change) | Not agreed yet |
| PMS5003 SET-pin sleep vs. always-on fan | Decide with Hardware |
| OTA this semester or later | Open |

---

## Sources

- [esp-rs/esp-hal](https://github.com/esp-rs/esp-hal) · [Embassy](https://embassy.dev/) ·
  [The Rust on ESP Book](https://docs.espressif.com/projects/rust/book/) ·
  [Rust on ESP docs](https://docs.espressif.com/projects/rust/)
- [DWIN – DMG80480T050_09W specifications](https://www.dwin-global.com/5-0-inch-hmi-tft-lcd-modeldmg80480t050_09windustrial-product/)
