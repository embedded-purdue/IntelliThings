// Control panel for the AI team, served by the emulator itself:
//   GET  /               the panel (main/web/index.html, embedded in the firmware)
//   GET  /api/meta       metrics, scenarios, nodes and topics
//   GET  /api/state      live system status and readings (same JSON as intellithings/emulator/state)
//   GET  /api/messages   recent AI messages received on the display topics
//   POST /api/cmd        run a command (same JSON as intellithings/emulator/cmd)
#pragma once

void web_server_start(void);
