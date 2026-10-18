#include "mqtt_node.h"

#include <stdio.h>
#include <string.h>

#include "ai_message.h"
#include "esp_log.h"
#include "mqtt_client.h"
#include "virtual_node.h"

static const char *TAG = "mqtt";

static char s_uri[128];
static char s_client_ids[NODE_COUNT][48];

const char *mqtt_broker_uri(void)
{
    if (!s_uri[0]) {
        snprintf(s_uri, sizeof(s_uri), "mqtt://%s:%d", CONFIG_EMU_MQTT_HOST, CONFIG_EMU_MQTT_PORT);
    }
    return s_uri;
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
        ai_message_handle(n, n->rx_buf, n->rx_len, n->rx_retained, truncated);
    }
}

static void on_mqtt_event(void *arg, esp_event_base_t base, int32_t id, void *data)
{
    vnode_t *n = arg;
    esp_mqtt_event_handle_t ev = data;

    switch ((esp_mqtt_event_id_t)id) {
    case MQTT_EVENT_BEFORE_CONNECT:
        ESP_LOGI(TAG, "%s: connecting to %s", n->id, mqtt_broker_uri());
        break;
    case MQTT_EVENT_CONNECTED:
        if (n->publish_count > 0 || n->mqtt_reconnects > 0) {
            n->mqtt_reconnects++;
        }
        n->mqtt_connected = true;
        // Retained "online" replaces the Last Will's "offline"; resubscribe on every
        // (re)connect. The broker then re-delivers the retained display message.
        esp_mqtt_client_publish(ev->client, n->topic_status, "online", 0, 1, 1);
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
        if (ev->error_handle->error_type == MQTT_ERROR_TYPE_CONNECTION_REFUSED) {
            ESP_LOGE(TAG, "%s: broker refused connection (code %d) — check username/password",
                     n->id, ev->error_handle->connect_return_code);
        } else if (ev->error_handle->error_type == MQTT_ERROR_TYPE_TCP_TRANSPORT) {
            ESP_LOGE(TAG, "%s: transport error (errno %d) — is %s reachable?", n->id,
                     ev->error_handle->esp_transport_sock_errno, mqtt_broker_uri());
        }
        break;
    default:
        break;
    }
}

void mqtt_nodes_start(void)
{
    if (!CONFIG_EMU_MQTT_HOST[0]) {
        ESP_LOGE(TAG, "No broker configured (CONFIG_EMU_MQTT_HOST); MQTT disabled");
        return;
    }
    for (int i = 0; i < NODE_COUNT; i++) {
        vnode_t *n = &g_nodes[i];
        snprintf(s_client_ids[i], sizeof(s_client_ids[i]), "intellithings-%s", n->id);

        esp_mqtt_client_config_t cfg = {
            .broker.address.uri = mqtt_broker_uri(),
            .credentials.client_id = s_client_ids[i],
            .credentials.username = CONFIG_EMU_MQTT_USERNAME[0] ? CONFIG_EMU_MQTT_USERNAME : NULL,
            .credentials.authentication.password =
                CONFIG_EMU_MQTT_PASSWORD[0] ? CONFIG_EMU_MQTT_PASSWORD : NULL,
            .session.keepalive = 15,
            .session.last_will = {
                .topic = n->topic_status,
                .msg = "offline",
                .qos = 1,
                .retain = 1,
            },
            .network.reconnect_timeout_ms = 5000,
            .buffer.size = 1024,
        };
        n->client = esp_mqtt_client_init(&cfg);
        if (!n->client) {
            ESP_LOGE(TAG, "%s: client init failed", n->id);
            continue;
        }
        esp_mqtt_client_register_event(n->client, ESP_EVENT_ANY_ID, on_mqtt_event, n);
        esp_mqtt_client_start(n->client);
    }
}
