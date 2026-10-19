#include "app_config.h"

#include <ctype.h>
#include <stdio.h>
#include <string.h>

#include "esp_log.h"
#include "freertos/FreeRTOS.h"
#include "freertos/semphr.h"
#include "nvs.h"

static const char *TAG = "config";

#define NVS_NS "emu_cfg"
#define PUBLISH_MS_MIN 1000
#define PUBLISH_MS_MAX 60000
#define KEEPALIVE_MIN 5
#define KEEPALIVE_MAX 120

static app_config_t s_cfg;
static bool s_saved;
static volatile uint32_t s_publish_ms;
static SemaphoreHandle_t s_lock;

void app_config_defaults(app_config_t *out)
{
    memset(out, 0, sizeof(*out));
    strlcpy(out->host, CONFIG_EMU_MQTT_HOST, sizeof(out->host));
    out->port = CONFIG_EMU_MQTT_PORT;
    strlcpy(out->username, CONFIG_EMU_MQTT_USERNAME, sizeof(out->username));
    strlcpy(out->password, CONFIG_EMU_MQTT_PASSWORD, sizeof(out->password));
    out->keepalive_s = CONFIG_EMU_MQTT_KEEPALIVE_S;
    out->publish_interval_ms = CONFIG_EMU_PUBLISH_INTERVAL_MS;
}

static void load_str(nvs_handle_t h, const char *key, char *dst, size_t len, bool *found)
{
    size_t n = len;
    if (nvs_get_str(h, key, dst, &n) == ESP_OK) {
        *found = true;
    }
}

void app_config_init(void)
{
    s_lock = xSemaphoreCreateMutex();
    configASSERT(s_lock);
    app_config_defaults(&s_cfg);

    nvs_handle_t h;
    if (nvs_open(NVS_NS, NVS_READONLY, &h) == ESP_OK) {
        app_config_t c = s_cfg;
        bool found = false;
        uint16_t u16;
        uint32_t u32;
        load_str(h, "host", c.host, sizeof(c.host), &found);
        load_str(h, "user", c.username, sizeof(c.username), &found);
        load_str(h, "pass", c.password, sizeof(c.password), &found);
        if (nvs_get_u16(h, "port", &u16) == ESP_OK) {
            c.port = u16;
            found = true;
        }
        if (nvs_get_u16(h, "keepalive", &u16) == ESP_OK) {
            c.keepalive_s = u16;
            found = true;
        }
        if (nvs_get_u32(h, "pub_ms", &u32) == ESP_OK) {
            c.publish_interval_ms = u32;
            found = true;
        }
        nvs_close(h);

        char msg[96];
        if (found && app_config_validate(&c, msg, sizeof(msg))) {
            s_cfg = c;
            s_saved = true;
        } else if (found) {
            ESP_LOGW(TAG, "Saved settings ignored (%s); using defaults", msg);
        }
    }
    s_publish_ms = s_cfg.publish_interval_ms;
    ESP_LOGI(TAG, "Broker mqtt://%s:%u, publish every %lu ms (%s)", s_cfg.host,
             (unsigned)s_cfg.port, (unsigned long)s_cfg.publish_interval_ms,
             s_saved ? "saved settings" : "defaults");
}

void app_config_get(app_config_t *out)
{
    xSemaphoreTake(s_lock, portMAX_DELAY);
    *out = s_cfg;
    xSemaphoreGive(s_lock);
}

bool app_config_saved(void)
{
    return s_saved;
}

uint32_t app_config_publish_interval_ms(void)
{
    return s_publish_ms;
}

static bool host_ok(const char *h)
{
    if (!h[0]) {
        return false;
    }
    for (const char *p = h; *p; p++) {
        if (!isalnum((unsigned char)*p) && *p != '.' && *p != '-' && *p != '_') {
            return false;
        }
    }
    return true;
}

static bool printable(const char *s)
{
    for (const char *p = s; *p; p++) {
        if ((unsigned char)*p < 0x20 || *p == 0x7f) {
            return false;
        }
    }
    return true;
}

bool app_config_validate(const app_config_t *c, char *msg, size_t len)
{
    if (!host_ok(c->host)) {
        snprintf(msg, len, "host: 1..%d characters, letters, digits, '.', '-' or '_'",
                 CFG_HOST_MAX);
        return false;
    }
    if (c->port == 0) {
        snprintf(msg, len, "port must be 1..65535");
        return false;
    }
    if (!printable(c->username) || !printable(c->password)) {
        snprintf(msg, len, "username and password can't contain control characters");
        return false;
    }
    if (c->keepalive_s < KEEPALIVE_MIN || c->keepalive_s > KEEPALIVE_MAX) {
        snprintf(msg, len, "keepalive_s must be %d..%d", KEEPALIVE_MIN, KEEPALIVE_MAX);
        return false;
    }
    if (c->publish_interval_ms < PUBLISH_MS_MIN || c->publish_interval_ms > PUBLISH_MS_MAX) {
        snprintf(msg, len, "publish_interval_ms must be %d..%d", PUBLISH_MS_MIN, PUBLISH_MS_MAX);
        return false;
    }
    return true;
}

bool app_config_save(const app_config_t *c, char *msg, size_t len)
{
    if (!app_config_validate(c, msg, len)) {
        return false;
    }
    nvs_handle_t h;
    esp_err_t err = nvs_open(NVS_NS, NVS_READWRITE, &h);
    if (err == ESP_OK) {
        if ((err = nvs_set_str(h, "host", c->host)) == ESP_OK &&
            (err = nvs_set_u16(h, "port", c->port)) == ESP_OK &&
            (err = nvs_set_str(h, "user", c->username)) == ESP_OK &&
            (err = nvs_set_str(h, "pass", c->password)) == ESP_OK &&
            (err = nvs_set_u16(h, "keepalive", c->keepalive_s)) == ESP_OK &&
            (err = nvs_set_u32(h, "pub_ms", c->publish_interval_ms)) == ESP_OK) {
            err = nvs_commit(h);
        }
        nvs_close(h);
    }
    if (err != ESP_OK) {
        snprintf(msg, len, "saving failed (%s)", esp_err_to_name(err));
        return false;
    }
    xSemaphoreTake(s_lock, portMAX_DELAY);
    s_cfg = *c;
    s_saved = true;
    s_publish_ms = c->publish_interval_ms;
    xSemaphoreGive(s_lock);
    snprintf(msg, len, "saved");
    return true;
}

bool app_config_reset(char *msg, size_t len)
{
    nvs_handle_t h;
    esp_err_t err = nvs_open(NVS_NS, NVS_READWRITE, &h);
    if (err == ESP_OK) {
        err = nvs_erase_all(h);
        if (err == ESP_OK) {
            err = nvs_commit(h);
        }
        nvs_close(h);
    }
    if (err != ESP_OK && err != ESP_ERR_NVS_NOT_FOUND) {
        snprintf(msg, len, "reset failed (%s)", esp_err_to_name(err));
        return false;
    }
    app_config_t d;
    app_config_defaults(&d);
    xSemaphoreTake(s_lock, portMAX_DELAY);
    s_cfg = d;
    s_saved = false;
    s_publish_ms = d.publish_interval_ms;
    xSemaphoreGive(s_lock);
    snprintf(msg, len, "defaults restored");
    return true;
}

bool app_config_mqtt_differs(const app_config_t *a, const app_config_t *b)
{
    return strcmp(a->host, b->host) != 0 || a->port != b->port ||
           strcmp(a->username, b->username) != 0 || strcmp(a->password, b->password) != 0 ||
           a->keepalive_s != b->keepalive_s;
}
