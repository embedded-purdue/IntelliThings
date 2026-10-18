# Firmware

Bare-metal **`no_std` Rust firmware for the ESP32-C6**, using `esp-hal`, `esp-rtos`,
Embassy async tasks/networking, and `esp-radio` for Wi-Fi. The package and binary are
named `intelli_things`.

## Toolchain and setup

Install [rustup](https://rust-lang.org/tools/install/). The checked-in
`rust-toolchain.toml` selects **stable Rust**, the `rust-src` component, and the
`riscv32imac-unknown-none-elf` target. `Cargo.toml` requires Rust **1.95 or newer**
and uses edition 2024. Nightly Rust and an ESP-IDF installation are not required.
The `unstable` features enabled on ESP crates expose experimental APIs; they do
not select a nightly compiler.

From the repository root:

```bash
cd software/firmware
rustup update stable
cargo install espflash --locked
```

Create `.cargo/esp-config.toml` as described below before building; `.cargo/config.toml`
includes it for local Wi-Fi settings.

```bash
cargo build                 # build without flashing
cargo run                   # build, flash a connected ESP32-C6, and monitor serial output
cargo fmt --check
cargo clippy
```

`cargo run` uses the configured runner: `espflash flash --monitor --chip esp32c6`.

## Current implementation

- `src/bin/main.rs`: hardware, heap, and RTOS initialization; async entry point.
- `src/lib.rs`: library module exports.
- `src/wifi.rs`: Wi-Fi connection, DHCP, Embassy network runner, and ping smoke test.

Wi-Fi setup returns the controller and network stack. Keep the controller alive to
maintain the connection, and reuse the stack for subsequent network operations.

Current bring-up: boot → Wi-Fi → DHCP → ping. MQTT and sensor snapshots are planned
next steps, not implemented yet.

[`Firmware_Architecture.md`](../Firmware_Architecture.md) describes the earlier
ESP-IDF design; its toolchain and threading instructions do not match this firmware.

## Wi-Fi and ping smoke test

Credentials are supplied through the local, ignored
`.cargo/esp-config.toml`; they are embedded into the firmware at build time. Do not
commit that file or share firmware images containing real credentials.

```toml
[env]
WIFI_SSID = { value = "your-2.4GHz-network", force = true }
WIFI_PASSWORD = { value = "your-password", force = true }
```

The firmware connects with DHCP and sends one ICMP echo request to `8.8.8.8`,
with a three-second timeout. The network stack has two socket slots: DHCP keeps
one occupied and ping uses the other. Increase the capacity if adding more
simultaneously active sockets.

Flash and monitor with `cargo run`; the serial output should show `Wi-Fi
connected`, `DHCP configured`, and either the round-trip time or a ping error.
