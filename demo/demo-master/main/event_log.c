#include "event_log.h"

#include <stdarg.h>
#include <stdio.h>
#include <string.h>

#include "esp_log.h"
#include "esp_timer.h"
#include "freertos/FreeRTOS.h"
#include "freertos/semphr.h"

static const char *TAG = "event";

static struct {
    uint32_t id;
    uint32_t t_s;
    char text[EVENT_TEXT_MAX];
} s_ring[EVENT_LOG_LEN];
static uint32_t s_next = 1;  // id of the next event
static SemaphoreHandle_t s_lock, s_copy_lock;

void event_log_init(void)
{
    s_lock = xSemaphoreCreateMutex();
    s_copy_lock = xSemaphoreCreateMutex();
    configASSERT(s_lock && s_copy_lock);
}

void event_log(const char *fmt, ...)
{
    char text[EVENT_TEXT_MAX];
    va_list ap;
    va_start(ap, fmt);
    vsnprintf(text, sizeof(text), fmt, ap);
    va_end(ap);
    ESP_LOGI(TAG, "%s", text);

    xSemaphoreTake(s_lock, portMAX_DELAY);
    uint32_t id = s_next++;
    s_ring[id % EVENT_LOG_LEN].id = id;
    s_ring[id % EVENT_LOG_LEN].t_s = (uint32_t)(esp_timer_get_time() / 1000000LL);
    strlcpy(s_ring[id % EVENT_LOG_LEN].text, text, EVENT_TEXT_MAX);
    xSemaphoreGive(s_lock);
}

void event_log_write(jw_t *w, uint32_t after)
{
    // Copy under the lock, write after it (the writer may be sending to a socket).
    static char copy[EVENT_LOG_LEN][EVENT_TEXT_MAX];
    static uint32_t ids[EVENT_LOG_LEN], ts[EVENT_LOG_LEN];
    xSemaphoreTake(s_copy_lock, portMAX_DELAY);
    int n = 0;
    xSemaphoreTake(s_lock, portMAX_DELAY);
    uint32_t first = s_next > EVENT_LOG_LEN ? s_next - EVENT_LOG_LEN : 1;
    if (first <= after) {
        first = after + 1;
    }
    for (uint32_t id = first; id < s_next; id++) {
        ids[n] = id;
        ts[n] = s_ring[id % EVENT_LOG_LEN].t_s;
        strlcpy(copy[n], s_ring[id % EVENT_LOG_LEN].text, EVENT_TEXT_MAX);
        n++;
    }
    xSemaphoreGive(s_lock);

    jw_raw(w, "[");
    for (int i = 0; i < n; i++) {
        jw_raw(w, i ? ",{\"id\":" : "{\"id\":");
        jw_num(w, ids[i]);
        jw_key(w, false, "t");
        jw_num(w, ts[i]);
        jw_key(w, false, "text");
        jw_str(w, copy[i]);
        jw_raw(w, "}");
    }
    jw_raw(w, "]");
    xSemaphoreGive(s_copy_lock);
}
