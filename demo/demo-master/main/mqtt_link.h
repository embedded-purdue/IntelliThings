// The demo master's one MQTT connection. Last Will: "offline" (retained) on <prefix>/status;
// "online" is published on every connect. The broker and the prefix come from settings.
#pragma once

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

typedef void (*mqtt_link_cb_t)(void);

void mqtt_link_init(mqtt_link_cb_t on_connected);
void mqtt_link_start(void);  // after the first IP
// Applies new settings: says "offline" on the old status topic, then reconnects. From any task
// except the MQTT event handler.
void mqtt_link_restart(void);

bool mqtt_link_connected(void);
// Publishes <prefix>/<leaf> (QoS 1). False when not connected or the client refuses.
bool mqtt_link_publish(const char *leaf, const char *data, int len, bool retain);
// The full topic for a leaf, with the current prefix.
void mqtt_link_topic(char *buf, size_t len, const char *leaf);

uint32_t mqtt_link_sent(void);       // messages handed to the client since boot
uint32_t mqtt_link_reconnects(void);
void mqtt_link_uri(char *buf, size_t len);  // "mqtt://host:port" in use
void mqtt_link_last_error(char *buf, size_t len);  // "" if none
