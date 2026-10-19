# Software — Firmware Subteam

Lead: **@LukeTuthill** · 9 people · owns **Rig S**

ESP32-C6 firmware for the sensor nodes, written in **Rust (`no_std`, stable) with
`esp-hal`, `esp-rtos`, and Embassy**. Current build and run instructions:
[`firmware/README.md`](firmware/README.md).

[`Firmware_Architecture.md`](Firmware_Architecture.md) describes the earlier ESP-IDF
design; its toolchain and threading instructions do not match the current firmware.

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
└── firmware/            no_std Rust node firmware — one image for every node
    └── src/
        ├── bin/
        │   └── main.rs  hardware/heap/RTOS initialization and async entry point
        ├── lib.rs       library module exports
        └── wifi.rs      Wi-Fi setup, Embassy network runner, and ping smoke test
```

Sensor, MQTT, display, LED, and shared-state modules are planned work, not yet part of
the source layout above. Wi-Fi setup returns a controller (keep it alive) and a reusable
Embassy network stack.

## Toolchain

| Item | Version |
|---|---|
| Rust | stable + `rust-src`; minimum Rust 1.95, edition 2024 |
| Target | `riscv32imac-unknown-none-elf` (ESP32-C6) |
| HAL | `esp-hal` ~1.2.2 |
| Runtime | `esp-rtos` 0.4.0 with Embassy support |
| Async executor / networking | `embassy-executor` 0.10.0 / `embassy-net` 0.9.1 |
| Wi-Fi driver | `esp-radio` 1.0.0-beta.1 |
| Flashing / serial monitor | `espflash` |

- This is bare-metal `no_std` firmware, not an ESP-IDF/FreeRTOS application. No nightly
  toolchain or ESP-IDF installation is required; crate features named `unstable` are
  unrelated to the Rust toolchain channel.
- MQTT is planned; no MQTT client is currently configured in `Cargo.toml`.
- Check new dependencies for `no_std`, ESP32-C6, and compatibility with the current HAL
  and Embassy versions.
- The current configuration lives in `firmware/rust-toolchain.toml`,
  `firmware/.cargo/config.toml`, and `firmware/Cargo.toml`. See the
  [firmware README](firmware/README.md) for first-time setup and local Wi-Fi credentials.

```bash
cd software/firmware
cargo build
cargo fmt && cargo clippy    # before every PR
```

**New to Rust on ESP?** Start with [The Rust on ESP Book](https://docs.espressif.com/projects/rust/book/),
then the [Rust on ESP docs](https://docs.espressif.com/projects/rust/) and the
[esp-rs/esp-hal](https://github.com/esp-rs/esp-hal) repo.

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
