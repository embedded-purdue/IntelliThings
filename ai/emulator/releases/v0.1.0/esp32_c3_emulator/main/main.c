// IntelliThings emulator: one ESP32-C3 posing as three sensor nodes (kitchen, bedroom,
// living room) on the real MQTT contract, plus a web control panel for the AI team.
// See ai/emulator/README.md.
#include <stdio.h>

#include "ai_message.h"
#include "emulator_ctrl.h"
#include "esp_event.h"
#include "esp_log.h"
#include "esp_netif.h"
#include "esp_timer.h"
#include "freertos/FreeRTOS.h"
#include "freertos/task.h"
#include "mdns.h"
#include "mqtt_client.h"
#include "mqtt_node.h"
#include "nvs_flash.h"
#include "payload.h"
#include "time_sync.h"
#include "virtual_node.h"
#include "web_server.h"
#include "wifi_manager.h"

static const char *TAG = "emulator";

static void start_mdns(void)
{
    if (mdns_init() != ESP_OK) {
        ESP_LOGW(TAG, "mDNS unavailable; use the IP address");
        return;
    }
    mdns_hostname_set(CONFIG_EMU_HOSTNAME);
    mdns_instance_name_set("IntelliThings emulator");
    mdns_service_add(NULL, "_http", "_tcp", 80, NULL, 0);
}

static void on_ip(bool first_time)
{
    char ip[16];
    wifi_get_ip(ip, sizeof(ip));
    if (first_time) {
        time_sync_start();
        start_mdns();
        web_server_start();
        ctrl_start();
        mqtt_nodes_start();
    }
    ESP_LOGI(TAG, "Control panel: http://%s/  (or http://%s.local/)", ip, CONFIG_EMU_HOSTNAME);
}

static void publish_all(void)
{
    char ts[40];
    bool has_ts = time_iso8601(ts, sizeof(ts));
    int64_t now = esp_timer_get_time();

    for (int i = 0; i < NODE_COUNT; i++) {
        vnode_t *n = &g_nodes[i];
        sim_reading_t r;
        const char *mode;
        nodes_lock();
        sim_read(&n->sim, &r);
        n->last = r;
        n->has_last = true;
        n->last_publish_us = now;
        mode = sim_mode(&n->sim);
        const char *what = n->sim.scenario     ? n->sim.scenario->name
                           : n->sim.auto_event ? n->sim.auto_event->name
                                               : "";
        nodes_unlock();

        char json[384];
        int len = payload_sensors_json(json, sizeof(json), n->id, &r, has_ts ? ts : NULL);
        if (len < 0) {
            ESP_LOGE(TAG, "%s: payload too long", n->id);
            continue;
        }
        static bool shown[NODE_COUNT];
        if (!shown[i]) {
            shown[i] = true;  // show the exact payload format once per node
            ESP_LOGI(TAG, "Example payload on %s:\n%s", n->topic_sensors, json);
        }
        bool sent = false;
        if (n->mqtt_connected) {
            sent = esp_mqtt_client_publish(n->client, n->topic_sensors, json, len, 0, 0) >= 0;
            if (sent) {
                n->publish_count++;
            }
        }
#if CONFIG_EMU_LOG_READINGS
        ESP_LOGI("node",
                 "%-16s T=%.1fC RH=%.1f%% CO2=%d VOC=%d PM1/2.5/10=%d/%d/%d lux=%.0f "
                 "dist=%dmm %s [%s%s%s]%s",
                 n->id, (double)r.v[METRIC_TEMPERATURE], (double)r.v[METRIC_HUMIDITY],
                 (int)r.v[METRIC_CO2], (int)r.v[METRIC_VOC], (int)r.v[METRIC_PM1_0],
                 (int)r.v[METRIC_PM2_5], (int)r.v[METRIC_PM10], (double)r.v[METRIC_LUX],
                 (int)r.v[METRIC_DISTANCE], r.presence ? "PRESENT" : "empty", mode,
                 what[0] ? ":" : "", what, sent ? "" : "  (not sent: MQTT offline)");
#else
        (void)mode;
        (void)what;
#endif
    }
}

static void sim_task(void *arg)
{
    const TickType_t tick = pdMS_TO_TICKS(CONFIG_EMU_SIM_TICK_MS);
    TickType_t wake = xTaskGetTickCount();
    int64_t prev_us = esp_timer_get_time();
    int64_t since_publish_ms = 0;

    for (;;) {
        vTaskDelayUntil(&wake, tick);
        int64_t now = esp_timer_get_time();
        float dt = (float)(now - prev_us) / 1e6f;
        prev_us = now;

        float hour = time_local_hour();
        nodes_lock();
        for (int i = 0; i < NODE_COUNT; i++) {
            sim_step(&g_nodes[i].sim, dt * CONFIG_EMU_SIM_SPEED, hour);
        }
        nodes_unlock();

        since_publish_ms += (int64_t)(dt * 1000.0f + 0.5f);
        if (since_publish_ms >= CONFIG_EMU_PUBLISH_INTERVAL_MS) {
            since_publish_ms -= CONFIG_EMU_PUBLISH_INTERVAL_MS;
            if (since_publish_ms >= CONFIG_EMU_PUBLISH_INTERVAL_MS) {
                since_publish_ms = 0;  // fell behind (e.g. long block); don't burst
            }
            publish_all();  // every interval, even if nothing changed
            ctrl_publish_state(true);
        } else {
            ctrl_publish_state(false);  // only if a command changed something
        }
    }
}

void app_main(void)
{
    ESP_LOGI(TAG, "IntelliThings emulator booting: 3 virtual nodes, publish every %d ms",
             CONFIG_EMU_PUBLISH_INTERVAL_MS);

    esp_err_t err = nvs_flash_init();  // required by Wi-Fi
    if (err == ESP_ERR_NVS_NO_FREE_PAGES || err == ESP_ERR_NVS_NEW_VERSION_FOUND) {
        ESP_ERROR_CHECK(nvs_flash_erase());
        err = nvs_flash_init();
    }
    ESP_ERROR_CHECK(err);
    ESP_ERROR_CHECK(esp_netif_init());
    ESP_ERROR_CHECK(esp_event_loop_create_default());

    esp_log_level_set("mqtt_client", ESP_LOG_WARN);
    esp_log_level_set("transport_base", ESP_LOG_WARN);

    time_sync_init();
    ai_message_init();
    ctrl_init();
    nodes_init();

    if (!CONFIG_EMU_WIFI_SSID[0]) {
        ESP_LOGE(TAG, "No Wi-Fi SSID configured. Set CONFIG_EMU_WIFI_SSID in sdkconfig.local "
                      "or `idf.py menuconfig`; running the simulation offline.");
    } else {
        wifi_manager_start(on_ip);
    }

    xTaskCreate(sim_task, "sim", 6144, NULL, 5, NULL);
}
