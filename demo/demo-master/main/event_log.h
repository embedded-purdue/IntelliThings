// Recent events for the admin page's overview (button presses, rolls, scenario steps, MQTT and
// Wi-Fi changes, admin edits). A small ring in RAM; each event is also logged on serial.
#pragma once

#include <stdbool.h>
#include <stdint.h>

#include "json_writer.h"

#define EVENT_LOG_LEN 40
#define EVENT_TEXT_MAX 72

void event_log_init(void);
void event_log(const char *fmt, ...) __attribute__((format(printf, 1, 2)));
// [{"id":n,"t":uptime_s,"text":"..."}, ...] oldest first, only ids above `after`.
void event_log_write(jw_t *w, uint32_t after);
