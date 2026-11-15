#include "ai_watch.h"

#include <stdio.h>
#include <string.h>

#include "cJSON.h"
#include "esp_timer.h"
#include "freertos/FreeRTOS.h"
#include "freertos/semphr.h"

#define TOPIC_SUMMARY "intellithings/ai/summary"
#define TOPIC_STATUS "intellithings/ai/inference_status"
#define TOPIC_DISPLAY "intellithings/+/display"
#define MAX_ROOMS 6
#define TEXT_MAX 256  // summary: at most 255 characters (mqtt.md)
#define ROOM_TEXT_MAX 160
#define NODE_MAX 33
#define TS_MAX 40  // the pipeline sends microseconds: 2026-11-15T20:11:11.653077+00:00

typedef struct {
    bool set;
    char text[TEXT_MAX];
    char ts[TS_MAX];
    int64_t at_us;
} msg_t;

typedef struct {
    char node[NODE_MAX];
    char text[ROOM_TEXT_MAX];
    char ts[TS_MAX];
    int64_t at_us;
} room_t;

static SemaphoreHandle_t s_lock, s_copy_lock;
static msg_t s_summary, s_status;
static room_t s_rooms[MAX_ROOMS];
static int s_n_rooms;

void ai_watch_init(void)
{
    s_lock = xSemaphoreCreateMutex();
    s_copy_lock = xSemaphoreCreateMutex();
    configASSERT(s_lock && s_copy_lock);
}

void ai_watch_subscribe(esp_mqtt_client_handle_t client)
{
    esp_mqtt_client_subscribe(client, TOPIC_SUMMARY, 0);
    esp_mqtt_client_subscribe(client, TOPIC_STATUS, 0);
    esp_mqtt_client_subscribe(client, TOPIC_DISPLAY, 0);
}

// Copies the JSON payload's `key` (and "ts") into text/ts. An empty payload clears it.
static void take(const char *data, int len, const char *key, char *text, size_t text_len, char *ts,
                 bool *set)
{
    text[0] = ts[0] = '\0';
    *set = false;
    cJSON *root = len > 0 ? cJSON_ParseWithLength(data, len) : NULL;
    const cJSON *v = cJSON_GetObjectItemCaseSensitive(root, key);
    const cJSON *t = cJSON_GetObjectItemCaseSensitive(root, "ts");
    if (cJSON_IsString(v)) {
        strlcpy(text, v->valuestring, text_len);
        *set = true;
    }
    if (cJSON_IsString(t)) {
        strlcpy(ts, t->valuestring, TS_MAX);
    }
    cJSON_Delete(root);
}

void ai_watch_on_data(const esp_mqtt_event_t *ev)
{
    // Whole messages only (they're small); skip the pieces of anything bigger than the buffer.
    if (!ev->topic || ev->current_data_offset != 0 || ev->data_len != ev->total_data_len) {
        return;
    }
    char topic[96];
    int tl = ev->topic_len < (int)sizeof(topic) - 1 ? ev->topic_len : (int)sizeof(topic) - 1;
    memcpy(topic, ev->topic, tl);
    topic[tl] = '\0';
    int64_t now = esp_timer_get_time();

    xSemaphoreTake(s_lock, portMAX_DELAY);
    if (strcmp(topic, TOPIC_SUMMARY) == 0) {
        take(ev->data, ev->data_len, "text", s_summary.text, TEXT_MAX, s_summary.ts, &s_summary.set);
        s_summary.at_us = now;
    } else if (strcmp(topic, TOPIC_STATUS) == 0) {
        take(ev->data, ev->data_len, "state", s_status.text, TEXT_MAX, s_status.ts, &s_status.set);
        s_status.at_us = now;
    } else {
        // intellithings/<node_id>/display
        const char *node = topic + strlen("intellithings/");
        const char *end = strchr(node, '/');
        int nl = end ? (int)(end - node) : 0;
        if (nl > 0 && nl < NODE_MAX) {
            int i = 0;
            while (i < s_n_rooms && (strncmp(s_rooms[i].node, node, nl) || s_rooms[i].node[nl])) {
                i++;
            }
            bool set;
            room_t r = {.at_us = now};
            memcpy(r.node, node, nl);
            take(ev->data, ev->data_len, "text", r.text, ROOM_TEXT_MAX, r.ts, &set);
            if (!set && i < s_n_rooms) {
                memmove(&s_rooms[i], &s_rooms[i + 1], (s_n_rooms - i - 1) * sizeof(room_t));
                s_n_rooms--;  // cleared
            } else if (set && i < s_n_rooms) {
                s_rooms[i] = r;
            } else if (set && s_n_rooms < MAX_ROOMS) {
                s_rooms[s_n_rooms++] = r;
            }
        }
    }
    xSemaphoreGive(s_lock);
}

static void write_msg(jw_t *w, const char *key, const msg_t *m, const char *field, int64_t now)
{
    jw_key(w, false, key);
    if (!m->set) {
        jw_raw(w, "null");
        return;
    }
    jw_raw(w, "{\"");
    jw_raw(w, field);
    jw_raw(w, "\":");
    jw_str(w, m->text);
    jw_key(w, false, "ts");
    jw_str(w, m->ts);
    jw_key(w, false, "age_s");
    jw_num(w, (double)((now - m->at_us) / 1000000LL));
    jw_raw(w, "}");
}

void ai_watch_write(jw_t *w)
{
    // Copied under the lock, written after it (the writer may be sending to a socket).
    static msg_t summary, status;
    static room_t rooms[MAX_ROOMS];
    xSemaphoreTake(s_copy_lock, portMAX_DELAY);
    xSemaphoreTake(s_lock, portMAX_DELAY);
    summary = s_summary;
    status = s_status;
    int n = s_n_rooms;
    memcpy(rooms, s_rooms, n * sizeof(room_t));
    xSemaphoreGive(s_lock);

    int64_t now = esp_timer_get_time();
    jw_raw(w, "{\"rooms\":[");
    for (int i = 0; i < n; i++) {
        jw_raw(w, i ? ",{\"node\":" : "{\"node\":");
        jw_str(w, rooms[i].node);
        jw_key(w, false, "text");
        jw_str(w, rooms[i].text);
        jw_key(w, false, "ts");
        jw_str(w, rooms[i].ts);
        jw_key(w, false, "age_s");
        jw_num(w, (double)((now - rooms[i].at_us) / 1000000LL));
        jw_raw(w, "}");
    }
    jw_raw(w, "]");
    write_msg(w, "summary", &summary, "text", now);
    write_msg(w, "thinking", &status, "state", now);
    jw_raw(w, "}");
    xSemaphoreGive(s_copy_lock);
}
