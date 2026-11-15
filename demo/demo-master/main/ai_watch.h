// What the AI says, for the admin page's "Now playing" check: a read-only subscriber to the AI
// pipeline's topics (docs/interfaces/mqtt.md): intellithings/ai/summary,
// intellithings/ai/inference_status and every node's intellithings/<node_id>/display. The demo
// master never publishes to them.
#pragma once

#include "json_writer.h"
#include "mqtt_client.h"

void ai_watch_init(void);
void ai_watch_subscribe(esp_mqtt_client_handle_t client);  // on every (re)connect
void ai_watch_on_data(const esp_mqtt_event_t *ev);          // MQTT_EVENT_DATA

// {"summary":{"text","ts","age_s"}|null,"thinking":{"state","ts","age_s"}|null,
//  "rooms":[{"node","text","ts","age_s"},...]}; age_s = seconds since it arrived.
void ai_watch_write(jw_t *w);
