// The three virtual sensor nodes. Simulation state is shared between the sim task,
// the web server and MQTT handlers, so it's guarded by one lock. Never call MQTT APIs
// while holding it.
#pragma once

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#include "mqtt_client.h"
#include "sim_model.h"

#define NODE_COUNT 3
#define AI_MSG_MAX_LEN 512  // longer display payloads are truncated

typedef struct {
    char id[32];
    const room_profile_t *profile;
    sim_state_t sim;
    sim_reading_t last;
    bool has_last;
    int64_t last_publish_us;
    uint32_t publish_count;

    char topic_sensors[64];
    char topic_status[64];
    char topic_display[64];

    esp_mqtt_client_handle_t client;
    volatile bool mqtt_connected;
    uint32_t mqtt_reconnects;

    // Reassembly of fragmented inbound display messages (touched only by this node's MQTT task)
    char rx_buf[AI_MSG_MAX_LEN + 1];
    size_t rx_len;
    bool rx_retained;
} vnode_t;

extern vnode_t g_nodes[NODE_COUNT];

void nodes_init(void);
void nodes_lock(void);
void nodes_unlock(void);
// Accepts a node ID ("emu-kitchen") or a room name ("kitchen"). NULL if unknown.
vnode_t *node_find(const char *id_or_room);
