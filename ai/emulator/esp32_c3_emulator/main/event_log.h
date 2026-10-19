// In-memory logs for the panel: the firmware's own log lines, the command log and the MQTT
// traffic the emulator sees (for the route diagram). Fixed-size rings allocated at boot.
// Every entry has a sequence number (starting at 1), so the panel fetches only new entries.
#pragma once

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#define EVLOG_LINE_MAX 112
#define EVLOG_CMD_MSG_MAX 120
#define EVLOG_TOPIC_MAX 48

typedef struct {
    uint32_t seq;
    uint32_t t_ms;  // since boot
    char text[EVLOG_LINE_MAX];
} evlog_line_t;

typedef struct {
    uint32_t seq;
    uint32_t t_ms;
    bool ok;
    char source[8];  // "web", "mqtt"
    char msg[EVLOG_CMD_MSG_MAX];
} evlog_cmd_t;

// One message the emulator sent or received. "out" = emulator -> broker.
typedef enum {
    ROUTE_SENSORS,      // out: intellithings/<node>/sensors
    ROUTE_STATUS,       // out: intellithings/<node>/status, intellithings/emulator/status
    ROUTE_STATE,        // out: intellithings/emulator/state
    ROUTE_DISPLAY_OUT,  // out: test AI message to intellithings/<node>/display
    ROUTE_DISPLAY_IN,   // in:  intellithings/<node>/display
    ROUTE_CMD_MQTT,     // in:  intellithings/emulator/cmd
    ROUTE_CMD_HTTP,     // in:  POST /api/cmd from the panel or a script
    ROUTE_AI_IN,        // in:  intellithings/ai/summary, intellithings/ai/inference_status
    ROUTE_DEMO,         // out: intellithings/emulator/demo/#
    ROUTE_KIND_COUNT
} route_kind_t;

typedef struct {
    uint32_t seq;
    uint32_t t_ms;
    uint8_t kind;  // route_kind_t
    uint16_t bytes;
    char topic[EVLOG_TOPIC_MAX];
} evlog_route_t;

// Installs the log hook (lines still go to serial). Call first thing in app_main.
void event_log_init(void);

void event_log_cmd(const char *source, bool ok, const char *msg);
void event_log_route(route_kind_t kind, const char *topic, size_t bytes);
const char *route_kind_name(route_kind_t kind);

// Copy up to max entries with seq > after, oldest first; returns how many were copied.
// *gap is set when entries after `after` were already overwritten. Safe from any task.
size_t event_log_lines(uint32_t after, evlog_line_t *out, size_t max, bool *gap);
size_t event_log_cmds(uint32_t after, evlog_cmd_t *out, size_t max, bool *gap);
size_t event_log_routes(uint32_t after, evlog_route_t *out, size_t max, bool *gap);
