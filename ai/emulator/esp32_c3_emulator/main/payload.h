// MQTT topic and sensors-payload formatting (docs/interfaces/mqtt.md).
// Plain C (no ESP-IDF calls) so it can be unit-tested on a host.
#pragma once

#include <stddef.h>

#include "sim_model.h"

#define TOPIC_ROOT "intellithings"
#define TOPIC_EMULATOR_STATUS TOPIC_ROOT "/emulator/status"
#define TOPIC_EMULATOR_STATE TOPIC_ROOT "/emulator/state"
#define TOPIC_EMULATOR_CMD TOPIC_ROOT "/emulator/cmd"

// "intellithings/<node_id>/<leaf>". Returns false if it didn't fit.
bool payload_topic(char *buf, size_t len, const char *node_id, const char *leaf);

// Compact JSON snapshot. ts may be NULL (omitted until the clock has synced).
// Returns the JSON length, or -1 if it didn't fit.
int payload_sensors_json(char *buf, size_t len, const char *node_id, const sim_reading_t *r,
                         const char *ts);
