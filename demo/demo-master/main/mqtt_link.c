#include "mqtt_link.h"

#include <stdio.h>
#include <string.h>

#include "ai_watch.h"
#include "esp_log.h"
#include "event_log.h"
#include "freertos/FreeRTOS.h"
#include "freertos/semphr.h"
#include "mqtt_client.h"
#include "settings.h"

static const char *TAG = "mqtt";

#define TOPIC_MAX 128

static esp_mqtt_client_handle_t s_client;
static SemaphoreHandle_t s_lock;  // guards s_client, s_prefix and the strings below
static mqtt_link_cb_t s_on_connected;
static volatile bool s_connected;
static bool s_ever_connected;
static uint32_t s_sent, s_reconnects;
static char s_prefix[SET_PREFIX_MAX + 1];
static char s_uri[96];
static char s_lwt_topic[TOPIC_MAX];
static char s_last_error[64];

static void lock(void)
{
    xSemaphoreTake(s_lock, portMAX_DELAY);
}

static void unlock(void)
{
    xSemaphoreGive(s_lock);
}

static void on_event(void *arg, esp_event_base_t base, int32_t id, void *data)
{
    esp_mqtt_event_handle_t ev = data;
    switch ((esp_mqtt_event_id_t)id) {
    case MQTT_EVENT_CONNECTED: {
        s_connected = true;
        if (s_ever_connected) {
            s_reconnects++;
        }
        s_ever_connected = true;
        s_last_error[0] = '\0';
        // No s_lock in here: mqtt_link_publish() holds it while waiting for the client, and the
        // client is busy running this handler. s_lwt_topic only changes with a new client.
        esp_mqtt_client_publish(ev->client, s_lwt_topic, "online", 0, 1, 1);
        ai_watch_subscribe(ev->client);
        event_log("MQTT connected to %s, status online", s_uri);
        if (s_on_connected) {
            s_on_connected();
        }
        break;
    }
    case MQTT_EVENT_DATA:
        ai_watch_on_data(ev);
        break;
    case MQTT_EVENT_DISCONNECTED:
        if (s_connected) {
            event_log("MQTT disconnected");
        }
        s_connected = false;
        break;
    case MQTT_EVENT_ERROR:
        if (ev->error_handle) {
            const esp_mqtt_error_codes_t *e = ev->error_handle;
            if (e->error_type == MQTT_ERROR_TYPE_CONNECTION_REFUSED) {
                snprintf(s_last_error, sizeof(s_last_error), "broker refused the login (code %d)",
                         e->connect_return_code);
            } else if (e->esp_tls_last_esp_err || e->esp_transport_sock_errno) {
                snprintf(s_last_error, sizeof(s_last_error), "can't reach the broker (errno %d)",
                         e->esp_transport_sock_errno);
            }
        }
        break;
    default:
        break;
    }
}

static void create_client(void)
{
    dm_settings_t s;
    settings_get(&s);
    lock();
    strlcpy(s_prefix, s.prefix, sizeof(s_prefix));
    snprintf(s_uri, sizeof(s_uri), "mqtt://%s:%u", s.host, (unsigned)s.port);
    snprintf(s_lwt_topic, sizeof(s_lwt_topic), "%s/status", s.prefix);
    esp_mqtt_client_config_t cfg = {
        .broker.address.uri = s_uri,
        .credentials.client_id = CONFIG_DM_HOSTNAME,
        .credentials.username = s.username[0] ? s.username : NULL,
        .credentials.authentication.password = s.username[0] ? s.password : NULL,
        .session.keepalive = s.keepalive_s,
        .session.last_will = {.topic = s_lwt_topic, .msg = "offline", .qos = 1, .retain = 1},
        .network.reconnect_timeout_ms = 5000,
        .network.timeout_ms = 5000,
        .buffer.size = 1024,
        .buffer.out_size = 4096,
    };
    s_client = esp_mqtt_client_init(&cfg);
    unlock();
    if (!s_client) {
        ESP_LOGE(TAG, "Client init failed");
        return;
    }
    esp_mqtt_client_register_event(s_client, ESP_EVENT_ANY_ID, on_event, NULL);
    esp_mqtt_client_start(s_client);
    ESP_LOGI(TAG, "Connecting to %s, topics under %s/", s_uri, s_prefix);
}

void mqtt_link_init(mqtt_link_cb_t on_connected)
{
    s_lock = xSemaphoreCreateMutex();
    configASSERT(s_lock);
    s_on_connected = on_connected;
}

void mqtt_link_start(void)
{
    if (!s_client) {
        create_client();
    }
}

void mqtt_link_restart(void)
{
    lock();
    esp_mqtt_client_handle_t old = s_client;
    s_client = NULL;
    unlock();
    if (old) {
        if (s_connected) {
            // A clean goodbye: the broker doesn't send the Last Will on a normal disconnect.
            esp_mqtt_client_publish(old, s_lwt_topic, "offline", 0, 1, 1);
            vTaskDelay(pdMS_TO_TICKS(200));
        }
        s_connected = false;
        esp_mqtt_client_stop(old);
        esp_mqtt_client_destroy(old);
    }
    event_log("MQTT settings applied, reconnecting");
    create_client();
}

bool mqtt_link_connected(void)
{
    return s_connected;
}

void mqtt_link_topic(char *buf, size_t len, const char *leaf)
{
    lock();
    snprintf(buf, len, "%s/%s", s_prefix, leaf);
    unlock();
}

bool mqtt_link_publish(const char *leaf, const char *data, int len, bool retain)
{
    char topic[TOPIC_MAX];
    mqtt_link_topic(topic, sizeof(topic), leaf);
    lock();
    int id = (s_client && s_connected) ? esp_mqtt_client_publish(s_client, topic, data, len, 1, retain)
                                       : -1;
    unlock();
    if (id < 0) {
        return false;
    }
    s_sent++;
    return true;
}

uint32_t mqtt_link_sent(void)
{
    return s_sent;
}

uint32_t mqtt_link_reconnects(void)
{
    return s_reconnects;
}

void mqtt_link_uri(char *buf, size_t len)
{
    lock();
    strlcpy(buf, s_uri, len);
    unlock();
}

void mqtt_link_last_error(char *buf, size_t len)
{
    strlcpy(buf, s_connected ? "" : s_last_error, len);
}
