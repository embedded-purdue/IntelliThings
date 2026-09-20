# Project Timeline

13 weeks to make something great. Meetings every Sunday, 1–4 PM.

## Weeks 1–3 · Planning & Research

Introduce the project, brainstorm solutions, define the system architecture, build the
bill of materials, and order development hardware.

- [ ] Architecture locked well enough to order parts
- [ ] BOM finalized and ordered (long lead times bite first)
- [ ] Repo, branches, and toolchains set up for all three subteams
- [ ] **Pi 5 hub stood up with Home Assistant** — start this now, not in week 6

## Weeks 4–5 · Hardware & Firmware Prototyping

Build the ESP-32 breadboard prototype, connect sensors and devices, and begin firmware
development and testing.

- [ ] Breadboard prototype with sensors reading over I2C
- [ ] First sensor drivers written and validated
- [ ] MQTT publish path working from ESP32 to the broker
- [ ] 7" display bring-up started

## Weeks 6–7 · AI & System Integration

Prototype the cloud AI agent and prompt design, connect the AI task pipeline, and begin
end-to-end testing with sensor data.

- [ ] Cloud agent reachable and reasoning over real sensor state
- [ ] HA AI Task pipeline connected
- [ ] MCP server exposing HA tools to the agent
- [ ] First end-to-end: sensor reading → agent decision → device actuation

## Weeks 8–10 · PCB & Full Pipeline Development

Design and order the PCB; integrate Home Assistant MCP, refine AI decision logic,
develop the 3D dashboard, and test the complete pipeline.

- [ ] PCB schematic → layout → **design review** → ordered
- [ ] Virtual device fleet populated in Home Assistant
- [ ] 3D floor-plan dashboard built
- [ ] Decision logic refined against real usage patterns

## Weeks 11–13 · Assembly & Final Integration

Assemble and solder the PCB, design and 3D-print the enclosure, finalize the dashboard
and virtual devices, and complete full-system verification.

- [ ] PCB assembled and bring-up complete
- [ ] Enclosure printed and fitted
- [ ] Demo actuator kit wired and rehearsed
- [ ] Full-system verification
- [ ] SPARK Challenge demo rehearsed end to end

## Scheduling notes

Parts ordering and PCB fab are the two hard external dependencies. Both have lead times
measured in weeks, and neither compresses. Everything else can be worked in parallel;
these two cannot be recovered if they slip.
