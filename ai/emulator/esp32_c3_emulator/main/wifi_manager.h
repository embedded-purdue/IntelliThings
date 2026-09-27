#pragma once

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

typedef void (*wifi_ip_cb_t)(bool first_time);

// Station mode with automatic reconnect (exponential backoff, 1 s .. 30 s).
// on_ip runs on the default event loop every time an IP is obtained.
void wifi_manager_start(wifi_ip_cb_t on_ip);
bool wifi_is_connected(void);
void wifi_get_ip(char *buf, size_t len);  // "" when not connected
int wifi_get_rssi(void);                  // 0 when not connected
uint32_t wifi_reconnect_count(void);
