#include "sim_model.h"

#include <math.h>
#include <stddef.h>
#include <string.h>

#define OUTDOOR_CO2_PPM 420.0f
#define PI_F 3.14159265f

const metric_info_t METRIC_INFO[METRIC_COUNT] = {
    //                 key            label          unit     min   max     dec noise  rel    tau_auto tau_forced
    [METRIC_TEMPERATURE] = {"temperature_c", "Temperature", "°C",     0,    50,     1, 0.05f, 0,     900,   180},
    [METRIC_HUMIDITY]    = {"humidity_pct",  "Humidity",    "%",      0,    100,    1, 0.3f,  0,     900,   180},
    [METRIC_VOC]         = {"voc_index",     "VOC Index",   "",       1,    500,    0, 2.0f,  0,     300,   60},
    [METRIC_CO2]         = {"co2_ppm",       "CO₂",         "ppm",    400,  5000,   0, 6.0f,  0,     0,     120},
    [METRIC_LUX]         = {"lux",           "Light",       "lux",    0,    65535,  1, 0.3f,  0.02f, 4,     4},
    [METRIC_DISTANCE]    = {"distance_mm",   "Distance",    "mm",     30,   1200,   0, 4.0f,  0,     2,     2},
    [METRIC_PM1_0]       = {"pm1_0_ugm3",    "PM1.0",       "µg/m³",  0,    500,    0, 0.4f,  0.05f, 0,     60},
    [METRIC_PM2_5]       = {"pm2_5_ugm3",    "PM2.5",       "µg/m³",  0,    500,    0, 0.6f,  0.05f, 600,   60},
    [METRIC_PM10]        = {"pm10_ugm3",     "PM10",        "µg/m³",  0,    500,    0, 0.8f,  0.05f, 0,     60},
};

int sim_metric_from_key(const char *key)
{
    for (int m = 0; m < METRIC_COUNT; m++) {
        if (strcmp(key, METRIC_INFO[m].key) == 0) {
            return m;
        }
    }
    return -1;
}

// ---------------------------------------------------------------------------
// Room profiles

const room_profile_t ROOM_KITCHEN = {
    .name = "kitchen", .label = "Kitchen",
    .temp_base = 22.0f, .rh_base = 45.0f, .voc_base = 110.0f, .pm25_base = 6.0f,
    .co2_per_person_ppm_s = 0.125f, .air_changes_per_h = 1.0f,
    .window_lux = 250.0f, .lamp_lux = 400.0f, .lights_threshold_lux = 200.0f,
    .dist_background_mm = 1100.0f, .pm10_ratio = 1.5f,
    .max_occupants = 2, .sleeping_room = false, .cooks = true,
    .voc_events_per_h = 0.0f, .dust_events_per_h = 0.2f,
    .occ_prob = {0.02f, 0.02f, 0.02f, 0.02f, 0.02f, 0.05f, 0.30f, 0.60f, 0.40f, 0.15f, 0.10f, 0.30f,
                 0.60f, 0.30f, 0.10f, 0.10f, 0.20f, 0.60f, 0.80f, 0.60f, 0.30f, 0.20f, 0.10f, 0.05f},
    .dwell_occ_min_s = 5 * 60, .dwell_occ_max_s = 25 * 60,
    .dwell_empty_min_s = 10 * 60, .dwell_empty_max_s = 60 * 60,
};

const room_profile_t ROOM_BEDROOM = {
    .name = "bedroom", .label = "Bedroom",
    .temp_base = 21.0f, .rh_base = 48.0f, .voc_base = 90.0f, .pm25_base = 3.0f,
    .co2_per_person_ppm_s = 0.167f, .air_changes_per_h = 0.5f,
    .window_lux = 150.0f, .lamp_lux = 200.0f, .lights_threshold_lux = 150.0f,
    .dist_background_mm = 1150.0f, .pm10_ratio = 1.3f,
    .max_occupants = 2, .sleeping_room = true, .cooks = false,
    .voc_events_per_h = 0.05f, .dust_events_per_h = 0.05f,
    .occ_prob = {0.95f, 0.95f, 0.95f, 0.95f, 0.95f, 0.95f, 0.80f, 0.50f, 0.20f, 0.10f, 0.10f, 0.10f,
                 0.10f, 0.10f, 0.10f, 0.10f, 0.10f, 0.10f, 0.10f, 0.10f, 0.15f, 0.30f, 0.60f, 0.90f},
    .dwell_occ_min_s = 20 * 60, .dwell_occ_max_s = 90 * 60,
    .dwell_empty_min_s = 15 * 60, .dwell_empty_max_s = 90 * 60,
};

const room_profile_t ROOM_LIVING_ROOM = {
    .name = "living-room", .label = "Living Room",
    .temp_base = 22.5f, .rh_base = 42.0f, .voc_base = 100.0f, .pm25_base = 5.0f,
    .co2_per_person_ppm_s = 0.083f, .air_changes_per_h = 0.7f,
    .window_lux = 450.0f, .lamp_lux = 300.0f, .lights_threshold_lux = 250.0f,
    .dist_background_mm = 1050.0f, .pm10_ratio = 1.4f,
    .max_occupants = 4, .sleeping_room = false, .cooks = false,
    .voc_events_per_h = 0.5f, .dust_events_per_h = 0.3f,
    .occ_prob = {0.03f, 0.03f, 0.03f, 0.03f, 0.03f, 0.03f, 0.10f, 0.20f, 0.20f, 0.25f, 0.25f, 0.25f,
                 0.30f, 0.30f, 0.30f, 0.30f, 0.30f, 0.50f, 0.70f, 0.80f, 0.80f, 0.70f, 0.40f, 0.10f},
    .dwell_occ_min_s = 15 * 60, .dwell_occ_max_s = 90 * 60,
    .dwell_empty_min_s = 10 * 60, .dwell_empty_max_s = 60 * 60,
};

// Automatic events: milder than the manual scenarios of the same kind.
static const scenario_t AUTO_COOKING = {
    .name = "cooking", .presence = 1, .occupants = 0, .lights = 1,
    .m = {
        [METRIC_TEMPERATURE] = {SC_DELTA, 2.0f, 0},
        [METRIC_HUMIDITY] = {SC_DELTA, 12.0f, 0},
        [METRIC_VOC] = {SC_RANGE, 200, 320},
        [METRIC_PM2_5] = {SC_RANGE, 25, 60},
    },
};
static const scenario_t AUTO_VOC = {
    .name = "candle/cleaning", .presence = -1, .occupants = 0, .lights = -1,
    .m = {
        [METRIC_VOC] = {SC_RANGE, 160, 260},
        [METRIC_PM2_5] = {SC_DELTA, 8.0f, 0},
    },
};
static const scenario_t AUTO_DUST = {
    .name = "dust", .presence = -1, .occupants = 0, .lights = -1,
    .m = {
        [METRIC_PM2_5] = {SC_RANGE, 15, 35},
    },
};

// ---------------------------------------------------------------------------
// Helpers

static uint32_t rnd_u32(sim_state_t *s)
{
    uint32_t x = s->rng;  // xorshift32
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    s->rng = x;
    return x;
}

static float rnd01(sim_state_t *s)
{
    return (float)(rnd_u32(s) >> 8) * (1.0f / 16777216.0f);
}

static float rnd_range(sim_state_t *s, float a, float b)
{
    return a + (b - a) * rnd01(s);
}

// Approximately N(0, 1)
static float rnd_gauss(sim_state_t *s)
{
    return (rnd01(s) + rnd01(s) + rnd01(s) + rnd01(s) - 2.0f) * 1.7320508f;
}

static float clampf(float x, float lo, float hi)
{
    return x < lo ? lo : (x > hi ? hi : x);
}

// First-order lag toward target; exact for any dt, so large sim-speed steps stay stable.
static float approach(float v, float target, float dt, float tau)
{
    if (tau <= 0.0f) {
        return target;
    }
    return v + (target - v) * (1.0f - expf(-dt / tau));
}

// 0..1, sunrise ~6:30, sunset ~19:30
static float daylight(float hour)
{
    if (hour < 6.5f || hour > 19.5f) {
        return 0.0f;
    }
    return sinf((hour - 6.5f) / 13.0f * PI_F);
}

static bool is_night(float hour)
{
    return hour >= 23.0f || hour < 6.5f;
}

static bool is_meal_time(float hour)
{
    return (hour >= 6.5f && hour < 9.0f) || (hour >= 11.5f && hour < 13.5f) ||
           (hour >= 17.0f && hour < 20.0f);
}

static const scenario_t *active_scenario(const sim_state_t *s)
{
    return s->scenario ? s->scenario : s->auto_event;
}

static bool overrides_active(const sim_state_t *s)
{
    if (s->presence_ovr >= 0) {
        return true;
    }
    for (int m = 0; m < METRIC_COUNT; m++) {
        if (s->ovr[m].active) {
            return true;
        }
    }
    return false;
}

static int effective_occupants(const sim_state_t *s)
{
    int occ = s->occupants;
    const scenario_t *sc = active_scenario(s);
    if (sc) {
        if (sc->occupants > 0) {
            occ = sc->occupants;
        }
        if (sc->presence == 0) {
            occ = 0;
        } else if (sc->presence == 1 && occ == 0) {
            occ = 1;
        }
    }
    if (s->presence_ovr == 0) {
        occ = 0;
    } else if (s->presence_ovr == 1 && occ == 0) {
        occ = 1;
    }
    return occ;
}

bool sim_presence(const sim_state_t *s)
{
    return effective_occupants(s) > 0;
}

int sim_occupants(const sim_state_t *s)
{
    return effective_occupants(s);
}

const char *sim_mode(const sim_state_t *s)
{
    if (overrides_active(s)) {
        return "override";
    }
    return s->scenario ? "scenario" : "auto";
}

// Forced behaviour for a metric: manual override > manual scenario > automatic event.
static const sc_metric_t *forced_metric(const sim_state_t *s, int m, sc_metric_t *tmp)
{
    if (s->ovr[m].active) {
        tmp->kind = SC_RANGE;
        tmp->a = s->ovr[m].min;
        tmp->b = s->ovr[m].max;
        return tmp;
    }
    const scenario_t *sc = active_scenario(s);
    if (sc && sc->m[m].kind != SC_NONE) {
        return &sc->m[m];
    }
    return NULL;
}

// Target that wanders inside [a, b] so forced values still look like live readings.
static float wander_target(sim_state_t *s, int m, float a, float b, float dt)
{
    if (a > b) {
        float t = a;
        a = b;
        b = t;
    }
    s->wander_timer_s[m] -= dt;
    if (s->wander_timer_s[m] <= 0.0f || s->wander[m] < a || s->wander[m] > b) {
        s->wander[m] = rnd_range(s, a, b);
        s->wander_timer_s[m] = rnd_range(s, 30.0f, 90.0f);
    }
    return s->wander[m];
}

static void start_auto_event(sim_state_t *s, const scenario_t *ev, float min_s, float max_s)
{
    float dur = rnd_range(s, min_s, max_s);
    if (s->occ_timer_s > 60.0f && dur > s->occ_timer_s) {
        dur = s->occ_timer_s;  // an event ends when the occupants leave
    }
    s->auto_event = ev;
    s->auto_event_left_s = dur;
    for (int m = 0; m < METRIC_COUNT; m++) {
        s->wander_timer_s[m] = 0.0f;
    }
}

// ---------------------------------------------------------------------------
// Public API

void sim_init(sim_state_t *s, const room_profile_t *profile, uint32_t seed)
{
    memset(s, 0, sizeof(*s));
    s->profile = profile;
    s->rng = seed ? seed : 0x9E3779B9u;
    sim_reset(s);
}

void sim_reset(sim_state_t *s)
{
    const room_profile_t *p = s->profile;
    sim_clear(s);
    memset(s->drift, 0, sizeof(s->drift));
    s->v[METRIC_TEMPERATURE] = p->temp_base + rnd_range(s, -0.3f, 0.3f);
    s->v[METRIC_HUMIDITY] = p->rh_base + rnd_range(s, -2.0f, 2.0f);
    s->v[METRIC_VOC] = p->voc_base;
    s->v[METRIC_CO2] = 440.0f + rnd_range(s, 0.0f, 60.0f);
    s->v[METRIC_LUX] = 0.0f;
    s->v[METRIC_DISTANCE] = p->dist_background_mm;
    s->v[METRIC_PM2_5] = p->pm25_base;
    s->v[METRIC_PM1_0] = p->pm25_base * 0.65f;
    s->v[METRIC_PM10] = p->pm25_base * p->pm10_ratio;
    s->cloud = 0.8f;
    s->occupants = 0;
    s->occ_timer_s = 0.0f;  // decide occupancy on the first step
    s->prev_effective_occ = 0;
    s->near_object = false;
    s->near_timer_s = 0.0f;
}

void sim_clear(sim_state_t *s)
{
    s->scenario = NULL;
    s->scenario_left_s = 0.0f;
    s->scenario_forever = false;
    s->auto_event = NULL;
    s->auto_event_left_s = 0.0f;
    memset(s->ovr, 0, sizeof(s->ovr));
    s->presence_ovr = -1;
    s->ovr_left_s = 0.0f;
    s->ovr_forever = false;
    for (int m = 0; m < METRIC_COUNT; m++) {
        s->wander_timer_s[m] = 0.0f;
    }
}

void sim_start_scenario(sim_state_t *s, const scenario_t *sc, int32_t duration_s)
{
    uint32_t dur = duration_s < 0 ? sc->duration_s : (uint32_t)duration_s;
    s->auto_event = NULL;
    s->scenario = sc;
    s->scenario_forever = (dur == 0);
    s->scenario_left_s = (float)dur;
    for (int m = 0; m < METRIC_COUNT; m++) {
        s->wander_timer_s[m] = 0.0f;
    }
}

void sim_apply_override(sim_state_t *s, const sim_override_req_t *req, uint32_t duration_s,
                        bool instant)
{
    for (int m = 0; m < METRIC_COUNT; m++) {
        if (!req->m[m].active) {
            continue;
        }
        s->ovr[m] = req->m[m];
        s->wander_timer_s[m] = 0.0f;
        if (instant) {
            s->wander[m] = rnd_range(s, req->m[m].min, req->m[m].max);
            s->wander_timer_s[m] = rnd_range(s, 30.0f, 90.0f);
            s->v[m] = s->wander[m];
        }
    }
    if (req->presence >= 0) {
        s->presence_ovr = req->presence;
    }
    s->ovr_forever = (duration_s == 0);
    s->ovr_left_s = (float)duration_s;
}

static void update_timers(sim_state_t *s, float dt)
{
    if (s->scenario && !s->scenario_forever) {
        s->scenario_left_s -= dt;
        if (s->scenario_left_s <= 0.0f) {
            s->scenario = NULL;
        }
    }
    if (s->auto_event) {
        s->auto_event_left_s -= dt;
        if (s->auto_event_left_s <= 0.0f) {
            s->auto_event = NULL;
        }
    }
    if (!s->ovr_forever && overrides_active(s)) {
        s->ovr_left_s -= dt;
        if (s->ovr_left_s <= 0.0f) {
            memset(s->ovr, 0, sizeof(s->ovr));
            s->presence_ovr = -1;
        }
    }
}

static void update_occupancy(sim_state_t *s, float dt, float hour)
{
    const room_profile_t *p = s->profile;
    s->occ_timer_s -= dt;
    if (s->occ_timer_s > 0.0f) {
        return;
    }
    bool was_occupied = s->occupants > 0;
    bool occupied = rnd01(s) < p->occ_prob[(int)hour % 24];
    if (occupied) {
        float r = rnd01(s);
        int n = 1 + (int)(r * r * p->max_occupants);  // biased toward fewer people
        s->occupants = n > p->max_occupants ? p->max_occupants : n;
        if (p->sleeping_room && is_night(hour)) {
            s->occ_timer_s = rnd_range(s, 2 * 3600.0f, 4 * 3600.0f);
        } else {
            s->occ_timer_s = rnd_range(s, p->dwell_occ_min_s, p->dwell_occ_max_s);
        }
    } else {
        s->occupants = 0;
        s->occ_timer_s = rnd_range(s, p->dwell_empty_min_s, p->dwell_empty_max_s);
    }

    if (occupied && !was_occupied && !s->scenario && !s->auto_event && p->cooks &&
        is_meal_time(hour) && rnd01(s) < 0.7f) {
        start_auto_event(s, &AUTO_COOKING, 15 * 60.0f, 40 * 60.0f);
    }
}

static void update_random_events(sim_state_t *s, float dt, int occ)
{
    const room_profile_t *p = s->profile;
    if (occ == 0 || s->scenario || s->auto_event) {
        return;
    }
    if (rnd01(s) < p->voc_events_per_h / 3600.0f * dt) {
        start_auto_event(s, &AUTO_VOC, 10 * 60.0f, 25 * 60.0f);
    } else if (rnd01(s) < p->dust_events_per_h / 3600.0f * dt) {
        start_auto_event(s, &AUTO_DUST, 5 * 60.0f, 15 * 60.0f);
    }
}

static bool lights_on(const sim_state_t *s, int occ, float hour)
{
    const room_profile_t *p = s->profile;
    const scenario_t *sc = active_scenario(s);
    if (sc && sc->lights >= 0) {
        return sc->lights == 1;
    }
    if (occ == 0) {
        return false;
    }
    if (p->sleeping_room && is_night(hour)) {
        return false;
    }
    return p->window_lux * daylight(hour) * s->cloud < p->lights_threshold_lux;
}

static void update_drift(sim_state_t *s, float dt)
{
    static const float sigma[METRIC_COUNT] = {
        [METRIC_TEMPERATURE] = 0.004f, [METRIC_HUMIDITY] = 0.03f,
        [METRIC_VOC] = 0.15f, [METRIC_PM2_5] = 0.02f,
    };
    static const float limit[METRIC_COUNT] = {
        [METRIC_TEMPERATURE] = 0.4f, [METRIC_HUMIDITY] = 3.0f,
        [METRIC_VOC] = 15.0f, [METRIC_PM2_5] = 2.0f,
    };
    float k = sqrtf(dt);
    for (int m = 0; m < METRIC_COUNT; m++) {
        if (sigma[m] > 0.0f) {
            s->drift[m] = clampf(s->drift[m] + rnd_gauss(s) * sigma[m] * k, -limit[m], limit[m]);
        }
    }
    s->cloud = clampf(s->cloud + rnd_gauss(s) * 0.002f * k, 0.5f, 1.0f);
}

static float auto_target(const sim_state_t *s, int m, int occ, float hour, bool lights)
{
    const room_profile_t *p = s->profile;
    float diurnal = sinf((hour - 10.0f) * PI_F / 12.0f);  // peaks ~16:00
    switch (m) {
    case METRIC_TEMPERATURE:
        return p->temp_base + 0.8f * diurnal + 0.25f * occ + (lights ? 0.1f : 0.0f) + s->drift[m];
    case METRIC_HUMIDITY:
        return p->rh_base - 2.0f * diurnal + 0.6f * occ + s->drift[m];
    case METRIC_VOC:
        return p->voc_base + 5.0f * occ + s->drift[m];
    case METRIC_LUX:
        return p->window_lux * daylight(hour) * s->cloud + (lights ? p->lamp_lux : 0.0f) + 0.3f;
    case METRIC_PM2_5:
        return p->pm25_base + 0.8f * occ + s->drift[m];
    default:
        return s->v[m];
    }
}

static void update_distance(sim_state_t *s, float dt, int occ)
{
    if (occ > 0) {
        if (s->prev_effective_occ == 0) {
            // Someone just walked in: they pass close to the unit.
            s->near_object = true;
            s->near_mm = rnd_range(s, 300.0f, 900.0f);
            s->near_timer_s = rnd_range(s, 10.0f, 60.0f);
        }
        s->near_timer_s -= dt;
        if (s->near_timer_s <= 0.0f) {
            s->near_object = !s->near_object && rnd01(s) < 0.5f;
            s->near_timer_s = s->near_object ? rnd_range(s, 10.0f, 120.0f)
                                             : rnd_range(s, 20.0f, 300.0f);
            if (s->near_object) {
                s->near_mm = rnd_range(s, 300.0f, 900.0f);
            }
        }
    } else {
        s->near_object = false;
    }
}

void sim_step(sim_state_t *s, float dt, float hour)
{
    const room_profile_t *p = s->profile;
    if (dt <= 0.0f) {
        return;
    }
    hour = fmodf(hour, 24.0f);
    if (hour < 0.0f) {
        hour += 24.0f;
    }

    update_timers(s, dt);
    update_occupancy(s, dt, hour);
    int occ = effective_occupants(s);
    update_random_events(s, dt, occ);
    update_drift(s, dt);
    update_distance(s, dt, occ);
    bool lights = lights_on(s, occ, hour);

    static const int smooth_metrics[] = {METRIC_TEMPERATURE, METRIC_HUMIDITY, METRIC_VOC,
                                         METRIC_LUX, METRIC_PM2_5};
    for (size_t i = 0; i < sizeof(smooth_metrics) / sizeof(smooth_metrics[0]); i++) {
        int m = smooth_metrics[i];
        sc_metric_t tmp;
        const sc_metric_t *f = forced_metric(s, m, &tmp);
        float target, tau;
        if (f && f->kind == SC_RANGE) {
            target = wander_target(s, m, f->a, f->b, dt);
            tau = METRIC_INFO[m].tau_forced_s;
        } else {
            target = auto_target(s, m, occ, hour, lights);
            tau = METRIC_INFO[m].tau_auto_s;
            if (f && f->kind == SC_DELTA) {
                target += f->a;
                tau = METRIC_INFO[m].tau_forced_s;
            }
        }
        s->v[m] = approach(s->v[m], target, dt, tau);
    }

    // CO2: mass balance between occupants breathing and ventilation.
    {
        sc_metric_t tmp;
        const sc_metric_t *f = forced_metric(s, METRIC_CO2, &tmp);
        if (f && f->kind == SC_RANGE) {
            float target = wander_target(s, METRIC_CO2, f->a, f->b, dt);
            s->v[METRIC_CO2] = approach(s->v[METRIC_CO2], target, dt,
                                        METRIC_INFO[METRIC_CO2].tau_forced_s);
        } else {
            float gen = occ * p->co2_per_person_ppm_s;
            float vent = (s->v[METRIC_CO2] - OUTDOOR_CO2_PPM) * p->air_changes_per_h / 3600.0f;
            s->v[METRIC_CO2] += (gen - vent) * dt;
        }
    }

    // Distance: background wall/furniture, or someone close to the unit.
    {
        sc_metric_t tmp;
        const sc_metric_t *f = forced_metric(s, METRIC_DISTANCE, &tmp);
        float target;
        if (f && f->kind == SC_RANGE) {
            target = wander_target(s, METRIC_DISTANCE, f->a, f->b, dt);
        } else {
            target = s->near_object ? s->near_mm : p->dist_background_mm;
        }
        s->v[METRIC_DISTANCE] = approach(s->v[METRIC_DISTANCE], target, dt,
                                         METRIC_INFO[METRIC_DISTANCE].tau_auto_s);
    }

    // PM1.0 / PM10 follow PM2.5 unless forced on their own.
    {
        static const int derived[] = {METRIC_PM1_0, METRIC_PM10};
        for (size_t i = 0; i < 2; i++) {
            int m = derived[i];
            sc_metric_t tmp;
            const sc_metric_t *f = forced_metric(s, m, &tmp);
            if (f && f->kind == SC_RANGE) {
                float target = wander_target(s, m, f->a, f->b, dt);
                s->v[m] = approach(s->v[m], target, dt, METRIC_INFO[m].tau_forced_s);
            } else {
                float ratio = (m == METRIC_PM1_0) ? 0.65f : p->pm10_ratio;
                s->v[m] = s->v[METRIC_PM2_5] * ratio;
            }
        }
    }

    for (int m = 0; m < METRIC_COUNT; m++) {
        s->v[m] = clampf(s->v[m], METRIC_INFO[m].min, METRIC_INFO[m].max);
    }
    s->prev_effective_occ = occ;
}

static float round_to(float x, int decimals)
{
    float scale = decimals == 0 ? 1.0f : (decimals == 1 ? 10.0f : 100.0f);
    return roundf(x * scale) / scale;
}

void sim_read(sim_state_t *s, sim_reading_t *out)
{
    for (int m = 0; m < METRIC_COUNT; m++) {
        const metric_info_t *mi = &METRIC_INFO[m];
        float v = s->v[m];
        float noisy = v + rnd_gauss(s) * mi->noise_abs + rnd_gauss(s) * mi->noise_rel * v;
        if (s->ovr[m].active) {
            // Once the value has reached a requested range, keep readings inside it
            // (min == max gives an exact, repeatable value).
            float eps = 0.5f / (mi->decimals == 0 ? 1.0f : 10.0f);
            if (v >= s->ovr[m].min - eps && v <= s->ovr[m].max + eps) {
                noisy = clampf(noisy, s->ovr[m].min, s->ovr[m].max);
            }
        }
        out->v[m] = round_to(clampf(noisy, mi->min, mi->max), mi->decimals);
    }
    // Keep the PMS5003 relationship PM1.0 <= PM2.5 <= PM10 unless those were forced apart.
    if (!s->ovr[METRIC_PM1_0].active && out->v[METRIC_PM1_0] > out->v[METRIC_PM2_5]) {
        out->v[METRIC_PM1_0] = out->v[METRIC_PM2_5];
    }
    if (!s->ovr[METRIC_PM10].active && out->v[METRIC_PM10] < out->v[METRIC_PM2_5]) {
        out->v[METRIC_PM10] = out->v[METRIC_PM2_5];
    }
    out->presence = sim_presence(s);
}
