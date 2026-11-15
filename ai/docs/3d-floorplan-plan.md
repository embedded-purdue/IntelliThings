# Demo dashboard

**Dashboard: Zircon3D.** Room topics and payloads live in the [demo device inventory](demo-device-inventory.md); do not maintain another device list here.

## Current status

Team update, Nov 8: `zircon3d-proxy` is set up in Docker with the team's Zircon account. Container logs report a successful connection to Home Assistant. Entity visibility and live updates in the Zircon3D viewer still need verification.

## Remaining work

1. Verify an existing HA sensor appears in Zircon3D and its value updates when HA receives a new reading.
2. Create HA entities from the agreed demo room topics and assign them to the matching HA areas. Keep demo and real sensor context distinguishable.
3. Build the room layout in Zircon3D and bind each displayed value to its HA entity. Start with temperature, humidity, occupancy, and ceiling-light state.
4. Add air-quality readings and the remaining device states from the inventory. Use simple labels/cards where an animation is unavailable; animations are optional.
5. Show the overall AI summary and thinking status alongside the room view. Add HA history/logbook for observed changes; use the agent's action records to distinguish attempted commands from confirmed outcomes.
6. Test on the actual booth monitor/browser and save reproducible HA and proxy deployment configuration under `ai/deployment/home_assistant/`, excluding credentials.

The dashboard displays HA state. Agent commands must reach the simulator, and resulting state changes must return through MQTT to HA. A dashboard toggle or AI summary alone does not establish that the simulated environment changed. The current agent's single turn-on action needs extending for the planned controls.

## Done when

- Each room shows its current readings, occupancy, and implemented device states.
- A changed demo MQTT payload updates the corresponding HA entity and dashboard view.
- An implemented AI action changes simulator state and that confirmed state appears on the dashboard.
- Unknown/unavailable values are visibly distinct from normal readings; stale-data handling is tested.
- The booth display runs smoothly and another teammate can reproduce the setup.

## References

- [Demo device inventory](demo-device-inventory.md): room-topic payloads and remaining integration decisions.
- [HA entity contract](../../docs/interfaces/ha-entities.md): existing entities and MCP controls; older proposed lists are marked separately.
- [MQTT contract](../../docs/interfaces/mqtt.md): existing interfaces; proposed demo section still needs synchronization with the inventory.
- [Zircon3D documentation](https://zircon3d.com/docs): installation and Designer instructions.
