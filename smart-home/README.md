# Smart Home — Smart Home Subteam

Home Assistant on the Raspberry Pi 5, the MQTT broker, Matter, virtual devices, and the
real-time dashboard.

This is the integration point where hardware, firmware, and the LLM all have to land.

## Scope

- Home Assistant OS on the Pi 5
- MQTT broker — ingest telemetry from firmware nodes
- Matter for local device interop
- Home Assistant **AI Task** integration
- Home Assistant **MCP Server** — expose HA as callable tools for the cloud agent
- Virtual device fleet (represents a full-size home) + physical demo devices
- Real-time dashboard, including the 3D floor-plan view
- Network setup and security

## Planned layout

```
smart-home/
├── home-assistant/
│   ├── configuration.yaml
│   ├── automations/
│   ├── dashboards/
│   └── packages/
├── mqtt/               # broker config, topic schema
└── docs/               # network diagram, device inventory
```

## Start here

Run Home Assistant locally in Docker — you don't need the Pi to begin:

```bash
docker run -d --name homeassistant \
  --privileged --restart=unless-stopped \
  -e TZ=America/Indiana/Indianapolis \
  -v "$(pwd)/home-assistant:/config" \
  --network=host \
  ghcr.io/home-assistant/home-assistant:stable
```

http://localhost:8123

## MQTT topic schema

Agree on this early and write it down here — firmware has to publish to whatever we
decide, and changing it later means reflashing every node.

Suggested shape:

```
intellithings/<node-id>/sensor/<metric>      # telemetry, node → broker
intellithings/<node-id>/status               # online/offline (LWT)
intellithings/<node-id>/command/<target>     # commands, broker → node
```

## Secrets

Home Assistant secrets go in `secrets.yaml` and are referenced with `!secret`.
That file is **gitignored** — never commit it. Commit a `secrets.yaml.example` with
the keys and blank values instead.

Also gitignored: `.storage/`, the SQLite database, and logs. Those are runtime state,
not configuration.

## Note on risk

Nobody started this project having configured Home Assistant, and HA AI Task, MCP
Server, and Matter are the thinnest-documented parts of the stack. Expect to
experiment, and expect this layer to gate the demo.

Stand the hub up in week 1. Not week 6.
