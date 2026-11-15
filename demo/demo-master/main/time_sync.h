#pragma once

#include <stdbool.h>
#include <stddef.h>

void time_sync_init(void);   // set timezone (call once at boot)
void time_sync_start(void);  // start SNTP (call after the first IP)
bool time_is_synced(void);
float time_local_hour(void); // 0..24; falls back to a boot-based clock before sync
// ISO 8601 with UTC offset, e.g. 2026-09-27T14:05:00-04:00. False if not synced.
bool time_iso8601(char *buf, size_t len);
