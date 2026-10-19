// Demo home: the whole-house entities, presets and randomizer behind demo mode. Plain C (no
// ESP-IDF calls) so it can be unit-tested on a host and reused by the real demo master
// (demo/demo-master/). The tables come from demo/demo_home.json via tools/gen_demo.py.
#pragma once

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#include "sim_model.h"

#define DEMO_MAX_ENTITIES 80  // also MAX_ENTITIES in tools/gen_demo.py

typedef enum { DEMO_NUM, DEMO_BOOL, DEMO_ENUM } demo_type_t;
// How a value is chosen when a preset doesn't set it (fill_rules in the JSON).
typedef enum { FILL_NORMAL, FILL_PRESENCE, FILL_LIGHT, FILL_LOCK, FILL_CAR, FILL_OFF } demo_fill_t;
// Values computed from others in the same area (derive_rules in the JSON).
typedef enum { DERIVE_NONE, DERIVE_PM1, DERIVE_PM10, DERIVE_LUX, DERIVE_DISTANCE } demo_derive_t;

typedef struct {
    const char *key;        // "kitchen"
    const char *label;
    const char *node_room;  // sim room name of the node shown here ("" = no node)
    bool sleep_area;
} demo_area_t;

typedef struct {
    const char *key;    // "kitchen.temperature_c"
    const char *field;  // "temperature_c" (the name in MQTT payloads)
    const char *label;
    const char *unit;
    uint8_t area;
    uint8_t type;    // demo_type_t
    uint8_t dec;     // decimals (numbers)
    uint8_t fill;    // demo_fill_t
    uint8_t derive;  // demo_derive_t
    float min, max;    // caps; for bool/enum: 0 .. options - 1
    float nmin, nmax;  // numbers: normal range; bool: nmin = chance of true
    const char *const *options;  // enum options, or bool labels {false, true}
    uint8_t n_options;
    uint16_t normal_mask;  // enum: options that count as normal
} demo_entity_t;

typedef struct {
    uint8_t entity;
    float value;  // number, 0/1, or enum option index
} demo_set_t;

typedef struct {
    const char *name;
    uint8_t group;  // index into DEMO_GROUPS
    const demo_set_t *sets;
    uint8_t n_sets;
} demo_preset_t;

extern const demo_area_t DEMO_AREAS[];
extern const int DEMO_AREA_COUNT;
extern const demo_entity_t DEMO_ENTITIES[];
extern const int DEMO_ENTITY_COUNT;
extern const demo_preset_t DEMO_PRESETS[];
extern const int DEMO_PRESET_COUNT;
extern const char *const DEMO_GROUPS[];
extern const char *const DEMO_GROUP_LABELS[];
extern const int DEMO_GROUP_COUNT;

typedef enum { SPREAD_NORMAL, SPREAD_UNIFORM } demo_spread_t;

typedef struct {
    float v[DEMO_MAX_ENTITIES];
} demo_values_t;

void demo_home_init(void);  // resolves the indexes the fill and derive rules use
int demo_entity_find(const char *key);  // -1 if unknown
int demo_preset_find(const char *name);  // 0-based, -1 if unknown

// Every entity for preset p (0-based): the preset's own values, the rest filled by the fill
// rules, then the derived values. Always the same result for the same preset.
void demo_fill_preset(demo_values_t *out, int p);
// A random home within the caps. SPREAD_NORMAL: each entity follows the fill rules (normal
// values) except with abnormal_pct % chance, when it takes any value within its caps.
// SPREAD_UNIFORM: every entity anywhere within its caps. Derived values follow.
void demo_roll(demo_values_t *out, uint32_t seed, demo_spread_t spread, uint8_t abnormal_pct);
// Next preset for the loop: random, never `last` (-1 = none).
int demo_pick_preset(uint32_t *rng, int last);

// False for a number outside its normal range or an enum option that isn't normal.
bool demo_is_normal(int e, float v);
// The value as a JSON literal: number, true/false, or "option". Returns the length.
int demo_json_value(int e, float v, char *buf, size_t len);
// The node payload for a node-room area. False if the area has no node.
bool demo_node_reading(const demo_values_t *h, int area, sim_reading_t *out);
