#include "demo_home.h"

#include <math.h>
#include <stdio.h>
#include <string.h>

enum { TOD_MORNING, TOD_AFTERNOON, TOD_EVENING, TOD_NIGHT };
enum { CAR_AWAY, CAR_PARKED, CAR_RUNNING };

#define MAX_AREAS 16
#define LIMIT_ABS 100000.0f  // no limit can go beyond +-this

static int s_tod = -1, s_anyone = -1;
static struct {
    int presence, light, pm25;
} s_area[MAX_AREAS];
static int s_nodes[DEMO_MAX_NODES];
static int s_node_count;
static demo_limit_t s_lim[DEMO_MAX_ENTITIES];

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
    s_node_count = 0;
    for (int a = 0; a < DEMO_AREA_COUNT && a < MAX_AREAS; a++) {
        s_area[a].presence = find_in_area(a, "presence");
        s_area[a].light = find_in_area(a, "light_on");
        s_area[a].pm25 = find_in_area(a, "pm2_5_ugm3");
        if (DEMO_AREAS[a].node_room[0] && s_node_count < DEMO_MAX_NODES) {
            s_nodes[s_node_count++] = a;
        }
    }
    demo_limits_reset();
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

static uint32_t fnv(uint32_t h, const char *s)
{
    for (; *s; s++) {
        h = (h ^ (uint8_t)*s) * 16777619u;
    }
    return (h ^ 0xFF) * 16777619u;
}

uint32_t demo_schema_hash(void)
{
    uint32_t h = 2166136261u;
    for (int e = 0; e < DEMO_ENTITY_COUNT; e++) {
        const demo_entity_t *d = &DEMO_ENTITIES[e];
        h = fnv(h, d->key);
        h = (h ^ d->type) * 16777619u;
        for (int i = 0; i < d->n_options; i++) {
            h = fnv(h, d->options[i]);
        }
    }
    return h;
}

int demo_node_count(void)
{
    return s_node_count;
}

int demo_node_area(int n)
{
    return (n >= 0 && n < s_node_count) ? s_nodes[n] : -1;
}

// ---------------------------------------------------------------------------
// Limits

const demo_limit_t *demo_limit(int e)
{
    return &s_lim[e];
}

void demo_limits_reset(void)
{
    for (int e = 0; e < DEMO_ENTITY_COUNT; e++) {
        const demo_entity_t *d = &DEMO_ENTITIES[e];
        s_lim[e] = (demo_limit_t){d->min, d->max, d->nmin, d->nmax};
    }
}

bool demo_limit_is_default(int e)
{
    const demo_entity_t *d = &DEMO_ENTITIES[e];
    const demo_limit_t *l = &s_lim[e];
    return l->min == d->min && l->max == d->max && l->nmin == d->nmin && l->nmax == d->nmax;
}

static float round_dec(float v, int dec)
{
    float p = powf(10.0f, (float)dec);
    return roundf(v * p) / p;
}

bool demo_limit_set(int e, const demo_limit_t *in, char *msg, size_t len)
{
    if (e < 0 || e >= DEMO_ENTITY_COUNT) {
        snprintf(msg, len, "unknown entity");
        return false;
    }
    const demo_entity_t *d = &DEMO_ENTITIES[e];
    demo_limit_t l = *in;
    if (!isfinite(l.min) || !isfinite(l.max) || !isfinite(l.nmin) || !isfinite(l.nmax)) {
        snprintf(msg, len, "%s: limits must be numbers", d->key);
        return false;
    }
    switch (d->type) {
    case DEMO_ENUM:
        snprintf(msg, len, "%s: options have no limits to edit", d->key);
        return false;
    case DEMO_BOOL:
        if (l.nmin < 0 || l.nmin > 1) {
            snprintf(msg, len, "%s: the chance of %s must be 0..1", d->key, d->options[1]);
            return false;
        }
        l.min = 0;
        l.max = 1;
        l.nmax = 0;
        break;
    default:
        l.min = round_dec(l.min, d->dec);
        l.max = round_dec(l.max, d->dec);
        l.nmin = round_dec(l.nmin, d->dec);
        l.nmax = round_dec(l.nmax, d->dec);
        if (fabsf(l.min) > LIMIT_ABS || fabsf(l.max) > LIMIT_ABS || !(l.min < l.max)) {
            snprintf(msg, len, "%s: caps must be low < high", d->key);
            return false;
        }
        if (d->derive == DERIVE_NONE && !(l.min <= l.nmin && l.nmin <= l.nmax && l.nmax <= l.max)) {
            snprintf(msg, len, "%s: the normal range must sit inside the caps", d->key);
            return false;
        }
        if (d->derive != DERIVE_NONE) {
            l.nmin = l.min;  // derived values have no normal range of their own
            l.nmax = l.max;
        }
        break;
    }
    s_lim[e] = l;
    return true;
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
        v = round_dec(v, d->dec);
    }
    return v < s_lim[e].min ? s_lim[e].min : v > s_lim[e].max ? s_lim[e].max : v;
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
    const demo_limit_t *l = &s_lim[e];
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
        return rnd01(r) < l->nmin;
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
    case FILL_BLINDS:
        switch (tod) {
        case TOD_NIGHT:
            return tidy(e, 0);
        case TOD_EVENING:
            return tidy(e, 10.0f * floorf(uniform(r, 0, 3.99f)));
        default:
            return tidy(e, 10.0f * floorf(uniform(r, 5, 10.99f)));
        }
    default:
        break;
    }
    switch (d->type) {
    case DEMO_NUM:
        return tidy(e, uniform(r, l->nmin, l->nmax));
    case DEMO_BOOL:
        return rnd01(r) < l->nmin;
    default:
        return pick_option(r, d->n_options, d->normal_mask);
    }
}

static float any_value(int e, uint32_t *r)
{
    const demo_entity_t *d = &DEMO_ENTITIES[e];
    switch (d->type) {
    case DEMO_NUM:
        return tidy(e, uniform(r, s_lim[e].min, s_lim[e].max));
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

void demo_scenario_from_builtin(demo_scenario_t *out, int b)
{
    memset(out, 0, sizeof(*out));
    const demo_builtin_t *p = &DEMO_BUILTIN[b];
    strlcpy(out->name, p->name, sizeof(out->name));
    out->group = p->group;
    out->flags = p->flags;
    strlcpy(out->expect, p->expect, sizeof(out->expect));
    out->n_sets = p->n_sets > DEMO_MAX_SETS ? DEMO_MAX_SETS : p->n_sets;
    memcpy(out->sets, p->sets, out->n_sets * sizeof(demo_set_t));
}

bool demo_value_check(int e, float v, char *msg, size_t len)
{
    if (e < 0 || e >= DEMO_ENTITY_COUNT) {
        snprintf(msg, len, "unknown entity");
        return false;
    }
    const demo_entity_t *d = &DEMO_ENTITIES[e];
    switch (d->type) {
    case DEMO_NUM:
        if (!isfinite(v) || v < s_lim[e].min || v > s_lim[e].max) {
            snprintf(msg, len, "%s must be %g..%g", d->key, (double)s_lim[e].min,
                     (double)s_lim[e].max);
            return false;
        }
        return true;
    case DEMO_BOOL:
        if (v != 0 && v != 1) {
            snprintf(msg, len, "%s must be true or false", d->key);
            return false;
        }
        return true;
    default:
        if (v != floorf(v) || v < 0 || v >= d->n_options) {
            snprintf(msg, len, "%s: unknown option", d->key);
            return false;
        }
        return true;
    }
}

// Fill pass of entity e: 0 time of day and anyone home (every rule reads them), 1 presence
// (the light rule reads it), 2 the rest, 3 derived values. Table order is the payload order, so
// the rules can't rely on it.
static int pass_of(int e)
{
    if (e == s_tod || e == s_anyone) {
        return 0;
    }
    if (DEMO_ENTITIES[e].derive != DERIVE_NONE) {
        return 3;
    }
    return DEMO_ENTITIES[e].fill == FILL_PRESENCE ? 1 : 2;
}

void demo_fill_scenario(demo_values_t *out, const demo_scenario_t *s)
{
    bool set[DEMO_MAX_ENTITIES] = {false};
    uint32_t r = fnv(0x9E3779B9u, s->name);  // same scenario, same values
    memset(out, 0, sizeof(*out));
    for (int i = 0; i < s->n_sets && i < DEMO_MAX_SETS; i++) {
        int e = s->sets[i].entity;
        if (e < DEMO_ENTITY_COUNT) {
            out->v[e] = tidy(e, s->sets[i].value);
            set[e] = true;
        }
    }
    for (int pass = 0; pass < 4; pass++) {
        for (int e = 0; e < DEMO_ENTITY_COUNT; e++) {
            if (!set[e] && pass_of(e) == pass) {
                out->v[e] = pass == 3 ? derive(e, out->v, &r) : fill_value(e, out->v, &r);
            }
        }
    }
}

void demo_roll(demo_values_t *out, uint32_t seed, demo_spread_t spread, uint8_t abnormal_pct)
{
    uint32_t r = seed ? seed : 1;
    memset(out, 0, sizeof(*out));
    for (int pass = 0; pass < 4; pass++) {
        for (int e = 0; e < DEMO_ENTITY_COUNT; e++) {
            if (pass_of(e) != pass) {
                continue;
            }
            if (pass == 3) {
                out->v[e] = derive(e, out->v, &r);
                continue;
            }
            bool wild = spread == SPREAD_UNIFORM || rnd01(&r) * 100.0f < (float)abnormal_pct;
            out->v[e] = wild ? any_value(e, &r) : fill_value(e, out->v, &r);
        }
    }
}

bool demo_is_normal(int e, float v)
{
    const demo_entity_t *d = &DEMO_ENTITIES[e];
    switch (d->type) {
    case DEMO_NUM:
        return d->derive != DERIVE_NONE || (v >= s_lim[e].nmin && v <= s_lim[e].nmax);
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

int demo_area_json(const demo_values_t *h, int area, const char *ts, char *buf, size_t len)
{
    int n = snprintf(buf, len, "{");
    bool first = true;
    for (int e = 0; e < DEMO_ENTITY_COUNT && n > 0 && (size_t)n < len; e++) {
        const demo_entity_t *d = &DEMO_ENTITIES[e];
        if (d->area != area || d->internal) {
            continue;
        }
        n += snprintf(buf + n, len - n, "%s\"%s\":", first ? "" : ",", d->field);
        if (n > 0 && (size_t)n < len) {
            n += demo_json_value(e, h->v[e], buf + n, len - n);
        }
        first = false;
    }
    if (ts && n > 0 && (size_t)n < len) {
        n += snprintf(buf + n, len - n, "%s\"ts\":\"%s\"", first ? "" : ",", ts);
    }
    if (n > 0 && (size_t)n < len) {
        n += snprintf(buf + n, len - n, "}");
    }
    return (n > 0 && (size_t)n < len) ? n : -1;
}
