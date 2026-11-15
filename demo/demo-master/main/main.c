// IntelliThings demo master (ECE SPARK booth): the scenario library and randomizer for the demo
// home, the game buzzer, MQTT publishing under intellithings/demo/# and the web interface.
// See demo/README.md for the plan and demo/demo-master/README.md for this firmware.

#include "ai_watch.h"
#include "button.h"
#include "demo_home.h"
#include "engine.h"
#include "esp_app_desc.h"
#include "esp_event.h"
#include "esp_log.h"
#include "esp_netif.h"
#include "event_log.h"
#include "library.h"
#include "mqtt_link.h"
#include "nvs_flash.h"
#include "settings.h"
#include "time_sync.h"
#include "web_server.h"
#include "wifi_manager.h"

static const char *TAG = "main";

static void on_ip(bool first_time)
{
    char ip[16];
    wifi_get_ip(ip, sizeof(ip));
    event_log("Wi-Fi up, demo page at http://%s/", ip);
    if (first_time) {
        time_sync_start();
        mqtt_link_start();
        web_server_start();
    }
}

static void on_press(void)
{
    engine_press(PRESS_BUZZER, NULL);
}

void app_main(void)
{
    ESP_LOGI(TAG, "IntelliThings demo master %s", esp_app_get_description()->version);

    esp_err_t err = nvs_flash_init();
    if (err == ESP_ERR_NVS_NO_FREE_PAGES || err == ESP_ERR_NVS_NEW_VERSION_FOUND) {
        ESP_ERROR_CHECK(nvs_flash_erase());
        err = nvs_flash_init();
    }
    ESP_ERROR_CHECK(err);
    ESP_ERROR_CHECK(esp_netif_init());
    ESP_ERROR_CHECK(esp_event_loop_create_default());

    event_log_init();
    time_sync_init();
    demo_home_init();
    settings_init();
    library_init();
    ai_watch_init();
    mqtt_link_init(engine_on_connected);  // the client itself starts after the first IP
    engine_init();
    button_init(on_press);
    wifi_manager_start(on_ip);
}
