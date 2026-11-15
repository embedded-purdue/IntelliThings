// Saved settings: the MQTT broker and topics, the look of the demo page, the admin PIN and the
// demo timing. Kconfig / sdkconfig.local hold the broker defaults; the admin page changes the
// rest at runtime. Everything is saved in NVS (namespace "dm") and survives a reboot.
#pragma once

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#include "demo_home.h"

#define SET_HOST_MAX 63
#define SET_USER_MAX 32
#define SET_PASS_MAX 64
#define SET_PREFIX_MAX 64
#define SET_PIN_MAX 8

typedef enum { SRC_PRESETS, SRC_RANDOM, SRC_MIX } loop_source_t;

typedef struct {
    // MQTT
    char host[SET_HOST_MAX + 1];
    uint16_t port;
    char username[SET_USER_MAX + 1];  // "" = anonymous
    char password[SET_PASS_MAX + 1];  // never sent back over HTTP
    uint16_t keepalive_s;
    char prefix[SET_PREFIX_MAX + 1];  // "intellithings/demo"
    // Demo page
    uint8_t theme;  // 0 dark, 1 light
    bool animation;
    bool fahrenheit;
    // Admin
    char pin[SET_PIN_MAX + 1];
    // Demo timing and randomizer (the state at boot)
    bool publish, loop;
    uint16_t interval_s;
    uint8_t source;  // loop_source_t
    uint8_t mix_pct;
    uint16_t roll_ms;
    uint16_t hold_s;
    uint8_t spread;  // demo_spread_t
    uint8_t abnormal_pct;
} dm_settings_t;

void settings_init(void);  // after nvs_flash_init() and demo_home_init()
void settings_get(dm_settings_t *out);
void settings_defaults(dm_settings_t *out);
uint32_t settings_version(void);  // bumps on every save

bool settings_validate(const dm_settings_t *s, char *msg, size_t len);
// Validates, writes to NVS and makes it current. Applying it is the caller's job.
bool settings_save(const dm_settings_t *s, char *msg, size_t len);
bool settings_reset(char *msg, size_t len);  // back to the defaults

bool settings_check_pin(const char *pin);

// Boots since the NVS was erased; makes the doorbell's event_id unique across reboots.
uint32_t settings_boot_count(void);

// Buzzer presses, all sources, kept across reboots.
uint32_t settings_presses(void);
void settings_count_press(void);
void settings_reset_presses(void);
