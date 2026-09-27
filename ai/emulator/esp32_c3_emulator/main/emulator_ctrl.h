// Emulator control: the command handler shared by the web panel (POST /api/cmd) and the
// MQTT topic intellithings/emulator/cmd, plus the status snapshot on intellithings/emulator/state.
// The emulator uses its own (fourth) MQTT connection so none of this mixes with node traffic.
#pragma once

#include <stdbool.h>
#include <stddef.h>

void ctrl_init(void);   // call once at boot
void ctrl_start(void);  // emulator MQTT connection (after the first IP)
bool ctrl_mqtt_connected(void);

// Runs a JSON command. Returns ok and writes a human-readable result into msg.
bool ctrl_execute(const char *json, size_t len, const char *source, char *msg, size_t msg_len);

char *ctrl_state_json(void);  // caller frees with cJSON_free()
char *ctrl_meta_json(void);   // metrics, scenarios, nodes; caller frees with cJSON_free()

// Called from the sim task: publishes state if due (every publish, or right after a command).
void ctrl_publish_state(bool force);
