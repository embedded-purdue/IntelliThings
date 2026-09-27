# Interfaces

Shared contracts between subteams — one file each. A contract is written here **before**
anyone builds against it, and changed only after every affected team's lead approves.

| Contract | Between | Due |
|---|---|---|
| Rig wiring diagram + pin map + power rails | Hardware ↔ Software | Oct 4 |
| MQTT topics + JSON payload schema (emulator follows it too) | Software ↔ AI | Oct 4 |
| `/ha-event` snapshot format (HA → cloud agent) | Local AI ↔ Cloud AI | Oct 4 |
| AI message format (`text`, `ts`) + max length for the AI section | AI ↔ Software | Oct 18 |
| HA entity names + MCP tools exposed to the agent | Local AI ↔ Cloud AI | Oct 18 |
| `/chat` request format | Cloud AI | Oct 25 |
| Final PCB pin map | Hardware → Software | Oct 25 |
| Final GUI layout ↔ enclosure bezel | Software ↔ Hardware | Nov 1 |
