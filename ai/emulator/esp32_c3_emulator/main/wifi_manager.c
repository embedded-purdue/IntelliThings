#include "wifi_manager.h"

#include <stdio.h>
#include <string.h>

#include "esp_event.h"
#include "esp_log.h"
#include "esp_netif.h"
#include "esp_timer.h"
#include "esp_wifi.h"

static const char *TAG = "wifi";

static wifi_ip_cb_t s_on_ip;
static esp_netif_t *s_netif;
static esp_timer_handle_t s_retry_timer;
static volatile bool s_connected;
static bool s_had_ip;
static uint32_t s_backoff_ms = 1000;
static uint32_t s_reconnects;
static char s_ip[16];

static void retry_cb(void *arg)
{
    ESP_LOGI(TAG, "Connecting to \"%s\"...", CONFIG_EMU_WIFI_SSID);
    esp_wifi_connect();
}

static void on_wifi_event(void *arg, esp_event_base_t base, int32_t id, void *data)
{
    if (id == WIFI_EVENT_STA_START) {
        ESP_LOGI(TAG, "Connecting to \"%s\"...", CONFIG_EMU_WIFI_SSID);
        esp_wifi_connect();
    } else if (id == WIFI_EVENT_STA_DISCONNECTED) {
        const wifi_event_sta_disconnected_t *ev = data;
        if (s_connected) {
            s_reconnects++;
        }
        s_connected = false;
        s_ip[0] = '\0';
        ESP_LOGW(TAG, "Disconnected (reason %d); retrying in %lu s", ev->reason,
                 (unsigned long)(s_backoff_ms / 1000));
        if (!esp_timer_is_active(s_retry_timer)) {
            esp_timer_start_once(s_retry_timer, (uint64_t)s_backoff_ms * 1000);
        }
        s_backoff_ms = s_backoff_ms * 2 > 30000 ? 30000 : s_backoff_ms * 2;
    }
}

static void on_ip_event(void *arg, esp_event_base_t base, int32_t id, void *data)
{
    const ip_event_got_ip_t *ev = data;
    snprintf(s_ip, sizeof(s_ip), IPSTR, IP2STR(&ev->ip_info.ip));
    s_connected = true;
    s_backoff_ms = 1000;
    ESP_LOGI(TAG, "Connected. IP %s  gateway " IPSTR "  RSSI %d dBm", s_ip,
             IP2STR(&ev->ip_info.gw), wifi_get_rssi());
    bool first = !s_had_ip;
    s_had_ip = true;
    if (s_on_ip) {
        s_on_ip(first);
    }
}

void wifi_manager_start(wifi_ip_cb_t on_ip)
{
    s_on_ip = on_ip;
    s_netif = esp_netif_create_default_wifi_sta();
    esp_netif_set_hostname(s_netif, CONFIG_EMU_HOSTNAME);

    wifi_init_config_t init = WIFI_INIT_CONFIG_DEFAULT();
    ESP_ERROR_CHECK(esp_wifi_init(&init));

    const esp_timer_create_args_t targs = {.callback = retry_cb, .name = "wifi_retry"};
    ESP_ERROR_CHECK(esp_timer_create(&targs, &s_retry_timer));

    ESP_ERROR_CHECK(esp_event_handler_register(WIFI_EVENT, ESP_EVENT_ANY_ID, on_wifi_event, NULL));
    ESP_ERROR_CHECK(esp_event_handler_register(IP_EVENT, IP_EVENT_STA_GOT_IP, on_ip_event, NULL));

    wifi_config_t cfg = {0};
    strlcpy((char *)cfg.sta.ssid, CONFIG_EMU_WIFI_SSID, sizeof(cfg.sta.ssid));
    strlcpy((char *)cfg.sta.password, CONFIG_EMU_WIFI_PASSWORD, sizeof(cfg.sta.password));
    cfg.sta.threshold.authmode = strlen(CONFIG_EMU_WIFI_PASSWORD) ? WIFI_AUTH_WPA2_PSK : WIFI_AUTH_OPEN;

    ESP_ERROR_CHECK(esp_wifi_set_mode(WIFI_MODE_STA));
    ESP_ERROR_CHECK(esp_wifi_set_config(WIFI_IF_STA, &cfg));
    ESP_ERROR_CHECK(esp_wifi_set_ps(WIFI_PS_NONE));  // snappier web panel and MQTT
    ESP_ERROR_CHECK(esp_wifi_start());
}

bool wifi_is_connected(void)
{
    return s_connected;
}

void wifi_get_ip(char *buf, size_t len)
{
    strlcpy(buf, s_connected ? s_ip : "", len);
}

int wifi_get_rssi(void)
{
    wifi_ap_record_t ap;
    if (esp_wifi_sta_get_ap_info(&ap) == ESP_OK) {
        return ap.rssi;
    }
    return 0;
}

uint32_t wifi_reconnect_count(void)
{
    return s_reconnects;
}
