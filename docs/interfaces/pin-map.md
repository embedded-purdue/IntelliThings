# Pin map and power

> **Status: stub.** Hardware (@LiamWatson-Purdue) owns this map together with Software
> (@LukeTuthill). Only the confirmed items are filled in; everything else is **TBD (Hardware)**.
> Both rigs and the final PCB must match it.

## Board and power (confirmed)

- **ESP32-C6-DevKitC-1** plugged into sockets on the PCB (no bare chip).
- **Sensor breakout boards** plug into 5- and 6-pin sockets on the PCB. Custom footprints are in
  progress.
- **Power in:** USB-C, power only, with 5.1 kΩ pull-downs on CC1/CC2. That gives a **5 V** rail;
  3.3 V comes from the DevKit.
- **The DWIN display needs 12 V,** so the 5 V input needs a 5→12 V boost. Part: TBD (Hardware).

## Pins

| Function | GPIO | Notes |
|---|---|---|
| I2C SDA (SHT41, SGP40, SCD41, BH1750, VL53L0X) | **GPIO6** | 100 kHz, confirmed in firmware |
| I2C SCL | **GPIO7** | |
| DWIN display UART TX / RX | TBD | 115200 8N1, 3.3 V TTL (display in TTL mode) |
| PMS5003 UART RX (+ optional SET) | TBD | 9600 baud |
| LD2410C OUT | TBD | not a strapping pin |
| WS2812B LED data | TBD | not a strapping pin |

**Avoid on the DevKitC-1:**
- GPIO12/13 (native USB).
- GPIO16/17 (USB-to-UART bridge).
- GPIO24–30 (used by the module).
- The strapping pins GPIO4, 5, 8, 9 and 15 need care.
