// The scenario library and the value limits, editable on the admin page. Both start from the
// built-in tables (demo/demo_home.json) and are saved together in the "demolib" flash partition,
// so edits survive a reboot. "Reset" brings the built-in library and limits back.
#pragma once

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#include "demo_home.h"
#include "json_writer.h"

#define LIB_MAX 200  // scenarios (MAX_SCENARIOS in tools/gen_demo.py)

void library_init(void);  // after demo_home_init(): the saved library, else the built-in one

int library_count(void);
bool library_get(int i, demo_scenario_t *out);  // 0-based
int library_pick(uint32_t *rng, int last);       // a random index other than last; -1 if empty
uint32_t library_version(void);                  // bumps on every change
bool library_is_factory(void);                   // built-in scenarios and limits, unedited

// i = -1 adds a scenario. Checks the name, group and values; saves to flash. *index = where it is.
bool library_put(int i, const demo_scenario_t *s, int *index, char *msg, size_t len);
bool library_delete(int i, char *msg, size_t len);
bool library_reset(char *msg, size_t len);  // built-in scenarios and limits

bool library_set_limit(int e, const demo_limit_t *l, char *msg, size_t len);  // set and save
bool library_reset_limits(char *msg, size_t len);

// [{"name":..,"group":..,"n":..}, ...]
void library_write_list(jw_t *w);
// {"name":..,"group":..,"set":{"kitchen.temperature_c":24.5,...}} for scenario i
bool library_write_scenario(jw_t *w, int i);
