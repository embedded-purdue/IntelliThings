#include "time_sync.h"

#include <stdlib.h>
#include <string.h>
#include <sys/time.h>
#include <time.h>

#include "esp_log.h"
#include "esp_netif_sntp.h"
#include "esp_timer.h"

static const char *TAG = "time";
static bool s_started;

static void on_sync(struct timeval *tv)
{
    char ts[40];
    if (time_iso8601(ts, sizeof(ts))) {
        ESP_LOGI(TAG, "Clock synced: %s", ts);
    }
}

void time_sync_init(void)
{
    setenv("TZ", CONFIG_EMU_TIMEZONE, 1);
    tzset();
}

void time_sync_start(void)
{
    if (s_started) {
        return;
    }
    s_started = true;
    esp_sntp_config_t config = ESP_NETIF_SNTP_DEFAULT_CONFIG(CONFIG_EMU_NTP_SERVER);
    config.sync_cb = on_sync;
    if (esp_netif_sntp_init(&config) != ESP_OK) {
        ESP_LOGE(TAG, "SNTP init failed; using the fallback clock");
        return;
    }
    ESP_LOGI(TAG, "SNTP started (%s)", CONFIG_EMU_NTP_SERVER);
}

bool time_is_synced(void)
{
    time_t now = time(NULL);
    struct tm tm;
    localtime_r(&now, &tm);
    return tm.tm_year + 1900 >= 2025;
}

float time_local_hour(void)
{
    if (time_is_synced()) {
        time_t now = time(NULL);
        struct tm tm;
        localtime_r(&now, &tm);
        return tm.tm_hour + tm.tm_min / 60.0f + tm.tm_sec / 3600.0f;
    }
    float h = CONFIG_EMU_FALLBACK_START_HOUR + (float)(esp_timer_get_time() / 1000000LL) / 3600.0f;
    while (h >= 24.0f) {
        h -= 24.0f;
    }
    return h;
}

bool time_iso8601(char *buf, size_t len)
{
    if (!time_is_synced() || len < 26) {
        return false;
    }
    time_t now = time(NULL);
    struct tm tm;
    localtime_r(&now, &tm);
    char tmp[40];
    size_t n = strftime(tmp, sizeof(tmp), "%Y-%m-%dT%H:%M:%S%z", &tm);  // ...-0400
    if (n < 5 || n + 2 > len) {
        return false;
    }
    // Insert the colon: -0400 -> -04:00
    memcpy(buf, tmp, n - 2);
    buf[n - 2] = ':';
    memcpy(buf + n - 1, tmp + n - 2, 2);
    buf[n + 1] = '\0';
    return true;
}
