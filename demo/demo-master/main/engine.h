// The demo engine: the scenario loop, "Roll the house" and publishing, on its own task.
// Adapted from the emulator's demo mode (ai/emulator/esp32_c3_emulator/main/demo_engine.c).
// Publishes one retained snapshot per area on <prefix>/<area key> and the doorbell event on
// <prefix>/home/doorbell, as in ai/docs/demo-device-inventory.md.
#pragma once

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#include "cJSON.h"
#include "json_writer.h"

typedef enum { PRESS_BUZZER, PRESS_WEB, PRESS_KEY, PRESS_ADMIN, PRESS_SOURCES } press_src_t;
typedef enum { PRESS_ROLLING, PRESS_BUSY, PRESS_LOCKED } press_result_t;

void engine_init(void);          // after settings_init() and library_init()
void engine_on_connected(void);  // MQTT (re)connected: republish everything
void engine_republish(void);     // republish every area now

// A buzzer press from any source. PRESS_LOCKED: *lock_left_s says for how long.
press_result_t engine_press(press_src_t src, int *lock_left_s);

// Admin commands {"op":...}: publish, loop, settings, roll, play, next, doorbell, republish,
// reset_counts. False with the reason in msg.
bool engine_command(const cJSON *root, char *msg, size_t len);

// The state for the pages; the values only when they changed after `since` (a seq).
void engine_write_state(jw_t *w, uint32_t since);
// Counters and timings for the admin overview.
void engine_write_admin(jw_t *w);
