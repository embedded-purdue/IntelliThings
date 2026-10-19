# Display VP map (DWIN)

> **Status: v1.0, agreed.** Owner and maintainer of the display UI: **@rakkicow (PM)**.
> Changes need review from @rakkicow and the Software lead (@LukeTuthill).
> Full formatted reference: [`display-vp-map.html`](display-vp-map.html) (open it in a browser).
> The generated `vp_map.csv`, `vp_map.json` and the drop-in Rust constants `dwin_vp.rs` will be
> added by @rakkicow.

## How the display works

- **Each node's ESP32-C6 drives its own DWIN DMG80480T050_09WN** over a UART: **115200 8N1,
  3.3 V TTL**. The display must be in **TTL mode** before it's wired; RS232 levels damage the
  ESP32.
- The UI is **custom DGUS structures written by the team** (by @rakkicow). DWIN's own DGUS design
  tool isn't used, and **LVGL was dropped**.
- The node only writes **input variables (VPs) in `0x1000–0x17FF`**. The display turns them into
  the screens. The DWIN panel's internal VPs are never written by the node.
- **The ESP32 drives the WS2812B LED bar**, not the DWIN panel. How it talks to the strip is
  still being worked out by Hardware. Whether the bar follows the map's `led_brightness` setting
  is still to be decided.
- **Bench setup (now only):** a separate "companion" ESP32-C6 runs the UI firmware and takes VP
  writes from the node over UART: node TX → companion GP2, companion GP3 → node RX, GND to GND.
  Or a laptop sends them over the companion's USB with `tools/companion.py`. The final nodes are
  one chip: each node writes the VPs to its own DWIN.

## Frame format

| | Bytes |
|---|---|
| Write | `5A A5 <len> 82 <VP hi> <VP lo> <data…>`. `len` = 3 + data bytes. Data is whole 16-bit words, high byte first, at most 240 bytes (120 words) per frame. One frame can cover several consecutive VPs |
| Reply | `5A A5 03 82 4F 4B` ("OK"). No CRC |
| Read | `5A A5 04 83 <VP hi> <VP lo> <n>` → `5A A5 <4+2n> 83 <VP> <n> <words>` |

## Value encoding

- **No reading:** `0x8000` (sensor missing or faulted); the tile shows *Waiting*.
- **Temperature and humidity are × 10:** `224` = 22.4 °C. Temperature is signed.
- **`u32`:** two words, high word first.
- **Text:** UTF-8, two bytes per word (first byte in the high half), ending with a `0x00` byte.
  A first word of `0x0000` means empty. Characters outside the display's font are skipped. The
  font covers A–Z, a–z, 0–9, ASCII punctuation and `° ₂ µ ³ · … ’ ‘ “ ” – — − ×`.
- **Units:** MQTT always carries **°C** (see [`mqtt.md`](mqtt.md)). The display can show °F or °C
  through the `units` setting below; its power-up default is °F.

## Which screens IntelliThings uses

| Screen / block | Used? |
|---|---|
| Rooms page (`0x1500–0x154E`) | **Not used** |
| Wi-Fi setup page with QR code (`0x1700–0x172F`) | **Not used.** Wi-Fi settings are compiled into the firmware for the prototype |
| Home, sensors, alert, night, away, boot, toast, action chips, "thinking" line | **Open discussion:** depends on how far the Software team gets |

## AI message length (needs testing)

> **Not final.** How much text fits in the AI section still has to be **tested on the real
> display**. The map's own estimate is **about 110 characters** (4 lines), or **about 80** when
> action chips are shown; longer text is cut with "…". The `message_text` buffer holds 512 bytes.
> Today the AI pipeline sends room messages of **at most 60 characters**, which fits either way.
> The final limit goes into [`mqtt.md`](mqtt.md) once it's measured.

## VP map

### Sensor readings (this node): `0x1000–0x1009`

Same order as the MQTT `…/sensors` snapshot, so one 10-word frame sends them all. Write every
0.5–1 s or on change. All power up as `0x8000`.

| VP | Name | Type | Units | MQTT field |
|---|---|---|---|---|
| `0x1000` | `temperature` | i16 | °C × 10 | `temperature_c` |
| `0x1001` | `humidity` | u16 | %RH × 10 | `humidity_pct` |
| `0x1002` | `voc_index` | u16 | index 1–500 | `voc_index` |
| `0x1003` | `co2` | u16 | ppm | `co2_ppm` |
| `0x1004` | `lux` | u16 | lux (clamp to 32767) | `lux` |
| `0x1005` | `distance` | u16 | mm (accepted, not shown yet) | `distance_mm` |
| `0x1006` | `pm1_0` | u16 | µg/m³ (accepted, not shown yet) | `pm1_0_ugm3` |
| `0x1007` | `pm2_5` | u16 | µg/m³ | `pm2_5_ugm3` |
| `0x1008` | `pm10` | u16 | µg/m³ (accepted, not shown yet) | `pm10_ugm3` |
| `0x1009` | `presence` | u16 | `0` nobody, `1` someone there. `0x8000`: never switches to the away page | `presence` |

### Connection: `0x1010–0x1013`

| VP | Name | Type | Power-up | Meaning |
|---|---|---|---|---|
| `0x1010` | `wifi` | u16 | `0` | `0` disconnected, `1` connected |
| `0x1011` | `hub` | u16 | `0` | MQTT/HA link: `0` connecting, `1` online, `2` offline (the AI card becomes the offline card) |
| `0x1012` | `synced_ago` | u16 s | `0xFFFF` | Write `0` after each publish or message; the display counts up. `0xFFFF` = never |
| `0x1013` | `retry_in` | u16 s | `0` | Countdown to the next reconnect on the offline card. `0` = none |

### Clock: `0x1020–0x1022`

Write once after SNTP sync (again hourly is fine).

| VP | Name | Type | Meaning |
|---|---|---|---|
| `0x1020–0x1021` | `unix_time` | u32 | Seconds since 1970 UTC, high word first. `0` = unknown (no clock shown) |
| `0x1022` | `utc_offset` | i16 min | `−240` = EDT, `−300` = EST |

### Settings: `0x1030–0x104B` (optional)

| VP | Name | Power-up | Meaning |
|---|---|---|---|
| `0x1030` | `units` | `0` | `0` °F, `1` °C |
| `0x1031` | `clock_24h` | `0` | `0` 12-hour, `1` 24-hour |
| `0x1032` | `room` | `0` | Top-bar room name: `0` Living Room, `1` Kitchen, `2` Bedroom (`3` Office, `4` Entry exist but have no node) |
| `0x1033` | `night_start` | `1350` | Minutes after midnight (22:30). `0xFFFF` = never |
| `0x1034` | `night_end` | `390` | Minutes after midnight (6:30) |
| `0x1035` | `away_after` | `600` | Seconds without presence before the away page. `0` = never |
| `0x1036` | `rotate_pages` | `0` | `0` off: the AI message stays on screen (the team's layout rule). `1` rotate pages |
| `0x1037` | `screen` | `0` | Force a screen for testing: `0` automatic, `1` home, `2` sensors, `3` rooms, `4` alert, `5` night, `6` away, `7` boot, `8` setup, `9` test pattern |
| `0x1038` | `mode` | `0` | `0` demo until the first write, then live; `1` live only; `2` demo only |
| `0x1039` | `led_brightness` | `100` | Light bar, as a share of its safe maximum (0–100 %). Whether IntelliThings uses it: TBD (the ESP32 drives the bar) |
| `0x103F` | `inputs_reset` | `0` | Write `0x5AA5` at node boot to reset every input to its power-up value |
| `0x1040–0x104B` | `room_name` | empty | Text (24 bytes). Overrides the top-bar room name |

### Display status (read-only): `0x10F0–0x10F5`

Read with `5A A5 04 83 10 F0 06` to check the display is alive.

| VP | Name | Meaning |
|---|---|---|
| `0x10F0` | `interface_version` | `0x0100` = v1.0 |
| `0x10F1` | `panel_state` | `0` no panel, `1` panel without the UI files, `2` drawing |
| `0x10F2` | `screen_shown` | Same codes as `screen` |
| `0x10F3` | `mode_now` | `0` demo, `1` live |
| `0x10F4` | `writes_ok` | Write frames applied (wraps) |
| `0x10F5` | `writes_rejected` | Write frames outside the map or to read-only words |

### AI message (the MQTT `…/display` payload): `0x10FE–0x1255`

The card shows the text **until a different text arrives**: no timeout, and rewriting the same
text changes nothing. A new text also clears `thinking`.

| VP | Name | Type | Meaning | MQTT |
|---|---|---|---|---|
| `0x10FE–0x10FF` | `message_ts` | u32 | Payload `ts` as Unix seconds, for "2 min ago". `0` = counted from arrival | `ts` |
| `0x1100–0x11FF` | `message_text` | text, 512 B | Payload `text`; sent as 3 frames (at `0x1100`, `0x1178`, `0x11F0`). Empty = idle line | `text` |
| `0x1200–0x121F` | `thinking` | text, 64 B | While not empty: spinner + this line. Write empty, or a new message, to end it | See [`mqtt.md`](mqtt.md): open discussion |

Action chips (two, stride `0x20`):

| Field | Type | Chip 1 | Chip 2 | Meaning |
|---|---|---|---|---|
| `device` | u16 | `0x1220` | `0x1240` | Icon: `0` fan, `1` light, `2` thermostat, `3` purifier, `4` humidifier, `5` window, `6` door, `7` lock, `0xFFFF` no chip |
| `label` | text, 28 B | `0x1221–0x122E` | `0x1241–0x124E` | "Fan", "Close the bedroom door" |
| `value` | text, 12 B | `0x1230–0x1235` | `0x1250–0x1255` | After an arrow, e.g. "40%". Empty = none |

### Toast: `0x1300–0x1333`

A banner over the home page for 5.5 s.

| VP | Name | Type | Meaning |
|---|---|---|---|
| `0x1300` | `toast_trigger` | u16 | Write a new value (count up) to show it; fill the fields in the same frame or before |
| `0x1301` | `toast_device` | u16 | Icon (same codes as chips) |
| `0x1302–0x1315` | `toast_title` | text, 40 B | "Lights dimmed to 30%" |
| `0x1316–0x1333` | `toast_detail` | text, 60 B | "Living Room · after sunset, someone's here" |

### Alert: `0x1400–0x14E5`

Takes over the screen while active.

| VP | Name | Type | Meaning |
|---|---|---|---|
| `0x1400` | `alert_active` | u16 | `0` no alert, `1` alert page |
| `0x1401` | `alert_metric` | u16 | `0` temperature, `1` humidity, `2` CO₂, `3` PM2.5, `4` VOC index, `5` light |
| `0x1402` | `alert_value` | i16 | Shown big; same scaling as the sensor block |
| `0x1403` | `alert_since` | u16 s | Since it started; the display counts up |
| `0x1404–0x1417` | `alert_title` | text, 40 B | "Smoke or cooking particles" |
| `0x1418–0x1423` | `alert_place` | text, 24 B | "Kitchen" |
| `0x1424–0x1431` | `alert_normal` | text, 28 B | "Normal is under 12" |
| `0x1432–0x1481` | `alert_message` | text, 160 B | "Probably the stove. I'll tell you when the air clears." |

Alert chips (stride `0x20`), same fields as the AI message chips: **Done 1** `0x1490`, **Done 2**
`0x14B0`, **Request for the user** `0x14D0` (`device`, then `label` at +1, `value` at +0x10).

### Rooms page: `0x1500–0x154E` (not used)

Five room entries (Living Room `0x1500`, Kitchen `0x1510`, Bedroom `0x1520`, Office `0x1530`,
Entry `0x1540`; stride `0x10`). **IntelliThings doesn't use this page.** See the HTML reference
for the fields.

### Boot checklist: `0x1600–0x16B3`

The boot page shows while any step is pending or running, and for 1.5 s after the last one
finishes. Write all four node steps (`done` for any not used); step 0 is the display's own.

| Step | `state` | `detail` (text, 32 B) |
|---|---|---|
| Display (read-only) | `0x1600` | `0x1601–0x1610` |
| Sensors | `0x1620` | `0x1621–0x1630` |
| Wi-Fi | `0x1640` | `0x1641–0x1650` |
| MQTT broker | `0x1660` | `0x1661–0x1670` |
| Home Assistant | `0x1680` | `0x1681–0x1690` |

`state`: `0` pending, `1` running, `2` done, `3` failed, `0xFFFF` not used. `0x16A0–0x16B3`
`boot_version` (text, 40 B): bottom line, e.g. "v0.1.0 · node living-room-01".

### Wi-Fi setup: `0x1700–0x172F` (not used)

`setup_ssid`, `setup_password`, `setup_url`. **Not used:** Wi-Fi settings are compiled into the
firmware for the prototype.

## Example frames

| What | Bytes |
|---|---|
| CO₂ = 812 ppm | `5A A5 05 82 10 03 03 2C` |
| Temperature 22.4 °C (224) | `5A A5 05 82 10 00 00 E0` |
| Temperature −5.0 °C (−50 = `0xFFCE`) | `5A A5 05 82 10 00 FF CE` |
| All ten readings: 22.4 °C, 41.2 %, VOC 118, 642 ppm, 320 lx, 1180 mm, PM 3/6/7, someone there | `5A A5 17 82 10 00 00 E0 01 9C 00 76 02 82 01 40 04 9C 00 03 00 06 00 07 00 01` |
| PM2.5 sensor faulted | `5A A5 05 82 10 07 80 00` |
| Wi-Fi connected, hub online, just synced | `5A A5 09 82 10 10 00 01 00 01 00 00` |
| Clock: 2026-10-18 14:05 EDT (18:05 UTC, offset −240) | `5A A5 09 82 10 20 6A D5 0A 4C FF 10` |
| AI message "Hi!" | `5A A5 07 82 11 00 48 69 21 00` |
| Clear the AI message | `5A A5 05 82 11 00 00 00` |
| Reset every input (at node boot) | `5A A5 05 82 10 3F 5A A5` |
| Read the display status (6 words) | `5A A5 04 83 10 F0 06` |
