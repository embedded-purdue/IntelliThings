#include "event_log.h"

#include <stdarg.h>
#include <stdio.h>
#include <string.h>

#include "esp_log.h"
#include "esp_timer.h"
#include "freertos/FreeRTOS.h"

#define LINES_CAP 80
#define CMDS_CAP 32
#define ROUTES_CAP 64

// Each element starts with its uint32_t seq. Slot of seq s is (s - 1) % cap.
typedef struct {
    uint8_t *buf;
    size_t elem;
    size_t cap;
    uint32_t next_seq;
} ring_t;

static evlog_line_t s_lines[LINES_CAP];
static evlog_cmd_t s_cmds[CMDS_CAP];
static evlog_route_t s_routes[ROUTES_CAP];

static ring_t s_line_ring = {(uint8_t *)s_lines, sizeof(evlog_line_t), LINES_CAP, 1};
static ring_t s_cmd_ring = {(uint8_t *)s_cmds, sizeof(evlog_cmd_t), CMDS_CAP, 1};
static ring_t s_route_ring = {(uint8_t *)s_routes, sizeof(evlog_route_t), ROUTES_CAP, 1};

// The log hook can run in any task, so the rings use a spinlock and only ever memcpy inside it.
static portMUX_TYPE s_mux = portMUX_INITIALIZER_UNLOCKED;
static vprintf_like_t s_prev_vprintf;

static uint32_t now_ms(void)
{
    return (uint32_t)(esp_timer_get_time() / 1000);
}

static void ring_push(ring_t *r, void *elem)
{
    portENTER_CRITICAL_SAFE(&s_mux);
    uint32_t seq = r->next_seq++;
    *(uint32_t *)elem = seq;
    memcpy(r->buf + ((seq - 1) % r->cap) * r->elem, elem, r->elem);
    portEXIT_CRITICAL_SAFE(&s_mux);
}

static size_t ring_copy(ring_t *r, uint32_t after, void *out, size_t max, bool *gap)
{
    size_t n = 0;
    portENTER_CRITICAL_SAFE(&s_mux);
    uint32_t newest = r->next_seq - 1;
    uint32_t oldest = newest >= r->cap ? newest - r->cap + 1 : 1;
    if (after > newest) {
        after = 0;  // a seq from before a reboot: start over
    }
    uint32_t from = after + 1;
    *gap = from < oldest && newest > 0;
    if (from < oldest) {
        from = oldest;
    }
    for (uint32_t seq = from; seq <= newest && n < max; seq++, n++) {
        memcpy((uint8_t *)out + n * r->elem, r->buf + ((seq - 1) % r->cap) * r->elem, r->elem);
    }
    portEXIT_CRITICAL_SAFE(&s_mux);
    return n;
}

// ---------------------------------------------------------------------------
// Log hook

static int log_vprintf(const char *fmt, va_list ap)
{
    va_list copy;
    va_copy(copy, ap);
    int ret = s_prev_vprintf ? s_prev_vprintf(fmt, ap) : vprintf(fmt, ap);

    evlog_line_t e;
    int len = vsnprintf(e.text, sizeof(e.text), fmt, copy);
    va_end(copy);
    if (len <= 0) {
        return ret;
    }
    // Clean up for the panel: drop ANSI colours, turn inner newlines into spaces, trim the end.
    size_t w = 0;
    for (size_t i = 0; e.text[i]; i++) {
        char c = e.text[i];
        if (c == '\033') {
            while (e.text[i] && e.text[i] != 'm') {
                i++;
            }
            if (!e.text[i]) {
                break;
            }
            continue;
        }
        e.text[w++] = (c == '\n' || c == '\r' || c == '\t') ? ' ' : c;
    }
    while (w > 0 && e.text[w - 1] == ' ') {
        w--;
    }
    e.text[w] = '\0';
    // The per-publish reading lines ("node: ...") would flush the ring every minute; the room
    // cards already show them.
    if (w == 0 || strstr(e.text, ") node: ") != NULL) {
        return ret;
    }
    e.t_ms = now_ms();
    ring_push(&s_line_ring, &e);
    return ret;
}

void event_log_init(void)
{
    s_prev_vprintf = esp_log_set_vprintf(log_vprintf);
}

// ---------------------------------------------------------------------------
// Commands and MQTT traffic

void event_log_cmd(const char *source, bool ok, const char *msg)
{
    evlog_cmd_t e = {0};
    e.t_ms = now_ms();
    e.ok = ok;
    strlcpy(e.source, source, sizeof(e.source));
    strlcpy(e.msg, msg, sizeof(e.msg));
    ring_push(&s_cmd_ring, &e);
}

void event_log_route(route_kind_t kind, const char *topic, size_t bytes)
{
    evlog_route_t e = {0};
    e.t_ms = now_ms();
    e.kind = (uint8_t)kind;
    e.bytes = bytes > 0xFFFF ? 0xFFFF : (uint16_t)bytes;
    strlcpy(e.topic, topic ? topic : "", sizeof(e.topic));
    ring_push(&s_route_ring, &e);
}

const char *route_kind_name(route_kind_t kind)
{
    static const char *const names[ROUTE_KIND_COUNT] = {
        [ROUTE_SENSORS] = "sensors",         [ROUTE_STATUS] = "status",
        [ROUTE_STATE] = "state",             [ROUTE_DISPLAY_OUT] = "display_out",
        [ROUTE_DISPLAY_IN] = "display_in",   [ROUTE_CMD_MQTT] = "cmd_mqtt",
        [ROUTE_CMD_HTTP] = "cmd_http",       [ROUTE_AI_IN] = "ai_in",
    };
    return kind < ROUTE_KIND_COUNT ? names[kind] : "?";
}

size_t event_log_lines(uint32_t after, evlog_line_t *out, size_t max, bool *gap)
{
    return ring_copy(&s_line_ring, after, out, max, gap);
}

size_t event_log_cmds(uint32_t after, evlog_cmd_t *out, size_t max, bool *gap)
{
    return ring_copy(&s_cmd_ring, after, out, max, gap);
}

size_t event_log_routes(uint32_t after, evlog_route_t *out, size_t max, bool *gap)
{
    return ring_copy(&s_route_ring, after, out, max, gap);
}
