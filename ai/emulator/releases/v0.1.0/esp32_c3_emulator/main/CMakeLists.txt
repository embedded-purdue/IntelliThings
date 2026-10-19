idf_component_register(
    SRCS
        "main.c"
        "sim_model.c"
        "scenarios.c"
        "payload.c"
        "virtual_node.c"
        "wifi_manager.c"
        "time_sync.c"
        "mqtt_node.c"
        "ai_message.c"
        "emulator_ctrl.c"
        "web_server.c"
    INCLUDE_DIRS "."
    PRIV_REQUIRES esp_wifi esp_netif esp_event esp_timer esp_http_server nvs_flash
    EMBED_TXTFILES "web/index.html"
)
