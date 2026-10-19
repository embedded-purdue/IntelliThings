// Runtime settings: the MQTT broker and the sensor publish interval. Kconfig / sdkconfig.local
// values are the defaults; the panel's Settings (POST /api/config) override them, saved in NVS
// (namespace "emu_cfg"), without a rebuild.
#pragma once

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#define CFG_HOST_MAX 63
#define CFG_USER_MAX 32
#define CFG_PASS_MAX 64

typedef struct {
    char host[CFG_HOST_MAX + 1];
    uint16_t port;
    char username[CFG_USER_MAX + 1];  // "" = anonymous
    char password[CFG_PASS_MAX + 1];  // never sent back over HTTP
    uint16_t keepalive_s;
    uint32_t publish_interval_ms;
} app_config_t;

void app_config_init(void);  // after nvs_flash_init()
void app_config_get(app_config_t *out);
void app_config_defaults(app_config_t *out);
bool app_config_saved(void);  // true if NVS overrides any default
uint32_t app_config_publish_interval_ms(void);

// Checks ranges and characters. False with a reason in msg.
bool app_config_validate(const app_config_t *cfg, char *msg, size_t len);
// Validates, writes to NVS and makes it current. Applying it to MQTT is the caller's job.
bool app_config_save(const app_config_t *cfg, char *msg, size_t len);
// Erases the NVS overrides; the defaults become current.
bool app_config_reset(char *msg, size_t len);

// True if the broker connection settings differ (host, port, login, keepalive).
bool app_config_mqtt_differs(const app_config_t *a, const app_config_t *b);
