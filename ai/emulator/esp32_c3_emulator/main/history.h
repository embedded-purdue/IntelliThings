// Reading history for the panel's plots. Plain C (no ESP-IDF calls) so it can be unit-tested
// on a host. Not thread-safe: the caller serialises access (nodes_lock()).
//
// Two tiers, values packed as uint16 (value * scale, HIST_NONE = no data):
//   HIST_FINE   - every published snapshot, 200 slots (10 min at the default 3 s interval)
//   HIST_COARSE - one slot per bucket (default 15 s), 240 slots (1 h); numbers are averaged
//                 over the bucket, presence is "present at any point"
#pragma once

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#include "sim_model.h"

#define HIST_NODES 3
#define HIST_SERIES (METRIC_COUNT + 1)  // the nine metrics in metric_t order, then presence
#define HIST_PRESENCE METRIC_COUNT
#define HIST_NONE 0xFFFFu

typedef enum { HIST_FINE, HIST_COARSE, HIST_TIER_COUNT } hist_tier_id_t;

#define HIST_FINE_CAP 200
#define HIST_COARSE_CAP 240

// A stable window onto a tier: at most cap - 1 slots, so one more record() while the caller
// streams it can't overwrite anything inside the window.
typedef struct {
    uint16_t count;
    uint16_t first;  // slot index of the oldest sample in the window
} hist_view_t;

void history_init(uint16_t coarse_bucket_s);
// One snapshot for every node, t_s = seconds since boot (monotonic).
void history_record(uint32_t t_s, const sim_reading_t readings[HIST_NODES]);

uint16_t history_cap(hist_tier_id_t tier);
uint16_t history_bucket_s(void);
// Divisor that turns a stored value back into the reading (10 for 1-decimal metrics, else 1).
uint16_t history_scale(int series);

hist_view_t history_view(hist_tier_id_t tier);
// Copy n entries starting at window position pos (0 = oldest).
void history_copy_times(hist_tier_id_t tier, hist_view_t v, uint16_t pos, uint16_t n,
                        uint32_t *out);
void history_copy_series(hist_tier_id_t tier, hist_view_t v, int node, int series, uint16_t pos,
                         uint16_t n, uint16_t *out);
