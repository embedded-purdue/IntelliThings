# Firmware — Software / Firmware Subteam

ESP-IDF firmware for the desktop companion, the distributed sensor nodes, and the
demo actuator kit.

This is the largest authored-code surface in the project by a wide margin.

## Scope

- ESP-IDF / FreeRTOS application for the companion unit and nodes
- Eight sensor drivers over I2C: temperature, humidity, PM, TVOC, CO2, ambient light,
  presence, ultrasonic
- 7" display UI
- IR blaster (universal remote transmit)
- MQTT client — publish telemetry to the Pi 5 broker
- OTA update path
- Data validation and cleaning before anything leaves the device

## Planned layout

```
firmware/
├── companion/          # flagship desktop unit app
├── node/               # distributed sensor node app
├── actuators/          # demo kit (WS2812B, relay, servo/fan)
└── components/         # shared ESP-IDF components
    ├── sensors/        # one driver per part
    ├── mqtt_client/
    ├── display/
    └── ir_blaster/
```

Shared code goes in `components/` as proper ESP-IDF components so all three apps can
pull from it. Don't copy-paste a driver between apps.

## Build

```bash
. ~/esp/esp-idf/export.sh
cd firmware/companion
idf.py set-target esp32c5
idf.py build
idf.py -p /dev/cu.usbserial-XXXX flash monitor
```

Exit the monitor with `Ctrl+]`.

## Conventions

- ESP-IDF style guide: 4-space indent, `snake_case`, no tabs
- One driver per sensor, each behind a clean `init()` / `read()` interface
- Return `esp_err_t`, check it, log failures with `ESP_LOGE`
- No blocking delays in tasks — use FreeRTOS primitives
- **No credentials in source.** Wi-Fi and MQTT creds come from a gitignored config.

## Testing

Sensors lie. Validate readings against a known reference before trusting a driver, and
write down what "plausible" looks like for each part — it's the only way we'll catch a
sensor that fails soft in week 11.
