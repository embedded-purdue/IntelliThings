# Software — Firmware Subteam

Lead: **@LukeTuthill** · 9 people · owns **Rig S**

ESP32-C6 firmware for the sensor nodes, written in **Rust (`std`, nightly) on ESP-IDF
v6.1.0**. Full design:
[`Firmware_Architecture.md`](Firmware_Architecture.md).

## Scope

- Firmware structure, shared state and MQTT scaffolding — the **baseline firmware** first
- I2C drivers: SHT41, SGP40 (VOC Index, compensated with SHT41 data), SCD41, BH1750, VL53L0X
- PMS5003 UART frame parsing; LD2410C presence on a GPIO
- **DWIN DGUS display driver** — dev GUI first, then the final GUI, plus the VP address map
- **WS2812B LED bar** via RMT, with a brightness cap
- Wi-Fi / MQTT reconnects, NVS provisioning, fault reporting; OTA if in scope
- Owns the **MQTT topic + JSON schema contract** with the AI subteam
- Bring-up on the final PCBs (pin map changes only)

## Layout

```
software/
└── firmware/            Rust (ESP-IDF) node firmware — one image for every node
    └── src/
        ├── main.rs      boot, start threads, supervisor loop
        ├── hardware/    bus.rs, sensors/ (one file per sensor)
        ├── network/     wifi.rs, mqtt.rs
        ├── ui/          display.rs (DWIN DGUS), led.rs (WS2812B)
        └── system/      state.rs, config.rs (NVS), logging.rs, errors.rs
```

Modules talk through the **shared system state** (`Arc<SharedState>`), not directly to each
other.

## Toolchain

| Item | Version |
|---|---|
| ESP-IDF | v6.1.0 → `ESP_IDF_VERSION = "v6.1"` |
| `esp-idf-sys` / `esp-idf-hal` / `esp-idf-svc` | 0.38.1 / 0.47.0 / 0.53.0 |
| Rust | nightly + `rust-src` (dated nightly pinned in `rust-toolchain.toml` after the first clean build) |
| Target | `riscv32imac-esp-espidf` |

- **MQTT isn't built into ESP-IDF v6** — it's added as the `espressif/mqtt` managed
  component in `Cargo.toml`.
- Check every third-party crate against `esp-idf-hal` 0.47 / `embedded-hal` 1.0 before
  adding it.
- Starter `.cargo/config.toml` and `rust-toolchain.toml` are in the Firmware Architecture
  doc §1.

```bash
cd software/firmware
cargo build
cargo fmt && cargo clippy    # before every PR
```

**New to Rust on ESP?** Start with [The Rust on ESP Book](https://docs.espressif.com/projects/rust/book/),
then the [Rust on ESP docs](https://docs.espressif.com/projects/rust/) and the
[esp-rs/esp-idf](https://github.com/esp-rs/esp-idf) repo.

## MQTT topics

| Topic | Direction | Content |
|---|---|---|
| `intellithings/<node_id>/sensors` | publish | One JSON snapshot of all readings |
| `intellithings/<node_id>/status` | publish (retained, Last Will) | `online` / `offline` |
| `intellithings/<node_id>/display` | subscribe (HA publishes retained) | `{"text": "...", "ts": "..."}` — latest AI message |

`<node_id>` comes from the chip's MAC. JSON field names follow the schema contract in
`docs/interfaces/` (fixed Oct 4) — the AI team's emulator (`ai/emulator/`) follows it too.

## Things that will bite you

- **The DWIN display must be in TTL mode** before it's wired to the ESP32 — RS232 levels
  damage the pins.
- The display renders its own screens (built in DWIN's DGUS tool, loaded from SD). The
  ESP32 only writes VP-address frames over UART — no framebuffer.
- The AI section keeps the latest message **until a new one arrives** — no timeout.
- Avoid GPIO12/13 (USB-JTAG) and 16/17 (UART bridge); don't put the LED data line or
  LD2410C OUT on strapping pins GPIO4, 5, 8, 9, 15.

## Workflow

- **Test on Rig S** before opening a PR. Rig wiring changes go through Hardware — both rigs
  stay identical.
- Review: sensor changes → a teammate who's worked on sensors; display/LED → a teammate
  who's worked on display/LED; architecture, cross-module or MQTT schema → the Software
  Lead.
- Talk to the module/task owner before changing GPIO assignments, sensor data structures,
  MQTT topics, system state, module APIs, the VP map or shared dependencies.

Branches: `software/baseline`, `software/sht41-driver`, `software/mqtt-reconnect`,
`software/dwin-dev-gui`, `software/led-bar`. See [`CONTRIBUTING.md`](../CONTRIBUTING.md).
