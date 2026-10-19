#include "demo_home.h"

#include <math.h>
#include <stdio.h>
#include <string.h>

enum { TOD_MORNING, TOD_AFTERNOON, TOD_EVENING, TOD_NIGHT };
enum { CAR_AWAY, CAR_PARKED, CAR_RUNNING };

#define MAX_AREAS 16

static int s_tod = -1, s_anyone = -1;
static struct {
    int presence, light, pm25;
    int metric[METRIC_COUNT];  // node-room areas: entity for each node payload field
} s_area[MAX_AREAS];

static int find_in_area(int area, const char *field)
{
    for (int e = 0; e < DEMO_ENTITY_COUNT; e++) {
        if (DEMO_ENTITIES[e].area == area && strcmp(DEMO_ENTITIES[e].field, field) == 0) {
            return e;
        }
    }
    return -1;
}

void demo_home_init(void)
{
    s_tod = demo_entity_find("home.time_of_day");
    s_anyone = demo_entity_find("home.anyone_home");
    for (int a = 0; a < DEMO_AREA_COUNT && a < MAX_AREAS; a++) {
        s_area[a].presence = find_in_area(a, "presence");
        s_area[a].light = find_in_area(a, "light_on");
        s_area[a].pm25 = find_in_area(a, "pm2_5_ugm3");
        for (int m = 0; m < METRIC_COUNT; m++) {
            s_area[a].metric[m] = find_in_area(a, METRIC_INFO[m].key);
        }
    }
}

int demo_entity_find(const char *key)
{
    for (int e = 0; e < DEMO_ENTITY_COUNT; e++) {
        if (strcmp(DEMO_ENTITIES[e].key, key) == 0) {
            return e;
        }
    }
    return -1;
}

int demo_preset_find(const char *name)
{
    for (int p = 0; p < DEMO_PRESET_COUNT; p++) {
        if (strcmp(DEMO_PRESETS[p].name, name) == 0) {
            return p;
        }
    }
    return -1;
}

// ---------------------------------------------------------------------------
// Random numbers (xorshift32: small, and the same sequence on any host)

static uint32_t next(uint32_t *r)
{
    uint32_t x = *r ? *r : 0x2545F491u;
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    *r = x;
    return x;
}

static float rnd01(uint32_t *r)
{
    return (float)(next(r) >> 8) * (1.0f / 16777216.0f);
}

static float uniform(uint32_t *r, float a, float b)
{
    return a + (b - a) * rnd01(r);
}

static float tidy(int e, float v)
{
    const demo_entity_t *d = &DEMO_ENTITIES[e];
    if (d->type == DEMO_NUM) {
        float p = powf(10.0f, d->dec);
        v = roundf(v * p) / p;
    }
    return v < d->min ? d->min : v > d->max ? d->max : v;
}

static float pick_option(uint32_t *r, int n, uint16_t mask)
{
    if (!mask) {
        mask = (uint16_t)((1u << n) - 1);
    }
    int count = 0;
    for (int i = 0; i < n; i++) {
        count += (mask >> i) & 1;
    }
    int k = (int)(next(r) % (uint32_t)count);
    for (int i = 0; i < n; i++) {
        if (((mask >> i) & 1) && k-- == 0) {
            return (float)i;
        }
    }
    return 0;
}

// ---------------------------------------------------------------------------
// Fill, abnormal and derive rules

static float fill_value(int e, const float *v, uint32_t *r)
{
    const demo_entity_t *d = &DEMO_ENTITIES[e];
    const demo_area_t *a = &DEMO_AREAS[d->area];
    bool anyone = s_anyone < 0 || v[s_anyone] >= 0.5f;
    int tod = s_tod < 0 ? TOD_AFTERNOON : (int)v[s_tod];

    switch (d->fill) {
    case FILL_PRESENCE:
        if (!anyone) {
            return 0;
        }
        if (tod == TOD_NIGHT) {
            return rnd01(r) < (a->sleep_area ? 0.9f : 0.05f);
        }
        return rnd01(r) < d->nmin;
    case FILL_LIGHT: {
        int pe = s_area[d->area].presence;
        if (pe < 0 || v[pe] < 0.5f) {
            return 0;
        }
        switch (tod) {
        case TOD_NIGHT:
            return a->sleep_area ? 0 : 1;  // asleep in the bedrooms
        case TOD_EVENING:
            return 1;
        case TOD_MORNING:
            return rnd01(r) < 0.5f;
        default:
            return rnd01(r) < 0.2f;
        }
    }
    case FILL_LOCK:
        return (!anyone || tod == TOD_NIGHT) ? 1 : (rnd01(r) < 0.5f);
    case FILL_CAR:
        return !anyone ? CAR_AWAY : (rnd01(r) < 0.8f ? CAR_PARKED : CAR_AWAY);
    case FILL_OFF:
        return 0;
    default:
        break;
    }
    switch (d->type) {
    case DEMO_NUM:
        return tidy(e, uniform(r, d->nmin, d->nmax));
    case DEMO_BOOL:
        return rnd01(r) < d->nmin;
    default:
        return pick_option(r, d->n_options, d->normal_mask);
    }
}

static float any_value(int e, uint32_t *r)
{
    const demo_entity_t *d = &DEMO_ENTITIES[e];
    switch (d->type) {
    case DEMO_NUM:
        return tidy(e, uniform(r, d->min, d->max));
    case DEMO_BOOL:
        return rnd01(r) < 0.5f;
    default:
        return pick_option(r, d->n_options, 0);
    }
}

static float derive(int e, const float *v, uint32_t *r)
{
    const demo_entity_t *d = &DEMO_ENTITIES[e];
    int pm25 = s_area[d->area].pm25, light = s_area[d->area].light;
    int pres = s_area[d->area].presence;
    int tod = s_tod < 0 ? TOD_AFTERNOON : (int)v[s_tod];
    switch (d->derive) {
    case DERIVE_PM1:
        return tidy(e, pm25 < 0 ? 0 : v[pm25] * 0.65f);
    case DERIVE_PM10:
        return tidy(e, pm25 < 0 ? 0 : v[pm25] * 1.4f);
    case DERIVE_LUX:
        if (light >= 0 && v[light] >= 0.5f) {
            return tidy(e, uniform(r, 250, 600));
        }
        switch (tod) {
        case TOD_MORNING:
            return tidy(e, uniform(r, 100, 500));
        case TOD_AFTERNOON:
            return tidy(e, uniform(r, 200, 800));
        case TOD_EVENING:
            return tidy(e, uniform(r, 5, 60));
        default:
            return tidy(e, uniform(r, 0, 3));
        }
    case DERIVE_DISTANCE:
        return (pres >= 0 && v[pres] >= 0.5f) ? tidy(e, uniform(r, 300, 900))
                                              : tidy(e, uniform(r, 1000, 1200));
    default:
        return v[e];
    }
}

// ---------------------------------------------------------------------------

void demo_fill_preset(demo_values_t *out, int p)
{
    bool set[DEMO_MAX_ENTITIES] = {false};
    uint32_t r = 0x9E3779B9u ^ ((uint32_t)(p + 1) * 2654435761u);
    memset(out, 0, sizeof(*out));
    if (p >= 0 && p < DEMO_PRESET_COUNT) {
        const demo_preset_t *pr = &DEMO_PRESETS[p];
        for (int i = 0; i < pr->n_sets; i++) {
            out->v[pr->sets[i].entity] = pr->sets[i].value;
            set[pr->sets[i].entity] = true;
        }
    }
    // Table order puts the home area (time of day, anyone home) first, and presence before
    // lights within an area (checked by gen_demo.py), so each rule sees what it depends on.
    for (int e = 0; e < DEMO_ENTITY_COUNT; e++) {
        if (!set[e] && DEMO_ENTITIES[e].derive == DERIVE_NONE) {
            out->v[e] = fill_value(e, out->v, &r);
        }
    }
    for (int e = 0; e < DEMO_ENTITY_COUNT; e++) {
        if (!set[e] && DEMO_ENTITIES[e].derive != DERIVE_NONE) {
            out->v[e] = derive(e, out->v, &r);
        }
    }
}

void demo_roll(demo_values_t *out, uint32_t seed, demo_spread_t spread, uint8_t abnormal_pct)
{
    uint32_t r = seed ? seed : 1;
    memset(out, 0, sizeof(*out));
    for (int e = 0; e < DEMO_ENTITY_COUNT; e++) {
        if (DEMO_ENTITIES[e].derive != DERIVE_NONE) {
            continue;
        }
        bool wild = spread == SPREAD_UNIFORM || rnd01(&r) * 100.0f < (float)abnormal_pct;
        out->v[e] = wild ? any_value(e, &r) : fill_value(e, out->v, &r);
    }
    for (int e = 0; e < DEMO_ENTITY_COUNT; e++) {
        if (DEMO_ENTITIES[e].derive != DERIVE_NONE) {
            out->v[e] = derive(e, out->v, &r);
        }
    }
}

int demo_pick_preset(uint32_t *rng, int last)
{
    if (DEMO_PRESET_COUNT <= 1) {
        return 0;
    }
    int p;
    do {
        p = (int)(next(rng) % (uint32_t)DEMO_PRESET_COUNT);
    } while (p == last);
    return p;
}

bool demo_is_normal(int e, float v)
{
    const demo_entity_t *d = &DEMO_ENTITIES[e];
    switch (d->type) {
    case DEMO_NUM:
        return d->derive != DERIVE_NONE || (v >= d->nmin && v <= d->nmax);
    case DEMO_ENUM:
        return !d->normal_mask || ((d->normal_mask >> (int)v) & 1);
    default:
        return true;
    }
}

int demo_json_value(int e, float v, char *buf, size_t len)
{
    const demo_entity_t *d = &DEMO_ENTITIES[e];
    switch (d->type) {
    case DEMO_NUM:
        if (d->dec == 0) {
            return snprintf(buf, len, "%ld", (long)lroundf(v));
        }
        return snprintf(buf, len, "%.*f", d->dec, (double)v);
    case DEMO_BOOL:
        return snprintf(buf, len, "%s", v >= 0.5f ? "true" : "false");
    default: {
        int i = (int)v;
        return snprintf(buf, len, "\"%s\"", (i >= 0 && i < d->n_options) ? d->options[i] : "");
    }
    }
}

bool demo_node_reading(const demo_values_t *h, int area, sim_reading_t *out)
{
    if (area < 0 || area >= DEMO_AREA_COUNT || !DEMO_AREAS[area].node_room[0]) {
        return false;
    }
    for (int m = 0; m < METRIC_COUNT; m++) {
        int e = s_area[area].metric[m];
        out->v[m] = e >= 0 ? h->v[e] : 0.0f;
    }
    int pe = s_area[area].presence;
    out->presence = pe >= 0 && h->v[pe] >= 0.5f;
    return true;
}
