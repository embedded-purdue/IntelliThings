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
        "app_config.c"
        "event_log.c"
        "history.c"
    INCLUDE_DIRS "."
    PRIV_REQUIRES esp_app_format esp_wifi esp_netif esp_event esp_timer esp_http_server nvs_flash
)

# Web assets are gzipped at build time and served with "Content-Encoding: gzip".
# Each one is embedded as _binary_<name>_gz_start / _end (e.g. index_html_gz).
idf_build_get_property(python PYTHON)
function(embed_gzipped_asset src)
    get_filename_component(name "${src}" NAME)
    set(in "${CMAKE_CURRENT_SOURCE_DIR}/${src}")
    set(out "${CMAKE_CURRENT_BINARY_DIR}/${name}.gz")
    add_custom_command(OUTPUT "${out}"
        COMMAND "${python}" "${CMAKE_CURRENT_SOURCE_DIR}/../tools/gzip_asset.py" "${in}" "${out}"
        DEPENDS "${in}" "${CMAKE_CURRENT_SOURCE_DIR}/../tools/gzip_asset.py"
        VERBATIM)
    target_add_binary_data(${COMPONENT_LIB} "${out}" BINARY)
endfunction()

embed_gzipped_asset("web/index.html")
embed_gzipped_asset("../../GUIDE.md")  # ai/emulator/GUIDE.md, served at /guide.md
