#include "ai_message.h"

#include <ctype.h>
#include <stdio.h>
#include <string.h>

#include "cJSON.h"
#include "esp_log.h"
#include "esp_timer.h"
#include "freertos/FreeRTOS.h"
#include "freertos/semphr.h"
#include "time_sync.h"

static const char *TAG = "ai_msg";

#define HISTORY_LEN 20
#define TEXT_MAX 400

typedef struct {
    char node_id[32];
    char label[24];
    char text[TEXT_MAX + 1];
    char ts[40];        // from the payload
    char received[40];  // local clock when it arrived ("" if not synced)
    int64_t received_us;
    bool retained;
    bool raw;           // payload wasn't {"text": ...}
} ai_entry_t;

static ai_entry_t s_history[HISTORY_LEN];
static int s_count;
static int s_head;  // next slot to write
static SemaphoreHandle_t s_lock;

void ai_message_init(void)
{
    s_lock = xSemaphoreCreateMutex();
    configASSERT(s_lock);
}

static void upper_copy(char *dst, size_t len, const char *src)
{
    size_t i = 0;
    for (; src[i] && i + 1 < len; i++) {
        dst[i] = (char)toupper((unsigned char)src[i]);
    }
    dst[i] = '\0';
}

void ai_message_handle(const vnode_t *node, const char *payload, size_t len, bool retained,
                       bool truncated)
{
    if (len == 0) {
        ESP_LOGI(TAG, "%s: display topic cleared (empty retained message)", node->id);
        return;
    }

    ai_entry_t e = {0};
    strlcpy(e.node_id, node->id, sizeof(e.node_id));
    strlcpy(e.label, node->profile->label, sizeof(e.label));
    e.retained = retained;
    e.received_us = esp_timer_get_time();
    time_iso8601(e.received, sizeof(e.received));

    cJSON *root = cJSON_ParseWithLength(payload, len);
    const cJSON *text = root ? cJSON_GetObjectItemCaseSensitive(root, "text") : NULL;
    if (cJSON_IsString(text) && text->valuestring) {
        strlcpy(e.text, text->valuestring, sizeof(e.text));
        const cJSON *ts = cJSON_GetObjectItemCaseSensitive(root, "ts");
        if (cJSON_IsString(ts) && ts->valuestring) {
            strlcpy(e.ts, ts->valuestring, sizeof(e.ts));
        }
    } else {
        // Not the agreed {"text","ts"} shape: show it as-is so the sender can see the problem.
        e.raw = true;
        strlcpy(e.text, payload, sizeof(e.text));
    }
    cJSON_Delete(root);

    // One printf call so the banner can't interleave with log lines from other tasks.
    char room[24];
    upper_copy(room, sizeof(room), node->profile->label);
    // Truncation usually breaks the JSON as well, so report it rather than "not JSON".
    char note[64] = "";
    if (truncated) {
        snprintf(note, sizeof(note), "(payload over %d bytes; truncated)\n", AI_MSG_MAX_LEN);
    } else if (e.raw) {
        strlcpy(note, "(payload was not {\"text\":...} JSON; shown raw)\n", sizeof(note));
    }
    char banner[TEXT_MAX + 320];
    snprintf(banner, sizeof(banner),
             "\n==================================================\n"
             "[AI MESSAGE] %s  (%s)%s\n"
             "%s\n"
             "%s%s%s%s"
             "==================================================\n",
             room, node->id, retained ? "  [retained]" : "", e.text,
             e.ts[0] ? "ts: " : "", e.ts, e.ts[0] ? "\n" : "",
             note);
    printf("%s", banner);
    fflush(stdout);

    xSemaphoreTake(s_lock, portMAX_DELAY);
    s_history[s_head] = e;
    s_head = (s_head + 1) % HISTORY_LEN;
    if (s_count < HISTORY_LEN) {
        s_count++;
    }
    xSemaphoreGive(s_lock);
}

char *ai_message_list_json(void)
{
    cJSON *root = cJSON_CreateObject();
    cJSON *arr = cJSON_AddArrayToObject(root, "messages");
    int64_t now = esp_timer_get_time();

    xSemaphoreTake(s_lock, portMAX_DELAY);
    for (int i = 0; i < s_count; i++) {
        const ai_entry_t *e = &s_history[(s_head - 1 - i + HISTORY_LEN) % HISTORY_LEN];
        cJSON *o = cJSON_CreateObject();
        cJSON_AddStringToObject(o, "node_id", e->node_id);
        cJSON_AddStringToObject(o, "room", e->label);
        cJSON_AddStringToObject(o, "text", e->text);
        cJSON_AddStringToObject(o, "ts", e->ts);
        cJSON_AddStringToObject(o, "received", e->received);
        cJSON_AddNumberToObject(o, "age_s", (double)((now - e->received_us) / 1000000LL));
        cJSON_AddBoolToObject(o, "retained", e->retained);
        cJSON_AddBoolToObject(o, "raw", e->raw);
        cJSON_AddItemToArray(arr, o);
    }
    xSemaphoreGive(s_lock);

    char *out = cJSON_PrintUnformatted(root);
    cJSON_Delete(root);
    return out;
}
