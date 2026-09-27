# IntelliThings — ESP32-C6 Firmware Architecture (Rust + ESP-IDF)
*Software subteam (lead @LukeTuthill) | Last updated Sept 26, 2026*

> Detailed firmware design for the sensor nodes. Summary and current decisions live in `docs/Project_Guideline.md` §4; team workflow in `docs/Collaboration_Guidelines.md` §2. If anything here disagrees with the Project Guideline, the guideline wins.
>
> **Node hardware this firmware targets:** ESP32-C6 · 5 I2C sensors (SHT41, SGP40, SCD41, BH1750, VL53L0X) · PMS5003 on UART · LD2410C OUT pin on GPIO · **DWIN DMG80480T050_09WN 5" display over UART (DGUS)** · **WS2812B LED bar via RMT** · MQTT only (no Matter).

---

## 1. Toolchain (confirmed Sept 26 against [esp-rs/esp-idf](https://github.com/esp-rs/esp-idf))

| Item | Version / setting |
|---|---|
| ESP-IDF | **v6.1.0** → `ESP_IDF_VERSION = "v6.1"` (ESP-IDF's tag for 6.1.0) |
| `esp-idf-sys` | **0.38.1** |
| `esp-idf-hal` | **0.47.0** |
| `esp-idf-svc` | **0.53.0** |
| Rust | **nightly + `rust-src`** — `std` for `riscv32imac-esp-espidf` is built from source (`-Zbuild-std=std,panic_abort`). Minimum Rust 1.82. The exact nightly date is pinned in `rust-toolchain.toml` after the first clean build (Sep 27). |
| Target | `riscv32imac-esp-espidf` (ESP32-C6) |

These are the first crate releases with ESP-IDF 6.1 compatibility fixes; the esp-rs CI builds the C6 target against v6.1 in its nightly runs.

**ESP-IDF v6 changes that affect us:**
- **MQTT moved out of ESP-IDF** into a managed component. Add to `Cargo.toml`:
  ```toml
  [[package.metadata.esp-idf-sys.extra_components]]
  remote_component = { name = "espressif/mqtt", version = "1.*" }
  ```
  The esp-rs CI tests this on v6.0 / ESP32-C3 only — verify it on the C6 rig first thing.
- `esp-idf-hal`'s timer drivers aren't available on ESP-IDF 6+ (not needed here — use `std::thread::sleep` / `std::time`).

**Starter config:**
```toml
# .cargo/config.toml
[build]
target = "riscv32imac-esp-espidf"
[target.'cfg(target_os = "espidf")']
linker = "ldproxy"
[unstable]
build-std = ["std", "panic_abort"]
[env]
ESP_IDF_VERSION = "v6.1"

# rust-toolchain.toml
[toolchain]
channel = "nightly"          # replace with the pinned nightly-YYYY-MM-DD
components = ["rust-src"]
```

**FreeRTOS is underneath `std`:** `std::thread::spawn` becomes a FreeRTOS task, and `std::sync::{Mutex, mpsc, Arc}` work normally. Plain `std` covers task creation, IPC and shared state; raw FreeRTOS primitives via `esp-idf-sys` are only for edge cases.

**Driver crates:**
- Sensors (`embedded-hal`-based, used with `esp-idf-hal`'s `I2cDriver`): `sht4x`, `sgp40`, `vl53l0x`, `bh1750`, `scd4x`. **Check each against `esp-idf-hal` 0.47 / `embedded-hal` 1.0** before pinning; a hand-rolled register read is a fine fallback — all five have simple, documented I2C register maps.
- LED bar: RMT via a crate such as `ws2812-esp32-rmt-driver` / `smart-leds` (check against `esp-idf-hal` 0.47), or hand-rolled RMT bit timing.
- Display: no crate needed — the DWIN DGUS frame format is simple enough to write directly over `esp-idf-hal`'s UART driver.
- Serialization: `serde` + `serde_json` for MQTT payloads.

**Learning resources:** [Rust on ESP docs](https://docs.espressif.com/projects/rust/) · [The Rust on ESP Book](https://docs.espressif.com/projects/rust/book/) (start here) · [esp-rs/esp-idf repo](https://github.com/esp-rs/esp-idf)

---

## 2. Task / Thread Layout

One RISC-V core, so the goal is that no task blocks the others for long (PMS5003 warm-up, Wi-Fi reconnects and MQTT publishes are the main blocking risks).

| # | Thread | Responsibility | Approx. period | Priority | Stack |
|---|---|---|---|---|---|
| 0 | `main` | Boot sequence, NVS, Wi-Fi, MQTT init, spawns the rest, then a supervisor/reconnect loop | once, then loop | default | default |
| 1 | `sensor_i2c` | Polls SHT41 → SGP40 (with SHT41 compensation) → VL53L0X → BH1750 → SCD41 on the shared I2C bus; writes shared state | 2–5 s (SCD41 updates ~every 5 s) | normal | ~4–6 KB |
| 2 | `pms5003_uart` | Reads PMS5003 binary frames (start bytes `0x42 0x4D`, checksum-verified); optionally sleeps the fan via SET pin between reads | 10–30 s (~30 s warm-up if sleeping) | normal | ~2–3 KB |
| 3 | `presence_gpio` | LD2410C **OUT** pin — GPIO interrupt on both edges with debounce, or a 200–500 ms poll | event / poll | normal | ~1–2 KB |
| 4 | `mqtt_publish` | Snapshots shared state, publishes the JSON snapshot — interval or on-change | 5–15 s or on-change | normal | ~3 KB |
| 5 | MQTT callback | Receives the retained AI message on `.../display`, writes it into shared state (not its own thread — `esp-idf-svc`'s client delivers via callback) | event | — | — |
| 6 | `dwin_display` | Writes sensor values to DGUS VP addresses continuously; rewrites the AI section only when a new message arrives | 0.5–1 s | normal | ~2 KB |
| 7 | `led_bar` | Drives the WS2812B bar via RMT with a brightness cap; patterns follow the LED bar team decision | ~30–50 ms animation tick | low | ~1–2 KB |

Total: 6 spawned threads + 1 MQTT callback + `main`.

---

## 3. Shared State & Concurrency

A few `Arc<Mutex<…>>` structs, since most consumers only want the latest value:

```rust
struct SensorSnapshot {
    temp_c: f32, humidity_pct: f32,        // SHT41
    voc_index: u16,                         // SGP40
    co2_ppm: u16,                           // SCD41
    lux: f32,                               // BH1750
    distance_mm: u16,                       // VL53L0X
    pm1_0: u16, pm2_5: u16, pm10: u16,      // PMS5003
    presence: bool,                         // LD2410C OUT
    last_updated: Instant,
}

enum SystemStatus {
    WifiConnecting,
    WifiConnected,
    MqttConnected,
    MqttDisconnected,
    SensorFault(&'static str),   // e.g. "pms5003_timeout", "i2c_nack"
    Normal,
}

struct AiMessage {
    text: String,
    ts: String,        // from the payload, shown in the final GUI
    is_new: bool,      // display rewrites the AI section only when set
}

struct SharedState {
    sensors: Mutex<SensorSnapshot>,
    status: Mutex<SystemStatus>,
    ai_message: Mutex<Option<AiMessage>>,   // persists until replaced — no timeout
}
```

Field names above are internal; the **published JSON field names follow the MQTT schema contract fixed Oct 4** (shared with the AI team and its emulator ESP32).

Wrap once in `Arc<SharedState>` and clone into each thread. Use `mpsc` only where order matters (e.g. discrete presence-change events). The I2C driver sits in `Arc<Mutex<I2cDriver>>` so a second consumer can be added later without refactoring.

---

## 4. Display — DWIN DGUS over UART

- **UART1, 115200 baud, 8N1.** The display's UART2 must be set to **TTL mode** before wiring (RS232 levels would damage the ESP32 pins).
- The screens are designed in DWIN's **DGUS** tool and loaded from an SD card. The ESP32 only writes values into display variables (VP addresses): `5A A5 <len> 82 <VP addr hi> <VP addr lo> <data…>`.
- **Layout rule:** all environmental readings are always on screen; a **dedicated AI section** shows the latest AI message and **keeps it until a new one arrives** — no timeout, no page switching.
- **Dev GUI** (simple readouts + connection status + AI box) first, **final GUI** later. The **VP address map** is shared between whoever designs screens and whoever writes the driver — keep it in `docs/interfaces/`.
- Text for the AI section must respect the max length agreed with the AI team (contract due Oct 18).

---

## 5. LED Bar — WS2812B via RMT

- One GPIO (not a strapping pin), RMT peripheral, 3.3 V → 5 V level shifter on the final PCB (try direct drive on the rigs first).
- **Global brightness cap** in firmware — at full white each LED can draw ~60 mA, and the bar dominates the 5 V budget.
- **What the bar shows is a 🗳 TEAM DECISION:** A. status only · B. AI feedback · C. air-quality gauge · D. mix. The driver should expose a small pattern API (`solid`, `pulse`, `gauge(level)`, `flash`) so any option is a configuration change, not a rewrite.
- If option A or D is chosen, the earlier status mapping still works: blue pulse = Wi-Fi connecting, cyan = MQTT connecting, dim green = normal, purple = AI message received, yellow = sensor fault, orange = MQTT lost, red = Wi-Fi lost.

---

## 6. MQTT Topics

| Topic | Direction | Content |
|---|---|---|
| `intellithings/<node_id>/sensors` | publish | One JSON snapshot of all readings |
| `intellithings/<node_id>/status` | publish (retained, Last Will) | `online` / `offline` |
| `intellithings/<node_id>/display` | subscribe (HA publishes **retained**) | `{"text": "...", "ts": "2026-10-18T14:05:00-04:00"}` — latest AI message; retained so a rebooted node re-shows it |

`<node_id>` comes from the C6's efuse MAC, so one firmware image fits every node. The AI team's emulator ESP32 publishes as `emu-1..3` on the same topics and schema.

---

## 7. Boot Sequence (`main`)

1. `esp_idf_svc::sys::link_patches()` + logger init.
2. NVS init — Wi-Fi credentials and broker address live here, not in code.
3. System event loop + Wi-Fi (`BlockingWifi`), updating `SystemStatus` as it goes.
4. Init I2C bus, UART0 (PMS5003, 9600), UART1 (DWIN, 115200), GPIO input (LD2410C OUT), RMT (LED bar).
5. MQTT client (`EspMqttClient`, from the `espressif/mqtt` managed component): Last Will on `.../status`, connect, subscribe to `.../display`, publish `online`.
6. Spawn the worker threads from §2 with clones of `Arc<SharedState>`; exclusively-owned drivers (UARTs, GPIO, RMT) move into their thread.
7. `main` becomes the supervisor loop: watches Wi-Fi/MQTT health and drives reconnects.

---

## 8. Error Handling

- Every sensor read returns `Result`; tolerate N consecutive failures (small retry with backoff) before setting `SensorFault(...)`, so one dropped transaction doesn't flap the status.
- Wi-Fi/MQTT disconnects surface through the event loop/callback; the supervisor re-drives connection state and updates `SystemStatus`.
- 8 MB flash leaves room for OTA (`ota_0`/`ota_1`); whether OTA is in scope this semester is still open (see §10).

---

## 9. Module Layout

```
software/firmware/src/
├── main.rs            boot, start threads, supervisor loop
├── hardware/
│   ├── bus.rs         I²C / UART setup
│   └── sensors/       sht41.rs  sgp40.rs  scd41.rs  bh1750.rs  vl53l0x.rs  pms5003.rs  ld2410c.rs
├── network/
│   ├── wifi.rs
│   └── mqtt.rs
├── ui/
│   ├── display.rs     DWIN DGUS driver
│   └── led.rs         WS2812B LED bar (RMT)
└── system/
    ├── state.rs       shared state
    ├── config.rs      NVS settings
    ├── logging.rs
    └── errors.rs
```

**Baseline firmware first** (target merged by Oct 4): boot → Wi-Fi → MQTT → publish a placeholder snapshot in the agreed schema. Then module tasks are assigned and built in parallel. Final-PCB bring-up (Nov 15) should only need pin-map changes.

---

## 10. Open Items

| Item | Status |
|---|---|
| Pinned nightly date | Set on Sep 27 after the first clean build |
| MQTT managed component on C6 + v6.1 | Verify on the rig Sep 27 |
| Third-party crates vs. `esp-idf-hal` 0.47 | Check each sensor/LED crate as it's added |
| What the LED bar shows | 🗳 Team decision (Project Guideline §10) |
| PMS5003 SET-pin sleep vs. always-on fan | Decide with Hardware by Oct 4 (affects a GPIO + warm-up delay) |
| Publish cadence (interval vs. on-change) | Decide with AI team before the Oct 18 MCP tests |
| OTA this semester or later | Decide before the baseline is finalized |

---

## Sources

- [esp-rs/esp-idf – esp-idf-sys / -hal / -svc (repo, CI, changelogs)](https://github.com/esp-rs/esp-idf) · [esp-idf-sys](https://crates.io/crates/esp-idf-sys) · [esp-idf-hal](https://crates.io/crates/esp-idf-hal) · [esp-idf-svc](https://crates.io/crates/esp-idf-svc)
- [Espressif – Rust on ESP](https://docs.espressif.com/projects/rust/) · [The Rust on ESP Book](https://docs.espressif.com/projects/rust/book/)
- [DWIN – DMG80480T050_09W specifications](https://www.dwin-global.com/5-0-inch-hmi-tft-lcd-modeldmg80480t050_09windustrial-product/)
