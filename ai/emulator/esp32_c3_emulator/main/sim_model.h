// Stateful environmental simulation for one virtual room.
// Plain C (no ESP-IDF calls) so it can be unit-tested on a host.
#pragma once

#include <stdbool.h>
#include <stdint.h>

typedef enum {
    METRIC_TEMPERATURE,
    METRIC_HUMIDITY,
    METRIC_VOC,
    METRIC_CO2,
    METRIC_LUX,
    METRIC_DISTANCE,
    METRIC_PM1_0,
    METRIC_PM2_5,
    METRIC_PM10,
    METRIC_COUNT
} metric_t;

typedef struct {
    const char *key;    // JSON field name in the sensors payload
    const char *label;
    const char *unit;
    float min, max;     // what the real sensor can plausibly report
    uint8_t decimals;
    float noise_abs;    // measurement noise added to each reading
    float noise_rel;
    float tau_auto_s;   // time constant when following automatic behaviour
    float tau_forced_s; // time constant when steered by a scenario or override
} metric_info_t;

extern const metric_info_t METRIC_INFO[METRIC_COUNT];

int sim_metric_from_key(const char *key);  // -1 if unknown

typedef struct {
    const char *name;   // "kitchen"
    const char *label;  // "Kitchen"
    float temp_base, rh_base, voc_base, pm25_base;
    float co2_per_person_ppm_s;  // CO2 rise per occupant (depends on room volume)
    float air_changes_per_h;
    float window_lux, lamp_lux, lights_threshold_lux;
    float dist_background_mm;
    float pm10_ratio;
    uint8_t max_occupants;
    bool sleeping_room;          // lights off while occupied at night
    bool cooks;                  // cooking events at meal times
    float voc_events_per_h;      // candles / cleaning while occupied
    float dust_events_per_h;     // PM from activity while occupied
    float occ_prob[24];          // chance the room is occupied, by hour
    float dwell_occ_min_s, dwell_occ_max_s;
    float dwell_empty_min_s, dwell_empty_max_s;
} room_profile_t;

extern const room_profile_t ROOM_KITCHEN;
extern const room_profile_t ROOM_BEDROOM;
extern const room_profile_t ROOM_LIVING_ROOM;

typedef enum { SC_NONE = 0, SC_RANGE, SC_DELTA } sc_kind_t;

typedef struct {
    uint8_t kind;  // sc_kind_t
    float a, b;    // SC_RANGE: [a, b]; SC_DELTA: +a on top of the automatic target
} sc_metric_t;

typedef struct {
    const char *name;
    const char *label;
    const char *icon;
    const char *default_room;  // room it makes most sense in ("" = any)
    uint32_t duration_s;
    int8_t presence;           // -1 unchanged, 0 empty, 1 occupied
    int8_t occupants;          // 0 = unchanged
    int8_t lights;             // -1 unchanged, 0 off, 1 on
    sc_metric_t m[METRIC_COUNT];
} scenario_t;

extern const scenario_t SCENARIOS[];
extern const int SCENARIO_COUNT;
const scenario_t *scenario_find(const char *name);

typedef struct {
    bool active;
    float min, max;
} range_override_t;

typedef struct {
    const room_profile_t *profile;
    uint32_t rng;

    float v[METRIC_COUNT];  // true environmental state (no measurement noise)
    float drift[METRIC_COUNT];
    float cloud;            // daylight variation 0.5..1

    int occupants;          // automatic occupancy
    float occ_timer_s;
    int prev_effective_occ;
    bool near_object;       // someone close to the unit (drives distance)
    float near_timer_s;
    float near_mm;

    // Automatic event (e.g. cooking at dinner time)
    const scenario_t *auto_event;
    float auto_event_left_s;

    // Manual scenario (control panel / MQTT)
    const scenario_t *scenario;
    float scenario_left_s;
    bool scenario_forever;

    // Manual range overrides
    range_override_t ovr[METRIC_COUNT];
    int8_t presence_ovr;    // -1 none
    float ovr_left_s;
    bool ovr_forever;

    float wander[METRIC_COUNT];      // current target inside a forced range
    float wander_timer_s[METRIC_COUNT];
} sim_state_t;

typedef struct {
    float v[METRIC_COUNT];
    bool presence;
} sim_reading_t;

typedef struct {
    range_override_t m[METRIC_COUNT];
    int8_t presence;        // -1 unchanged
} sim_override_req_t;

void sim_init(sim_state_t *s, const room_profile_t *profile, uint32_t seed);
void sim_step(sim_state_t *s, float dt_s, float hour);
void sim_read(sim_state_t *s, sim_reading_t *out);

// duration_s: <0 = scenario default, 0 = until cleared
void sim_start_scenario(sim_state_t *s, const scenario_t *sc, int32_t duration_s);
// Merges into existing overrides. duration_s 0 = until cleared.
void sim_apply_override(sim_state_t *s, const sim_override_req_t *req, uint32_t duration_s,
                        bool instant);
void sim_clear(sim_state_t *s);   // back to automatic behaviour
void sim_reset(sim_state_t *s);   // clear + return to baseline values

bool sim_presence(const sim_state_t *s);
int sim_occupants(const sim_state_t *s);     // after scenarios/overrides
const char *sim_mode(const sim_state_t *s);  // "auto" | "scenario" | "override"
