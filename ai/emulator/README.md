# Emulator ESP32

Owner: @spicybutter (PM) · AI subteam

One ESP32 with no sensors that publishes as three nodes (`emu-1`, `emu-2`, `emu-3`) on the
same MQTT topics and JSON schema as the real nodes, so Home Assistant and the agent can be
built before the firmware is ready. It subscribes to each `.../display` topic and prints
the AI messages over serial. See [`../README.md`](../README.md#emulator-esp32--owner-spicybutter-pm)
and Project Guideline §3.1a.
