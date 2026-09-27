#include "payload.h"

#include <stdbool.h>
#include <stdio.h>

bool payload_topic(char *buf, size_t len, const char *node_id, const char *leaf)
{
    int n = snprintf(buf, len, TOPIC_ROOT "/%s/%s", node_id, leaf);
    return n > 0 && (size_t)n < len;
}

int payload_sensors_json(char *buf, size_t len, const char *node_id, const sim_reading_t *r,
                         const char *ts)
{
    int n = snprintf(buf, len, "{\"node_id\":\"%s\"", node_id);
    for (int m = 0; m < METRIC_COUNT && n > 0 && (size_t)n < len; m++) {
        const metric_info_t *mi = &METRIC_INFO[m];
        if (mi->decimals == 0) {
            n += snprintf(buf + n, len - n, ",\"%s\":%ld", mi->key, (long)r->v[m]);
        } else {
            n += snprintf(buf + n, len - n, ",\"%s\":%.1f", mi->key, (double)r->v[m]);
        }
    }
    if (n > 0 && (size_t)n < len) {
        n += snprintf(buf + n, len - n, ",\"presence\":%s", r->presence ? "true" : "false");
    }
    if (ts && n > 0 && (size_t)n < len) {
        n += snprintf(buf + n, len - n, ",\"ts\":\"%s\"", ts);
    }
    if (n > 0 && (size_t)n < len) {
        n += snprintf(buf + n, len - n, "}");
    }
    return (n > 0 && (size_t)n < len) ? n : -1;
}
