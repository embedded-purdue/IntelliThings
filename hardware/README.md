----

# **Placeholder**

----

# Hardware — Hardware Subteam

PCB design, enclosure CAD, and the physical build of the companion unit, sensor nodes,
and demo actuator kit.

## Scope

- Select and wire environmental sensors to the ESP32-C5
- Custom PCB: schematic → layout → fab → assembly
- 7" display mount and integration
- Power supply design
- 3D-printed chassis for the companion; enclosures for the nodes
- Demo kit wiring: WS2812B strips, 5V relay, servo/fan

## Planned layout

```
hardware/
├── pcb/
│   ├── companion/      # flagship board
│   └── node/           # sensor node board
├── cad/
│   ├── chassis/        # companion enclosure
│   └── node-enclosure/
├── datasheets/         # one PDF per BOM part
└── bom/
```

## Bill of materials

Estimated budget **~$375**.

| Item | Use | Qty | Est. |
|---|---|---:|---:|
| ESP32-C5 | MCU for companion + nodes | 2 | $38 |
| Base environmental sensors | Temp, humidity, PM, TVOC, presence, ultrasonic | 6 | $60 |
| True CO2 sensor (SCD40/41) | High-accuracy air quality | 1 | $20 |
| Ambient light sensor (BH1750) | Room brightness | 1 | $3 |
| IR transmitter (blaster) | Universal remote | 1 | $2 |
| Actuation demo kit | WS2812B, 5V relay, servo/fan | 1 set | $12 |
| 7" color display | Companion UI | 1 | $50 |
| PCB prototypes | Fabrication | 5 | $50 |
| Electronic components (SMT) | PCB assembly | ~15 types | $80 |
| Power supply | Nodes / hub / actuators | 1 | $10 |
| Raspberry Pi 5 | Home Assistant hub | 1 | owned |
| Wi-Fi router | Smart home network | 1 | owned |
| 3D printed enclosure | Final prototype | 1 | self-print |
| Cloud LLM API | Reasoning engine | — | $10 |

## The design review rule

**Five prototype fabs. That's the whole budget.**

No board gets ordered without a design review — schematic and layout, at least two sets
of eyes, one of them a lead. Turn time on a fab run is measured in weeks and we don't
have weeks to spare in the back half of the semester.

Check before ordering: footprints against datasheets, decoupling on every IC, power
rail current budget, connector orientation, test points on anything you'll need to probe.

## What to commit

**Yes:** schematics, layouts, CAD source files, BOMs, datasheets.
**No:** gerbers, STLs, autosave/backup files — all re-exportable, all gitignored.

Anything over ~10 MB, ask in Discord first.
