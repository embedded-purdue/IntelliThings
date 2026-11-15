#include "settings.h"

#include <ctype.h>
#include <stdio.h>
#include <string.h>

#include "esp_log.h"
#include "freertos/FreeRTOS.h"
#include "freertos/semphr.h"
#include "nvs.h"

static const char *TAG = "settings";

#define NVS_NS "dm"
#define BLOB_VERSION 2

typedef struct {
    uint16_t version;
    uint16_t size;
    dm_settings_t s;
} blob_t;

static dm_settings_t s_cur;
static uint32_t s_version = 1;
static uint32_t s_presses;
static uint32_t s_boots;
static SemaphoreHandle_t s_lock;

void settings_defaults(dm_settings_t *out)
{
    memset(out, 0, sizeof(*out));
    strlcpy(out->host, CONFIG_DM_MQTT_HOST, sizeof(out->host));
    out->port = CONFIG_DM_MQTT_PORT;
    strlcpy(out->username, CONFIG_DM_MQTT_USERNAME, sizeof(out->username));
    strlcpy(out->password, CONFIG_DM_MQTT_PASSWORD, sizeof(out->password));
    out->keepalive_s = CONFIG_DM_MQTT_KEEPALIVE_S;
    strlcpy(out->prefix, CONFIG_DM_TOPIC_PREFIX, sizeof(out->prefix));
    out->theme = 0;
    out->animation = true;
    out->fahrenheit = true;
    strlcpy(out->pin, CONFIG_DM_ADMIN_PIN, sizeof(out->pin));
    out->publish = true;
    out->loop = true;
    out->interval_s = 90;
    out->source = SRC_PRESETS;
    out->mix_pct = 30;
    out->roll_ms = 4000;
    out->hold_s = 30;
    out->spread = SPREAD_NORMAL;
    out->abnormal_pct = 15;
}

void settings_init(void)
{
    s_lock = xSemaphoreCreateMutex();
    configASSERT(s_lock);
    settings_defaults(&s_cur);

    nvs_handle_t h;
    if (nvs_open(NVS_NS, NVS_READWRITE, &h) != ESP_OK) {
        ESP_LOGE(TAG, "NVS unavailable; using the defaults");
        return;
    }
    nvs_get_u32(h, "boots", &s_boots);
    s_boots++;
    nvs_set_u32(h, "boots", s_boots);
    nvs_commit(h);
    blob_t b;
    size_t len = sizeof(b);
    if (nvs_get_blob(h, "cfg", &b, &len) == ESP_OK) {
        char msg[96];
        if (len != sizeof(b) || b.version != BLOB_VERSION || b.size != sizeof(dm_settings_t)) {
            ESP_LOGW(TAG, "Saved settings are from another firmware layout; using the defaults");
        } else if (!settings_validate(&b.s, msg, sizeof(msg))) {
            ESP_LOGW(TAG, "Saved settings ignored (%s); using the defaults", msg);
        } else {
            s_cur = b.s;
            ESP_LOGI(TAG, "Loaded saved settings");
        }
    }
    nvs_get_u32(h, "presses", &s_presses);
    nvs_close(h);
}

void settings_get(dm_settings_t *out)
{
    xSemaphoreTake(s_lock, portMAX_DELAY);
    *out = s_cur;
    xSemaphoreGive(s_lock);
}

uint32_t settings_version(void)
{
    return s_version;
}

static bool host_ok(const char *s)
{
    if (!*s) {
        return false;
    }
    for (; *s; s++) {
        if (!isalnum((unsigned char)*s) && *s != '.' && *s != '-') {
            return false;
        }
    }
    return true;
}

static bool printable(const char *s, bool spaces)
{
    for (; *s; s++) {
        unsigned char c = (unsigned char)*s;
        if (c < 0x20 || c > 0x7E || (!spaces && c == ' ')) {
            return false;
        }
    }
    return true;
}

static bool prefix_ok(const char *s)
{
    size_t n = strlen(s);
    if (n == 0 || s[0] == '/' || s[n - 1] == '/' || strstr(s, "//") || !printable(s, false)) {
        return false;
    }
    return !strchr(s, '+') && !strchr(s, '#');
}

bool settings_validate(const dm_settings_t *s, char *msg, size_t len)
{
    if (!host_ok(s->host)) {
        snprintf(msg, len, "broker host: letters, digits, dots and dashes only");
        return false;
    }
    if (s->port == 0) {
        snprintf(msg, len, "broker port must be 1..65535");
        return false;
    }
    if (s->keepalive_s < 5 || s->keepalive_s > 120) {
        snprintf(msg, len, "keepalive must be 5..120 s");
        return false;
    }
    if (!printable(s->username, false) || !printable(s->password, true)) {
        snprintf(msg, len, "broker login: printable ASCII only");
        return false;
    }
    if (!prefix_ok(s->prefix)) {
        snprintf(msg, len, "topic prefix: no spaces, + or #, no leading, trailing or double /");
        return false;
    }
    if (!s->pin[0] || !printable(s->pin, false)) {
        snprintf(msg, len, "PIN: 1..%d characters, no spaces", SET_PIN_MAX);
        return false;
    }
    if (s->theme > 1 || s->source > SRC_MIX || s->spread > SPREAD_UNIFORM) {
        snprintf(msg, len, "unknown theme, loop source or spread");
        return false;
    }
    if (s->interval_s < 10 || s->interval_s > 3600) {
        snprintf(msg, len, "scenario interval must be 10..3600 s");
        return false;
    }
    if (s->roll_ms < 1000 || s->roll_ms > 15000) {
        snprintf(msg, len, "roll length must be 1..15 s");
        return false;
    }
    if (s->hold_s > 3600) {
        snprintf(msg, len, "hold after a roll must be 0..3600 s");
        return false;
    }
    if (s->mix_pct > 100 || s->abnormal_pct > 100) {
        snprintf(msg, len, "percentages must be 0..100");
        return false;
    }
    return true;
}

bool settings_save(const dm_settings_t *s, char *msg, size_t len)
{
    if (!settings_validate(s, msg, len)) {
        return false;
    }
    blob_t b = {.version = BLOB_VERSION, .size = sizeof(dm_settings_t), .s = *s};
    nvs_handle_t h;
    esp_err_t err = nvs_open(NVS_NS, NVS_READWRITE, &h);
    if (err == ESP_OK) {
        err = nvs_set_blob(h, "cfg", &b, sizeof(b));
        if (err == ESP_OK) {
            err = nvs_commit(h);
        }
        nvs_close(h);
    }
    if (err != ESP_OK) {
        snprintf(msg, len, "couldn't save the settings (%s)", esp_err_to_name(err));
        return false;
    }
    xSemaphoreTake(s_lock, portMAX_DELAY);
    s_cur = *s;
    s_version++;
    xSemaphoreGive(s_lock);
    snprintf(msg, len, "settings saved");
    return true;
}

bool settings_reset(char *msg, size_t len)
{
    nvs_handle_t h;
    if (nvs_open(NVS_NS, NVS_READWRITE, &h) == ESP_OK) {
        nvs_erase_key(h, "cfg");
        nvs_commit(h);
        nvs_close(h);
    }
    xSemaphoreTake(s_lock, portMAX_DELAY);
    settings_defaults(&s_cur);
    s_version++;
    xSemaphoreGive(s_lock);
    snprintf(msg, len, "settings back to the defaults");
    return true;
}

bool settings_check_pin(const char *pin)
{
    xSemaphoreTake(s_lock, portMAX_DELAY);
    bool ok = pin && strcmp(pin, s_cur.pin) == 0;
    xSemaphoreGive(s_lock);
    return ok;
}

static void save_presses(uint32_t n)
{
    nvs_handle_t h;
    if (nvs_open(NVS_NS, NVS_READWRITE, &h) == ESP_OK) {
        nvs_set_u32(h, "presses", n);
        nvs_commit(h);
        nvs_close(h);
    }
}

uint32_t settings_boot_count(void)
{
    return s_boots;
}

uint32_t settings_presses(void)
{
    return s_presses;
}

void settings_count_press(void)
{
    xSemaphoreTake(s_lock, portMAX_DELAY);
    uint32_t n = ++s_presses;
    xSemaphoreGive(s_lock);
    save_presses(n);
}

void settings_reset_presses(void)
{
    xSemaphoreTake(s_lock, portMAX_DELAY);
    s_presses = 0;
    xSemaphoreGive(s_lock);
    save_presses(0);
}
