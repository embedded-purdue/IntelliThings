// AI messages arriving on intellithings/<node_id>/display: printed as a banner on serial
// (the emulator's stand-in for the node display) and kept for the control panel.
#pragma once

#include <stdbool.h>
#include <stddef.h>

#include "virtual_node.h"

void ai_message_init(void);
// payload is NUL-terminated; truncated says it was cut to AI_MSG_MAX_LEN.
void ai_message_handle(const vnode_t *node, const char *payload, size_t len, bool retained,
                       bool truncated);
// {"messages":[...]} newest first. Caller frees with cJSON_free().
char *ai_message_list_json(void);
