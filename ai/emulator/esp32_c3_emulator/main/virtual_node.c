#include "virtual_node.h"

#include <stdlib.h>
#include <string.h>

#include "esp_log.h"
#include "esp_random.h"
#include "freertos/FreeRTOS.h"
#include "freertos/semphr.h"
#include "payload.h"

static const char *TAG = "nodes";

vnode_t g_nodes[NODE_COUNT];
static SemaphoreHandle_t s_lock;

static void node_setup(vnode_t *n, const char *id, const room_profile_t *profile)
{
    strlcpy(n->id, id, sizeof(n->id));
    n->profile = profile;
    // Seed from the hardware RNG so the three rooms never move in lockstep.
    sim_init(&n->sim, profile, esp_random() | 1u);
    if (!payload_topic(n->topic_sensors, sizeof(n->topic_sensors), id, "sensors") ||
        !payload_topic(n->topic_status, sizeof(n->topic_status), id, "status") ||
        !payload_topic(n->topic_display, sizeof(n->topic_display), id, "display")) {
        ESP_LOGE(TAG, "Node ID \"%s\" is too long for its topics", id);
        abort();
    }
    ESP_LOGI(TAG, "%-12s -> %s", profile->label, id);
}

void nodes_init(void)
{
    s_lock = xSemaphoreCreateMutex();
    configASSERT(s_lock);
    node_setup(&g_nodes[0], CONFIG_EMU_NODE_ID_KITCHEN, &ROOM_KITCHEN);
    node_setup(&g_nodes[1], CONFIG_EMU_NODE_ID_BEDROOM, &ROOM_BEDROOM);
    node_setup(&g_nodes[2], CONFIG_EMU_NODE_ID_LIVING_ROOM, &ROOM_LIVING_ROOM);
}

void nodes_lock(void)
{
    xSemaphoreTake(s_lock, portMAX_DELAY);
}

void nodes_unlock(void)
{
    xSemaphoreGive(s_lock);
}

vnode_t *node_find(const char *id_or_room)
{
    if (!id_or_room) {
        return NULL;
    }
    for (int i = 0; i < NODE_COUNT; i++) {
        if (strcmp(g_nodes[i].id, id_or_room) == 0 ||
            strcmp(g_nodes[i].profile->name, id_or_room) == 0) {
            return &g_nodes[i];
        }
    }
    return NULL;
}
