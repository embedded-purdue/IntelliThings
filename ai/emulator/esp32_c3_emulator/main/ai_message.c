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

static ai_msg_t s_history[HISTORY_LEN];
static uint32_t s_next_seq = 1;  // slot of seq s is (s - 1) % HISTORY_LEN
static ai_msg_t s_latest[NODE_COUNT];
static bool s_has_latest[NODE_COUNT];
static ai_topic_msg_t s_ai[2];
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

    int idx = (int)(node - g_nodes);
    ai_msg_t entry = {0};
    ai_msg_t *e = &entry;
    strlcpy(e->node_id, node->id, sizeof(e->node_id));
    strlcpy(e->label, node->profile->label, sizeof(e->label));
    e->retained = retained;
    e->truncated = truncated;
    e->received_us = esp_timer_get_time();
    time_iso8601(e->received, sizeof(e->received));

    cJSON *root = cJSON_ParseWithLength(payload, len);
    const cJSON *text = root ? cJSON_GetObjectItemCaseSensitive(root, "text") : NULL;
    if (cJSON_IsString(text) && text->valuestring) {
        strlcpy(e->text, text->valuestring, sizeof(e->text));
        const cJSON *ts = cJSON_GetObjectItemCaseSensitive(root, "ts");
        if (cJSON_IsString(ts) && ts->valuestring) {
            strlcpy(e->ts, ts->valuestring, sizeof(e->ts));
        }
    } else {
        // Not the agreed {"text","ts"} shape: show it as-is so the sender can see the problem.
        e->raw = true;
        strlcpy(e->text, payload, sizeof(e->text));
    }
    cJSON_Delete(root);

    // One printf call so the banner can't interleave with log lines from other tasks.
    char room[24];
    upper_copy(room, sizeof(room), node->profile->label);
    // Truncation usually breaks the JSON as well, so report it rather than "not JSON".
    char note[64] = "";
    if (truncated) {
        snprintf(note, sizeof(note), "(payload over %d bytes; truncated)\n", AI_MSG_MAX_LEN);
    } else if (e->raw) {
        strlcpy(note, "(payload was not {\"text\":...} JSON; shown raw)\n", sizeof(note));
    }
    char banner[AI_TEXT_MAX + 400];  // worst case ~700 bytes; -Os checks the bound
    snprintf(banner, sizeof(banner),
             "\n==================================================\n"
             "[AI MESSAGE] %.23s  (%.31s)%s\n"
             "%.400s\n"
             "%s%.39s%s%.63s"
             "==================================================\n",
             room, node->id, retained ? "  [retained]" : "", e->text,
             e->ts[0] ? "ts: " : "", e->ts, e->ts[0] ? "\n" : "", note);
    printf("%s", banner);
    fflush(stdout);
    // The banner bypasses the log hook; put a short line in the panel's system log too.
    ESP_LOGI(TAG, "%s: AI message%s (%u chars)", node->id, retained ? " [retained]" : "",
             (unsigned)strlen(e->text));

    xSemaphoreTake(s_lock, portMAX_DELAY);
    e->seq = s_next_seq++;
    s_history[(e->seq - 1) % HISTORY_LEN] = *e;
    s_latest[idx] = *e;
    s_has_latest[idx] = true;
    xSemaphoreGive(s_lock);
}

void ai_message_handle_ai_topic(ai_topic_t which, const char *payload, size_t len, bool retained)
{
    ai_topic_msg_t m = {0};
    m.received_us = esp_timer_get_time();
    m.retained = retained;
    if (len == 0) {
        // Empty retained payload: the topic was cleared.
        xSemaphoreTake(s_lock, portMAX_DELAY);
        s_ai[which].valid = false;
        xSemaphoreGive(s_lock);
        return;
    }
    m.valid = true;
    const char *field = which == AI_TOPIC_SUMMARY ? "text" : "state";
    cJSON *root = cJSON_ParseWithLength(payload, len);
    const cJSON *v = root ? cJSON_GetObjectItemCaseSensitive(root, field) : NULL;
    if (cJSON_IsString(v) && v->valuestring) {
        strlcpy(m.text, v->valuestring, sizeof(m.text));
        const cJSON *ts = cJSON_GetObjectItemCaseSensitive(root, "ts");
        if (cJSON_IsString(ts) && ts->valuestring) {
            strlcpy(m.ts, ts->valuestring, sizeof(m.ts));
        }
    } else {
        m.raw = true;
        strlcpy(m.text, payload, sizeof(m.text));
    }
    cJSON_Delete(root);

    if (which == AI_TOPIC_SUMMARY) {
        ESP_LOGI(TAG, "AI summary%s: %.60s%s", retained ? " [retained]" : "", m.text,
                 strlen(m.text) > 60 ? "..." : "");
    } else {
        ESP_LOGI(TAG, "AI agent: %s%s", m.text, retained ? " [retained]" : "");
    }
    xSemaphoreTake(s_lock, portMAX_DELAY);
    s_ai[which] = m;
    xSemaphoreGive(s_lock);
}

static void add_msg_json(cJSON *arr, const ai_msg_t *e, int64_t now)
{
    cJSON *o = cJSON_CreateObject();
    cJSON_AddNumberToObject(o, "seq", e->seq);
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

char *ai_message_list_json(void)
{
    cJSON *root = cJSON_CreateObject();
    cJSON *arr = cJSON_AddArrayToObject(root, "messages");
    int64_t now = esp_timer_get_time();

    xSemaphoreTake(s_lock, portMAX_DELAY);
    uint32_t newest = s_next_seq - 1;
    uint32_t count = newest < HISTORY_LEN ? newest : HISTORY_LEN;
    for (uint32_t i = 0; i < count; i++) {
        add_msg_json(arr, &s_history[(newest - i - 1) % HISTORY_LEN], now);
    }
    xSemaphoreGive(s_lock);

    char *out = cJSON_PrintUnformatted(root);
    cJSON_Delete(root);
    return out;
}

size_t ai_message_copy(uint32_t after, ai_msg_t *out, size_t max, bool *gap)
{
    size_t n = 0;
    xSemaphoreTake(s_lock, portMAX_DELAY);
    uint32_t newest = s_next_seq - 1;
    uint32_t oldest = newest >= HISTORY_LEN ? newest - HISTORY_LEN + 1 : 1;
    if (after > newest) {
        after = 0;  // a seq from before a reboot: start over
    }
    uint32_t from = after + 1;
    *gap = from < oldest && newest > 0;
    if (from < oldest) {
        from = oldest;
    }
    for (uint32_t seq = from; seq <= newest && n < max; seq++) {
        out[n++] = s_history[(seq - 1) % HISTORY_LEN];
    }
    xSemaphoreGive(s_lock);
    return n;
}

bool ai_message_latest(int node_index, ai_msg_t *out)
{
    if (node_index < 0 || node_index >= NODE_COUNT) {
        return false;
    }
    xSemaphoreTake(s_lock, portMAX_DELAY);
    bool has = s_has_latest[node_index];
    if (has) {
        *out = s_latest[node_index];
    }
    xSemaphoreGive(s_lock);
    return has;
}

void ai_message_ai_topic(ai_topic_t which, ai_topic_msg_t *out)
{
    xSemaphoreTake(s_lock, portMAX_DELAY);
    *out = s_ai[which];
    xSemaphoreGive(s_lock);
}
