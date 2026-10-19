#include "mqtt_node.h"

#include <stdio.h>
#include <string.h>

#include "ai_message.h"
#include "emulator_ctrl.h"
#include "esp_log.h"
#include "event_log.h"
#include "freertos/FreeRTOS.h"
#include "freertos/task.h"
#include "mqtt_client.h"
#include "virtual_node.h"

static const char *TAG = "mqtt";

#define URI_MAX 96

static char s_uri[URI_MAX];
static portMUX_TYPE s_uri_mux = portMUX_INITIALIZER_UNLOCKED;
static char s_client_ids[NODE_COUNT][48];
static TaskHandle_t s_reconf_task;
static volatile bool s_started;
static volatile bool s_reconfiguring;

void mqtt_broker_uri(char *buf, size_t len)
{
    portENTER_CRITICAL(&s_uri_mux);
    strlcpy(buf, s_uri, len);
    portEXIT_CRITICAL(&s_uri_mux);
}

static void set_uri(const app_config_t *c)
{
    char uri[URI_MAX];
    snprintf(uri, sizeof(uri), "mqtt://%s:%u", c->host, (unsigned)c->port);
    portENTER_CRITICAL(&s_uri_mux);
    strlcpy(s_uri, uri, sizeof(s_uri));
    portEXIT_CRITICAL(&s_uri_mux);
}

void mqtt_fill_config(esp_mqtt_client_config_t *out, const app_config_t *c, const char *uri)
{
    memset(out, 0, sizeof(*out));
    out->broker.address.uri = uri;
    // esp-mqtt keeps the previous login when these are NULL, so clearing a login needs a
    // reboot (web_server.c handles that).
    out->credentials.username = c->username[0] ? c->username : NULL;
    out->credentials.authentication.password = c->password[0] ? c->password : NULL;
    out->session.keepalive = c->keepalive_s;
    out->network.reconnect_timeout_ms = 5000;
    // Connect/write timeout (default 10 s). Also how long a broker change can wait for a
    // client stuck connecting to an unreachable broker before it stops.
    out->network.timeout_ms = 5000;
    out->buffer.size = 1024;
}

static void node_config(vnode_t *n, int i, const app_config_t *c, const char *uri,
                        esp_mqtt_client_config_t *cfg)
{
    mqtt_fill_config(cfg, c, uri);
    cfg->credentials.client_id = s_client_ids[i];
    cfg->session.last_will.topic = n->topic_status;
    cfg->session.last_will.msg = "offline";
    cfg->session.last_will.qos = 1;
    cfg->session.last_will.retain = 1;
}

static void handle_data(vnode_t *n, const esp_mqtt_event_t *ev)
{
    // Large payloads arrive in chunks; only the first carries the topic.
    if (ev->current_data_offset == 0) {
        n->rx_len = 0;
        n->rx_retained = ev->retain;
        if (ev->topic_len != (int)strlen(n->topic_display) ||
            strncmp(ev->topic, n->topic_display, ev->topic_len) != 0) {
            ESP_LOGW(TAG, "%s: unexpected topic %.*s", n->id, ev->topic_len, ev->topic);
        }
    }
    size_t room = AI_MSG_MAX_LEN - n->rx_len;
    size_t take = (size_t)ev->data_len < room ? (size_t)ev->data_len : room;
    memcpy(n->rx_buf + n->rx_len, ev->data, take);
    n->rx_len += take;
    n->rx_buf[n->rx_len] = '\0';

    if (ev->current_data_offset + ev->data_len >= ev->total_data_len) {
        bool truncated = ev->total_data_len > AI_MSG_MAX_LEN;
        event_log_route(ROUTE_DISPLAY_IN, n->topic_display, (size_t)ev->total_data_len);
        ai_message_handle(n, n->rx_buf, n->rx_len, n->rx_retained, truncated);
    }
}

static void on_mqtt_event(void *arg, esp_event_base_t base, int32_t id, void *data)
{
    vnode_t *n = arg;
    esp_mqtt_event_handle_t ev = data;
    char uri[URI_MAX];

    switch ((esp_mqtt_event_id_t)id) {
    case MQTT_EVENT_BEFORE_CONNECT:
        mqtt_broker_uri(uri, sizeof(uri));
        ESP_LOGI(TAG, "%s: connecting to %s", n->id, uri);
        break;
    case MQTT_EVENT_CONNECTED:
        if (n->publish_count > 0 || n->mqtt_reconnects > 0) {
            n->mqtt_reconnects++;
        }
        n->mqtt_connected = true;
        // Retained "online" replaces the Last Will's "offline"; resubscribe on every
        // (re)connect. The broker then re-delivers the retained display message.
        esp_mqtt_client_publish(ev->client, n->topic_status, "online", 0, 1, 1);
        event_log_route(ROUTE_STATUS, n->topic_status, 6);
        esp_mqtt_client_subscribe(ev->client, n->topic_display, 1);
        ESP_LOGI(TAG, "%s: connected; published online, subscribing to %s", n->id,
                 n->topic_display);
        break;
    case MQTT_EVENT_SUBSCRIBED:
        ESP_LOGI(TAG, "%s: subscribed to %s", n->id, n->topic_display);
        break;
    case MQTT_EVENT_DISCONNECTED:
        if (n->mqtt_connected) {
            ESP_LOGW(TAG, "%s: disconnected; will reconnect", n->id);
        }
        n->mqtt_connected = false;
        break;
    case MQTT_EVENT_DATA:
        handle_data(n, ev);
        break;
    case MQTT_EVENT_ERROR:
        mqtt_broker_uri(uri, sizeof(uri));
        if (ev->error_handle->error_type == MQTT_ERROR_TYPE_CONNECTION_REFUSED) {
            ESP_LOGE(TAG, "%s: broker refused connection (code %d) — check username/password",
                     n->id, ev->error_handle->connect_return_code);
        } else if (ev->error_handle->error_type == MQTT_ERROR_TYPE_TCP_TRANSPORT) {
            ESP_LOGE(TAG, "%s: transport error (errno %d) — is %s reachable?", n->id,
                     ev->error_handle->esp_transport_sock_errno, uri);
        }
        break;
    default:
        break;
    }
}

bool mqtt_node_publish_sensors(vnode_t *n, const char *json, int len)
{
    if (!n->client || !n->mqtt_connected) {
        return false;
    }
    if (esp_mqtt_client_publish(n->client, n->topic_sensors, json, len, 0, 0) < 0) {
        return false;
    }
    n->publish_count++;
    event_log_route(ROUTE_SENSORS, n->topic_sensors, (size_t)len);
    return true;
}

static void start_node(vnode_t *n, int i, const app_config_t *c, const char *uri)
{
    esp_mqtt_client_config_t cfg;
    node_config(n, i, c, uri, &cfg);
    if (!n->client) {
        n->client = esp_mqtt_client_init(&cfg);
        if (!n->client) {
            ESP_LOGE(TAG, "%s: client init failed", n->id);
            return;
        }
        esp_mqtt_client_register_event(n->client, ESP_EVENT_ANY_ID, on_mqtt_event, n);
    } else if (esp_mqtt_set_config(n->client, &cfg) != ESP_OK) {
        ESP_LOGE(TAG, "%s: applying the new settings failed", n->id);
    }
    esp_mqtt_client_start(n->client);
}

void mqtt_nodes_start(void)
{
    app_config_t c;
    app_config_get(&c);
    set_uri(&c);
    for (int i = 0; i < NODE_COUNT; i++) {
        snprintf(s_client_ids[i], sizeof(s_client_ids[i]), "intellithings-%.*s",
                 (int)sizeof(g_nodes[i].id) - 1, g_nodes[i].id);
    }
    s_started = true;
    if (!c.host[0]) {
        ESP_LOGE(TAG, "No broker configured; set one in the panel's Settings");
        return;
    }
    char uri[URI_MAX];
    mqtt_broker_uri(uri, sizeof(uri));
    for (int i = 0; i < NODE_COUNT; i++) {
        start_node(&g_nodes[i], i, &c, uri);
    }
    ctrl_start();
}

// Runs on its own task: esp_mqtt_client_stop() blocks until the client task has exited (up to
// a connect timeout) and can't be called from an MQTT event handler. Client handles are never
// freed, so other tasks may keep calling publish on them; while stopped those calls just fail.
static void reconf_task(void *arg)
{
    for (;;) {
        ulTaskNotifyTake(pdTRUE, portMAX_DELAY);
        if (!s_started) {
            continue;  // not online yet; mqtt_nodes_start() reads the new settings itself
        }
        s_reconfiguring = true;
        app_config_t c;
        app_config_get(&c);
        set_uri(&c);  // the state shows the new broker straight away
        char uri[URI_MAX];
        mqtt_broker_uri(uri, sizeof(uri));
        ESP_LOGI(TAG, "New broker settings: leaving the old broker, then %s", uri);

        for (int i = 0; i < NODE_COUNT; i++) {
            vnode_t *n = &g_nodes[i];
            if (!n->client) {
                continue;
            }
            if (n->mqtt_connected) {
                // Leave the old broker cleanly: a stop sends DISCONNECT, so the Last Will
                // wouldn't fire. QoS 0 so it isn't re-sent from the outbox to the new broker.
                esp_mqtt_client_publish(n->client, n->topic_status, "offline", 0, 0, 1);
            }
            n->mqtt_connected = false;
            esp_mqtt_client_stop(n->client);
            n->mqtt_connected = false;  // no events after stop; this is final
        }
        ctrl_mqtt_stop();

        for (int i = 0; i < NODE_COUNT; i++) {
            start_node(&g_nodes[i], i, &c, uri);
        }
        ctrl_mqtt_restart(&c, uri);
        ESP_LOGI(TAG, "Connecting to %s", uri);
        s_reconfiguring = false;
    }
}

void mqtt_nodes_init(void)
{
    BaseType_t ok = xTaskCreate(reconf_task, "mqtt_cfg", 4096, NULL, 4, &s_reconf_task);
    configASSERT(ok == pdPASS);
}

void mqtt_reconfigure_async(void)
{
    s_reconfiguring = true;
    xTaskNotifyGive(s_reconf_task);
}

bool mqtt_reconfiguring(void)
{
    return s_reconfiguring;
}
