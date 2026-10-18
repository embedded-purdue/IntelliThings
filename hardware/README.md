# Hardware — Hardware Subteam

Lead: **@LiamWatson-Purdue** · 7 people · owns **Rig H**

The breadboard rigs, the pin map, power design, the custom PCB and the 3D-printed
enclosure for the three final sensor nodes. Design reasoning:
[`Hardware_BOM_Candidates.md`](Hardware_BOM_Candidates.md) ·
per-part specs: [`Full_Parts_List.md`](Full_Parts_List.md) ·
spending and build plan: [Project Guideline §3 and §7](../docs/Project_Guideline.md).

## Scope

- Build **Rig H and Rig S from one wiring diagram** so they stay identical; own the pin map
  (shared with Software)
- Pre-power checks: DWIN display set to **TTL mode**, 8-pin pinout confirmed, LD2410C supply
  voltage confirmed
- Power design; measure real current draw on the rig
- Compare the two WS2812B strips and pick one for the final bar
- Final PCB: schematic → layout → design review → JLCPCB order → SMT assembly of **3 nodes**
- Enclosure design and 3D printing ×3

## The node

| Block | Part | Interface |
|---|---|---|
| MCU | ESP32-C6 (DevKitC-1-N8 on the rigs) | — |
| Temp/RH, VOC, CO₂, light, distance | SHT41, SGP40, SCD41, BH1750, VL53L0X | One I2C bus (0x44, 0x59, 0x62, 0x23, 0x29) |
| Particulates | PMS5003 | UART0, 9600 baud (+ optional SET pin) |
| Presence | LD2410C mmWave | GPIO (OUT pin only) |
| Display | DWIN DMG80480T050_09WN, 5", 12 V | UART1, 8-pin 2.0 mm connector |
| LED bar | WS2812B | RMT, one data line via 3.3 → 5 V level shifter |

**7–8 pins, 2 UARTs.** On the DevKitC-1 avoid GPIO12/13 (USB-JTAG) and 16/17 (UART bridge);
don't put the LED data line or LD2410C OUT on strapping pins GPIO4, 5, 8, 9, 15.

## Power

Rails: **12 V** display · **5 V** PMS5003, LD2410C, LED bar · **3.3 V** ESP32-C6 + I2C sensors.
Input is 12 V (buck to 5 V) or 5 V (boost to 12 V for the display) — chosen during PCB
design. The **LED bar dominates the 5 V budget** (~60 mA per LED at full white). Common
ground everywhere.

## Final PCB

- **MCU:** bare ESP32-C6 chip if time allows; **fallback:** the DevKit plugged into female
  headers (or soldered to male headers). Decided at the **Oct 25** layout session — switch
  to the fallback rather than slip the order.
- **Option:** split into sensor / ESP / connector-power boards (each its own KiCad project;
  board-to-board pinouts become contracts in `docs/interfaces/`).
- Copy pull-ups, decoupling and regulators from the Adafruit breakout reference designs.

**Placement rules:** SHT41 and SCD41 away from heat (ESP32, converters, backlight) · clear
PMS5003 air inlet and outlet · windows for VL53L0X and BH1750 · no metal or ground pour in
front of the LD2410C · antenna keep-out for the bare chip or DevKit.

**Dates:** schematic by **Oct 18** · layout by **Oct 25** · design review and **JLCPCB order
Nov 1** (~7 days to DHL delivery, +~1 day with a stencil) · 3 nodes assembled by **Nov 15**.

## KiCad collaboration

- One board, one KiCad project, split into hierarchical sheets: **Power · MCU · I/O**.
- Each sheet has one owner at a time (assigned through Issues). **Never two people on the
  same sheet at once.**
- Keep every symbol and footprint in project-local libraries under `hardware/`.
- **Agree before changing:** GPIO assignments, I2C addresses, UART use, voltage levels,
  power requirements, connector types and pinouts, board placement.
- Layout: review the schematic together, confirm footprints and stock, 2–3 people route,
  everyone else reviews datasheets/footprints/DRC, **final team design review before
  ordering**.

## What to commit

**Yes:** schematics, layouts, CAD source, BOMs, datasheets.
**No:** gerbers, STLs, autosave/backup files — all re-exportable, all gitignored.
Anything over ~10 MB, ask in Discord first.

Branches: `hardware/power-input`, `hardware/mcu-bare-chip`, `hardware/io-sensor-bus`,
`hardware/led-bar`. See [`CONTRIBUTING.md`](../CONTRIBUTING.md).
