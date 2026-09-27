# Firmware

Rust (`std`, nightly) firmware for the ESP32-C6 sensor nodes, on ESP-IDF v6.1.0. One image
for every node. Module layout, toolchain and starter config:
[`../Firmware_Architecture.md`](../Firmware_Architecture.md).

Baseline first: boot → Wi-Fi → MQTT → placeholder snapshot in the agreed schema.
