// Demo mode engine: runs the scenario loop and "Roll the house" on its own task and publishes
// the demo home under intellithings/emulator/demo/# over the control connection. Commands
// arrive as {"action":"demo","op":...} on POST /api/cmd or intellithings/emulator/cmd.
#pragma once

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#include "cJSON.h"
#include "demo_home.h"

#define DEMO_HISTORY_LEN 30

typedef struct {
    uint32_t seq;  // 1, 2, ... each time the values change; 0 = boot values
    demo_values_t values;
    const char *state;  // "idle", "looping", "rolling", "hold"
    bool publish, loop;
    int preset;         // 0-based, -1 for a roll
    uint32_t roll;      // roll number when the values came from a roll
    int64_t changed_us;
} demo_snapshot_t;

typedef struct {
    uint32_t seq;
    uint32_t t_s;  // seconds since boot
    int8_t preset;
    uint32_t roll;
    int16_t v[DEMO_MAX_ENTITIES];  // value x 10^dec
} demo_hist_t;

void demo_engine_init(void);  // at boot, after demo_home_init()
void demo_engine_on_connected(void);  // control client (re)connected: republish if on

// Runs a demo command; false with the reason in msg.
bool demo_engine_command(const cJSON *root, char *msg, size_t len);

void demo_engine_snapshot(demo_snapshot_t *out);
void demo_engine_add_state(cJSON *root);  // "demo" object for /api/state
// History entry i (0 = oldest) of the last DEMO_HISTORY_LEN changes; false past the end.
bool demo_engine_history(int i, demo_hist_t *out);
