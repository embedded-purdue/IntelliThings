# Firmware
To run, install rustup https://rust-lang.org/tools/install/
For the first time, run `cargo install espflash --locked` and then `cargo run`.
Every time you want to build/deploy, run `cargo run` which will build and deploy to an esp32c6 (if connected)

[`../Firmware_Architecture.md`](../Firmware_Architecture.md).

Baseline first: boot → Wi-Fi → MQTT → placeholder snapshot in the agreed schema.

## Wi-Fi and ping smoke test

Credentials are supplied through the local, ignored
`.cargo/esp-config.toml`; they are embedded into the firmware at build time, so
do not flash a build made from a shared machine or commit that file.

```toml
[env]
WIFI_SSID = { value = "your-2.4GHz-network", force = true }
WIFI_PASSWORD = { value = "your-password", force = true }
```

The firmware connects with DHCP and sends four ICMP echo requests to `8.8.8.8`.
Flash and monitor with `cargo run`; the serial output should show `Wi-Fi
connected`, `DHCP configured`, and either the round-trip time or a ping error.
