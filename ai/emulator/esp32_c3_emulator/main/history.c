#include "history.h"

#include <math.h>
#include <string.h>

typedef struct {
    uint16_t cap;
    uint16_t count;
    uint16_t head;  // next slot to write
    uint32_t *t;
    uint16_t (*v)[HIST_NODES][HIST_SERIES];
} tier_t;

static uint32_t s_fine_t[HIST_FINE_CAP];
static uint16_t s_fine_v[HIST_FINE_CAP][HIST_NODES][HIST_SERIES];
static uint32_t s_coarse_t[HIST_COARSE_CAP];
static uint16_t s_coarse_v[HIST_COARSE_CAP][HIST_NODES][HIST_SERIES];

static tier_t s_tiers[HIST_TIER_COUNT] = {
    [HIST_FINE] = {HIST_FINE_CAP, 0, 0, s_fine_t, s_fine_v},
    [HIST_COARSE] = {HIST_COARSE_CAP, 0, 0, s_coarse_t, s_coarse_v},
};

// Coarse bucket being filled
static uint16_t s_bucket_s = 15;
static uint32_t s_bucket_start;
static bool s_bucket_open;
static uint16_t s_bucket_n;
static float s_sum[HIST_NODES][HIST_SERIES];
static bool s_present[HIST_NODES];

uint16_t history_scale(int series)
{
    if (series < 0 || series >= METRIC_COUNT) {
        return 1;
    }
    const metric_info_t *mi = &METRIC_INFO[series];
    // One decimal fits in uint16 only if max * 10 does (lux doesn't: stored as whole lux).
    return (mi->decimals > 0 && mi->max * 10.0f < 65535.0f) ? 10 : 1;
}

static uint16_t encode(int series, float v)
{
    float x = roundf(v * (float)history_scale(series));
    if (!(x >= 0.0f)) {  // also catches NaN
        return 0;
    }
    return x >= 65534.0f ? 65534 : (uint16_t)x;
}

static void push(tier_t *t, uint32_t t_s, const uint16_t v[HIST_NODES][HIST_SERIES])
{
    t->t[t->head] = t_s;
    memcpy(t->v[t->head], v, sizeof(t->v[0]));
    t->head = (uint16_t)((t->head + 1) % t->cap);
    if (t->count < t->cap) {
        t->count++;
    }
}

static void close_bucket(void)
{
    if (!s_bucket_open || s_bucket_n == 0) {
        return;
    }
    uint16_t v[HIST_NODES][HIST_SERIES];
    for (int n = 0; n < HIST_NODES; n++) {
        for (int s = 0; s < METRIC_COUNT; s++) {
            v[n][s] = encode(s, s_sum[n][s] / (float)s_bucket_n);
        }
        v[n][HIST_PRESENCE] = s_present[n] ? 1 : 0;
    }
    push(&s_tiers[HIST_COARSE], s_bucket_start, v);
    s_bucket_open = false;
}

void history_init(uint16_t coarse_bucket_s)
{
    s_bucket_s = coarse_bucket_s ? coarse_bucket_s : 15;
    for (int i = 0; i < HIST_TIER_COUNT; i++) {
        s_tiers[i].count = 0;
        s_tiers[i].head = 0;
    }
    s_bucket_open = false;
}

void history_record(uint32_t t_s, const sim_reading_t readings[HIST_NODES])
{
    uint16_t v[HIST_NODES][HIST_SERIES];
    for (int n = 0; n < HIST_NODES; n++) {
        for (int s = 0; s < METRIC_COUNT; s++) {
            v[n][s] = encode(s, readings[n].v[s]);
        }
        v[n][HIST_PRESENCE] = readings[n].presence ? 1 : 0;
    }
    push(&s_tiers[HIST_FINE], t_s, v);

    // Buckets are aligned to multiples of the bucket length since boot.
    uint32_t start = t_s - t_s % s_bucket_s;
    if (s_bucket_open && start != s_bucket_start) {
        close_bucket();
    }
    if (!s_bucket_open) {
        s_bucket_open = true;
        s_bucket_start = start;
        s_bucket_n = 0;
        memset(s_sum, 0, sizeof(s_sum));
        memset(s_present, 0, sizeof(s_present));
    }
    for (int n = 0; n < HIST_NODES; n++) {
        for (int s = 0; s < METRIC_COUNT; s++) {
            s_sum[n][s] += readings[n].v[s];
        }
        s_present[n] = s_present[n] || readings[n].presence;
    }
    s_bucket_n++;
}

uint16_t history_cap(hist_tier_id_t tier)
{
    return s_tiers[tier].cap;
}

uint16_t history_bucket_s(void)
{
    return s_bucket_s;
}

hist_view_t history_view(hist_tier_id_t tier)
{
    const tier_t *t = &s_tiers[tier];
    hist_view_t v;
    v.count = t->count < t->cap ? t->count : (uint16_t)(t->cap - 1);
    v.first = (uint16_t)((t->head + t->cap - v.count) % t->cap);
    return v;
}

void history_copy_times(hist_tier_id_t tier, hist_view_t v, uint16_t pos, uint16_t n,
                        uint32_t *out)
{
    const tier_t *t = &s_tiers[tier];
    for (uint16_t i = 0; i < n && pos + i < v.count; i++) {
        out[i] = t->t[(v.first + pos + i) % t->cap];
    }
}

void history_copy_series(hist_tier_id_t tier, hist_view_t v, int node, int series, uint16_t pos,
                         uint16_t n, uint16_t *out)
{
    const tier_t *t = &s_tiers[tier];
    for (uint16_t i = 0; i < n && pos + i < v.count; i++) {
        out[i] = t->v[(v.first + pos + i) % t->cap][node][series];
    }
}
