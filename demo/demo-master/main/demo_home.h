// Demo home: the whole-house entities and the rules that fill and randomize them. Plain C (no
// ESP-IDF calls), adapted from the emulator's demo mode (ai/emulator/esp32_c3_emulator/main/
// demo_home.c). The entity tables come from demo/demo_home.json via tools/gen_demo.py; the
// limits (caps and normal ranges) start from the JSON and can be changed at runtime. Each area
// is one MQTT topic (ai/docs/demo-device-inventory.md).
#pragma once

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#define DEMO_MAX_ENTITIES 160  // also MAX_ENTITIES in tools/gen_demo.py
#define DEMO_MAX_SETS 20      // values one scenario can set (MAX_SETS)
#define DEMO_NAME_MAX 47      // scenario name, UTF-8 bytes (MAX_NAME)
#define DEMO_EXPECT_MAX 119   // expected AI response, UTF-8 bytes (MAX_EXPECT)
#define DEMO_MAX_NODES 4      // areas with a sensor node
#define SCEN_DOORBELL 0x01    // scenario flag: ring the doorbell when it starts

typedef enum { DEMO_NUM, DEMO_BOOL, DEMO_ENUM } demo_type_t;
// Device and household states, sensor readings, or internal context (not published).
typedef enum { KIND_DEVICE, KIND_SENSOR, KIND_CONTEXT } demo_kind_t;
// How a value is chosen when a scenario doesn't set it (fill_rules in the JSON).
typedef enum {
    FILL_NORMAL, FILL_PRESENCE, FILL_LIGHT, FILL_LOCK, FILL_CAR, FILL_OFF, FILL_BLINDS
} demo_fill_t;
// Values computed from others in the same area (derive_rules in the JSON).
typedef enum { DERIVE_NONE, DERIVE_PM1, DERIVE_PM10, DERIVE_LUX, DERIVE_DISTANCE } demo_derive_t;

typedef struct {
    const char *key;        // "kitchen"
    const char *label;
    const char *node_room;  // room of the sensor node shown here ("" = no node)
    bool sleep_area;
} demo_area_t;

typedef struct {
    const char *key;    // "kitchen.temperature_c"
    const char *field;  // "temperature_c" (the name in MQTT payloads)
    const char *label;
    const char *unit;
    uint8_t area;
    uint8_t type;    // demo_type_t
    uint8_t kind;    // demo_kind_t
    bool internal;   // drives the rules and shows on the demo page, but isn't published
    uint8_t dec;     // decimals (numbers)
    uint8_t fill;    // demo_fill_t
    uint8_t derive;  // demo_derive_t
    float min, max;    // built-in caps; for bool/enum: 0 .. options - 1
    float nmin, nmax;  // numbers: normal range; bool: nmin = chance of true
    const char *const *options;  // enum options, or bool labels {false, true}
    uint8_t n_options;
    uint16_t normal_mask;  // enum: options that count as normal
} demo_entity_t;

typedef struct __attribute__((packed)) {
    uint8_t entity;
    float value;  // number, 0/1, or enum option index
} demo_set_t;

// A built-in scenario from the JSON (the factory library).
typedef struct {
    const char *name;
    uint8_t group;  // index into DEMO_GROUPS
    uint8_t flags;  // SCEN_*
    const char *expect;  // the AI response a tester should look for ("" = none)
    const demo_set_t *sets;
    uint8_t n_sets;
} demo_builtin_t;

extern const demo_area_t DEMO_AREAS[];
extern const int DEMO_AREA_COUNT;
extern const demo_entity_t DEMO_ENTITIES[];
extern const int DEMO_ENTITY_COUNT;
extern const demo_builtin_t DEMO_BUILTIN[];
extern const int DEMO_BUILTIN_COUNT;
extern const char *const DEMO_GROUPS[];
extern const char *const DEMO_GROUP_LABELS[];
extern const int DEMO_GROUP_COUNT;

// A scenario as the library keeps it (editable on the board).
typedef struct {
    char name[DEMO_NAME_MAX + 1];
    uint8_t group;
    uint8_t flags;  // SCEN_*
    uint8_t n_sets;
    char expect[DEMO_EXPECT_MAX + 1];  // shown on the admin page only, never published
    demo_set_t sets[DEMO_MAX_SETS];
} demo_scenario_t;

// Runtime limits of one entity. Numbers: caps [min, max] and normal range [nmin, nmax].
// Bools: nmin = chance of true in a normal home. Enums: not editable.
typedef struct {
    float min, max, nmin, nmax;
} demo_limit_t;

typedef enum { SPREAD_NORMAL, SPREAD_UNIFORM } demo_spread_t;

typedef struct {
    float v[DEMO_MAX_ENTITIES];
} demo_values_t;

void demo_home_init(void);  // resolves the indexes the fill and derive rules use
int demo_entity_find(const char *key);  // -1 if unknown
uint32_t demo_schema_hash(void);  // changes when the entity table changes (library storage)

// Areas with a sensor node, in table order: node n is area demo_node_area(n).
int demo_node_count(void);
int demo_node_area(int n);

const demo_limit_t *demo_limit(int e);
// Checks and sets one entity's limits; false with the reason in msg.
bool demo_limit_set(int e, const demo_limit_t *l, char *msg, size_t len);
void demo_limits_reset(void);  // back to the JSON's limits
bool demo_limit_is_default(int e);

void demo_scenario_from_builtin(demo_scenario_t *out, int b);
// Checks a value for entity e (type, option, within the current caps); false with a reason.
bool demo_value_check(int e, float v, char *msg, size_t len);

// Every entity for scenario s: its own values, the rest filled by the fill rules, then the
// derived values. Always the same result for the same scenario.
void demo_fill_scenario(demo_values_t *out, const demo_scenario_t *s);
// A random home within the caps. SPREAD_NORMAL: each entity follows the fill rules (normal
// values) except with abnormal_pct % chance, when it takes any value within its caps.
// SPREAD_UNIFORM: every entity anywhere within its caps. Derived values follow.
void demo_roll(demo_values_t *out, uint32_t seed, demo_spread_t spread, uint8_t abnormal_pct);

// False for a number outside its normal range or an enum option that isn't normal.
bool demo_is_normal(int e, float v);
// The value as a JSON literal: number, true/false, or "option". Returns the length.
int demo_json_value(int e, float v, char *buf, size_t len);
// The area's payload for <prefix>/<area key>: every published entity of the area in table
// order, then "ts" (omitted when NULL). Returns the length, or -1 if buf is too small.
int demo_area_json(const demo_values_t *h, int area, const char *ts, char *buf, size_t len);
