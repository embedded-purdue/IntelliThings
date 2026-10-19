// One MQTT connection per virtual node, with its Last Will and display subscription.
// Broker settings come from app_config; a change restarts every client without a reboot.
#pragma once

#include <stdbool.h>
#include <stddef.h>

#include "app_config.h"
#include "mqtt_client.h"
#include "virtual_node.h"

void mqtt_nodes_init(void);   // at boot: creates the reconfigure task
void mqtt_nodes_start(void);  // after the first IP: node clients + the emulator's control client

// "mqtt://host:port" currently in use.
void mqtt_broker_uri(char *buf, size_t len);
// Fills an esp-mqtt config from app_config (shared with the control client). client_id and
// the Last Will are left to the caller. String pointers point into cfg.
void mqtt_fill_config(esp_mqtt_client_config_t *out, const app_config_t *cfg, const char *uri);

// Publish a node's sensor snapshot (QoS 0) and log it for the route view. False if offline.
bool mqtt_node_publish_sensors(vnode_t *n, const char *json, int len);

// Restart every MQTT client with the current app_config, on the reconfigure task.
void mqtt_reconfigure_async(void);
bool mqtt_reconfiguring(void);  // true from the request until every client has restarted
