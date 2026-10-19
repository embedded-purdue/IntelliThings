// AI messages arriving on intellithings/<node_id>/display: printed as a banner on serial
// (the emulator's stand-in for the node display) and kept for the control panel. Also keeps
// the latest intellithings/ai/summary and intellithings/ai/inference_status (read-only).
#pragma once

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#include "virtual_node.h"

#define AI_TEXT_MAX 400
#define AI_SUMMARY_MAX 300

typedef struct {
    uint32_t seq;  // 1, 2, ... in arrival order
    char node_id[32];
    char label[24];
    char text[AI_TEXT_MAX + 1];
    char ts[40];        // from the payload
    char received[40];  // local clock when it arrived ("" if not synced)
    int64_t received_us;
    bool retained;
    bool raw;           // payload wasn't {"text": ...}
    bool truncated;     // payload was over AI_MSG_MAX_LEN bytes
} ai_msg_t;

typedef enum { AI_TOPIC_SUMMARY, AI_TOPIC_INFERENCE } ai_topic_t;

typedef struct {
    bool valid;
    char text[AI_SUMMARY_MAX + 1];  // summary text, or inference "state"
    char ts[40];
    int64_t received_us;
    bool retained;
    bool raw;
} ai_topic_msg_t;

void ai_message_init(void);
// payload is NUL-terminated; truncated says it was cut to AI_MSG_MAX_LEN.
void ai_message_handle(const vnode_t *node, const char *payload, size_t len, bool retained,
                       bool truncated);
void ai_message_handle_ai_topic(ai_topic_t which, const char *payload, size_t len, bool retained);

// {"messages":[...]} newest first (GET /api/messages). Caller frees with cJSON_free().
char *ai_message_list_json(void);
// Copy up to max messages with seq > after, oldest first; *gap if some were already dropped.
size_t ai_message_copy(uint32_t after, ai_msg_t *out, size_t max, bool *gap);
// Latest message for node index i (g_nodes order). False if none yet.
bool ai_message_latest(int node_index, ai_msg_t *out);
void ai_message_ai_topic(ai_topic_t which, ai_topic_msg_t *out);
