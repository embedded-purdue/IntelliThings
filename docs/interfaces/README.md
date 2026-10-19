# Interfaces

Shared contracts between subteams, one file each. A contract is written here **before**
anyone builds against it, and changed only after every affected team's lead approves.

| Contract | Between | Status |
|---|---|---|
| [MQTT topics + JSON payloads](mqtt.md): **the single list of every MQTT topic** | Software ↔ AI | Agreed (Oct 4) |
| [Display VP map](display-vp-map.md) ([HTML reference](display-vp-map.html)) | Software ↔ display UI (@rakkicow) | v1.0 |
| AI message format (`text`, `ts`) + max length for the AI section | AI ↔ Software | Format in [`mqtt.md`](mqtt.md); **length still being tested** on the display |
| [HA entity names + MCP tools](ha-entities.md) | AI subteam (shared with all) | Draft (what's built) |
| [Rig wiring + pin map + power](pin-map.md) | Hardware ↔ Software | Stub: I2C pins and power confirmed, rest TBD |
| Final PCB pin map | Hardware → Software | Due Oct 25 |
| Final GUI layout ↔ enclosure bezel | Software ↔ Hardware | Due Nov 1 |
| `/ha-event` snapshot format | AI subteam | Only for the future cloud (Lambda) option; not needed by the local pipeline |
| `/chat` request format | AI subteam | After the Discord/Telegram decision |
