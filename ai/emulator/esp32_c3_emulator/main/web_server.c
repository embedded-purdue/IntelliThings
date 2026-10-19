#include "web_server.h"

#include <math.h>
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#include "ai_message.h"
#include "app_config.h"
#include "cJSON.h"
#include "emulator_ctrl.h"
#include "esp_http_server.h"
#include "esp_log.h"
#include "esp_system.h"
#include "esp_timer.h"
#include "event_log.h"
#include "history.h"
#include "mqtt_node.h"
#include "time_sync.h"
#include "virtual_node.h"

static const char *TAG = "web";

#define BODY_MAX 2048

// Gzipped at build time (main/CMakeLists.txt); every browser accepts gzip.
extern const char index_html_gz_start[] asm("_binary_index_html_gz_start");
extern const char index_html_gz_end[] asm("_binary_index_html_gz_end");
extern const char guide_md_gz_start[] asm("_binary_GUIDE_md_gz_start");
extern const char guide_md_gz_end[] asm("_binary_GUIDE_md_gz_end");

// ---------------------------------------------------------------------------
// Helpers

static esp_err_t send_json(httpd_req_t *req, char *json)
{
    if (!json) {
        httpd_resp_send_err(req, HTTPD_500_INTERNAL_SERVER_ERROR, "out of memory");
        return ESP_FAIL;
    }
    httpd_resp_set_type(req, "application/json");
    httpd_resp_set_hdr(req, "Cache-Control", "no-store");
    esp_err_t err = httpd_resp_sendstr(req, json);
    cJSON_free(json);
    return err;
}

static esp_err_t send_result(httpd_req_t *req, bool ok, const char *msg, cJSON *extra)
{
    cJSON *res = extra ? extra : cJSON_CreateObject();
    cJSON_AddBoolToObject(res, "ok", ok);
    cJSON_AddStringToObject(res, "msg", msg);
    char *json = cJSON_PrintUnformatted(res);
    cJSON_Delete(res);
    if (!ok) {
        httpd_resp_set_status(req, "400 Bad Request");
    }
    return send_json(req, json);
}

// Reads a JSON body of 1..max bytes. NULL (and the error already sent) on failure.
static char *read_body(httpd_req_t *req, size_t max, size_t *len)
{
    if (req->content_len == 0 || req->content_len > max) {
        char msg[48];
        snprintf(msg, sizeof(msg), "body must be 1..%u bytes of JSON", (unsigned)max);
        httpd_resp_send_err(req, HTTPD_400_BAD_REQUEST, msg);
        return NULL;
    }
    char *body = malloc(req->content_len + 1);
    if (!body) {
        httpd_resp_send_err(req, HTTPD_500_INTERNAL_SERVER_ERROR, "out of memory");
        return NULL;
    }
    size_t got = 0;
    while (got < req->content_len) {
        int r = httpd_req_recv(req, body + got, req->content_len - got);
        if (r == HTTPD_SOCK_ERR_TIMEOUT) {
            continue;
        }
        if (r <= 0) {
            free(body);
            return NULL;
        }
        got += r;
    }
    body[got] = '\0';
    *len = got;
    return body;
}

static bool query_param(httpd_req_t *req, const char *key, char *val, size_t len)
{
    char q[128];
    return httpd_req_get_url_query_str(req, q, sizeof(q)) == ESP_OK &&
           httpd_query_key_value(q, key, val, len) == ESP_OK;
}

static uint32_t query_u32(httpd_req_t *req, const char *key)
{
    char v[16];
    return query_param(req, key, v, sizeof(v)) ? (uint32_t)strtoul(v, NULL, 10) : 0;
}

// Chunked JSON writer: keeps big responses (poll) out of a cJSON tree and off the heap.
typedef struct {
    httpd_req_t *req;
    esp_err_t err;
    size_t n;
    char buf[768];
} jw_t;

static void jw_flush(jw_t *w)
{
    if (w->n && w->err == ESP_OK) {
        w->err = httpd_resp_send_chunk(w->req, w->buf, w->n);
    }
    w->n = 0;
}

static void jw_mem(jw_t *w, const char *s, size_t len)
{
    while (len) {
        if (w->n == sizeof(w->buf)) {
            jw_flush(w);
        }
        size_t k = sizeof(w->buf) - w->n;
        k = len < k ? len : k;
        memcpy(w->buf + w->n, s, k);
        w->n += k;
        s += k;
        len -= k;
    }
}

static void jw_raw(jw_t *w, const char *s)
{
    jw_mem(w, s, strlen(s));
}

static void jw_printf(jw_t *w, const char *fmt, ...)
{
    char tmp[128];
    va_list ap;
    va_start(ap, fmt);
    int n = vsnprintf(tmp, sizeof(tmp), fmt, ap);
    va_end(ap);
    if (n > 0) {
        jw_mem(w, tmp, (size_t)n < sizeof(tmp) ? (size_t)n : sizeof(tmp) - 1);
    }
}

static void jw_str(jw_t *w, const char *s)
{
    jw_mem(w, "\"", 1);
    const char *run = s;
    for (; *s; s++) {
        unsigned char c = (unsigned char)*s;
        if (c == '"' || c == '\\' || c < 0x20) {
            jw_mem(w, run, s - run);
            char esc[8];
            if (c == '"' || c == '\\') {
                esc[0] = '\\';
                esc[1] = (char)c;
                esc[2] = '\0';
            } else {
                snprintf(esc, sizeof(esc), "\\u%04x", c);
            }
            jw_raw(w, esc);
            run = s + 1;
        }
    }
    jw_mem(w, run, s - run);
    jw_mem(w, "\"", 1);
}

static const char *jbool(bool b)
{
    return b ? "true" : "false";
}

// ---------------------------------------------------------------------------
// Static pages

static esp_err_t index_get(httpd_req_t *req)
{
    httpd_resp_set_type(req, "text/html; charset=utf-8");
    httpd_resp_set_hdr(req, "Content-Encoding", "gzip");
    return httpd_resp_send(req, index_html_gz_start, index_html_gz_end - index_html_gz_start);
}

// GET /guide.md (the panel's guide popup), /guide.md?download=1 (saves the file)
static esp_err_t guide_get(httpd_req_t *req)
{
    char v[4];
    httpd_resp_set_type(req, "text/markdown; charset=utf-8");
    httpd_resp_set_hdr(req, "Content-Encoding", "gzip");
    if (query_param(req, "download", v, sizeof(v))) {
        httpd_resp_set_hdr(req, "Content-Disposition",
                           "attachment; filename=\"IntelliThings-emulator-guide.md\"");
    }
    return httpd_resp_send(req, guide_md_gz_start, guide_md_gz_end - guide_md_gz_start);
}

// ---------------------------------------------------------------------------
// v0.1.0 API (unchanged)

static esp_err_t meta_get(httpd_req_t *req)
{
    return send_json(req, ctrl_meta_json());
}

static esp_err_t state_get(httpd_req_t *req)
{
    return send_json(req, ctrl_state_json());
}

static esp_err_t messages_get(httpd_req_t *req)
{
    return send_json(req, ai_message_list_json());
}

static esp_err_t cmd_post(httpd_req_t *req)
{
    size_t len;
    char *body = read_body(req, BODY_MAX, &len);
    if (!body) {
        return ESP_FAIL;
    }
    char msg[160];
    bool ok = ctrl_execute(body, len, "web", msg, sizeof(msg));
    free(body);
    return send_result(req, ok, msg, NULL);
}

// ---------------------------------------------------------------------------
// GET /api/poll?log=&cmd=&ev=&msg= : state + everything new since the given sequence numbers

#define BATCH 4

static void write_lines(jw_t *w, uint32_t after)
{
    evlog_line_t b[BATCH];
    bool gap = false, first = true;
    jw_raw(w, "{\"items\":[");
    for (;;) {
        bool g;
        size_t n = event_log_lines(after, b, BATCH, &g);
        gap = gap || g;
        for (size_t i = 0; i < n; i++, first = false) {
            jw_printf(w, "%s{\"seq\":%lu,\"t_ms\":%lu,\"text\":", first ? "" : ",",
                      (unsigned long)b[i].seq, (unsigned long)b[i].t_ms);
            jw_str(w, b[i].text);
            jw_raw(w, "}");
            after = b[i].seq;
        }
        if (n < BATCH) {
            break;
        }
    }
    jw_printf(w, "],\"next\":%lu,\"gap\":%s}", (unsigned long)after, jbool(gap));
}

static void write_cmds(jw_t *w, uint32_t after)
{
    evlog_cmd_t b[BATCH];
    bool gap = false, first = true;
    jw_raw(w, "{\"items\":[");
    for (;;) {
        bool g;
        size_t n = event_log_cmds(after, b, BATCH, &g);
        gap = gap || g;
        for (size_t i = 0; i < n; i++, first = false) {
            jw_printf(w, "%s{\"seq\":%lu,\"t_ms\":%lu,\"ok\":%s,\"source\":", first ? "" : ",",
                      (unsigned long)b[i].seq, (unsigned long)b[i].t_ms, jbool(b[i].ok));
            jw_str(w, b[i].source);
            jw_raw(w, ",\"msg\":");
            jw_str(w, b[i].msg);
            jw_raw(w, "}");
            after = b[i].seq;
        }
        if (n < BATCH) {
            break;
        }
    }
    jw_printf(w, "],\"next\":%lu,\"gap\":%s}", (unsigned long)after, jbool(gap));
}

static void write_routes(jw_t *w, uint32_t after)
{
    evlog_route_t b[BATCH];
    bool gap = false, first = true;
    jw_raw(w, "{\"items\":[");
    for (;;) {
        bool g;
        size_t n = event_log_routes(after, b, BATCH, &g);
        gap = gap || g;
        for (size_t i = 0; i < n; i++, first = false) {
            jw_printf(w, "%s{\"seq\":%lu,\"t_ms\":%lu,\"kind\":\"%s\",\"bytes\":%u,\"topic\":",
                      first ? "" : ",", (unsigned long)b[i].seq, (unsigned long)b[i].t_ms,
                      route_kind_name((route_kind_t)b[i].kind), (unsigned)b[i].bytes);
            jw_str(w, b[i].topic);
            jw_raw(w, "}");
            after = b[i].seq;
        }
        if (n < BATCH) {
            break;
        }
    }
    jw_printf(w, "],\"next\":%lu,\"gap\":%s}", (unsigned long)after, jbool(gap));
}

static void write_msg(jw_t *w, const ai_msg_t *m, int64_t now)
{
    jw_printf(w, "{\"seq\":%lu,\"node_id\":", (unsigned long)m->seq);
    jw_str(w, m->node_id);
    jw_raw(w, ",\"room\":");
    jw_str(w, m->label);
    jw_raw(w, ",\"text\":");
    jw_str(w, m->text);
    jw_raw(w, ",\"ts\":");
    jw_str(w, m->ts);
    jw_raw(w, ",\"received\":");
    jw_str(w, m->received);
    jw_printf(w, ",\"age_s\":%lld,\"retained\":%s,\"raw\":%s,\"truncated\":%s}",
              (long long)((now - m->received_us) / 1000000LL), jbool(m->retained),
              jbool(m->raw), jbool(m->truncated));
}

static void write_messages(jw_t *w, uint32_t after, int64_t now)
{
    ai_msg_t m;  // ~560 bytes; one at a time
    bool gap = false, first = true;
    jw_raw(w, "{\"items\":[");
    for (;;) {
        bool g;
        if (ai_message_copy(after, &m, 1, &g) == 0) {
            gap = gap || g;
            break;
        }
        gap = gap || g;
        jw_raw(w, first ? "" : ",");
        first = false;
        write_msg(w, &m, now);
        after = m.seq;
    }
    jw_printf(w, "],\"next\":%lu,\"gap\":%s}", (unsigned long)after, jbool(gap));
}

static void write_ai_topic(jw_t *w, const char *name, const char *field, ai_topic_t which,
                           int64_t now)
{
    ai_topic_msg_t t;
    ai_message_ai_topic(which, &t);
    jw_printf(w, ",\"%s\":", name);
    if (!t.valid) {
        jw_raw(w, "null");
        return;
    }
    jw_printf(w, "{\"%s\":", field);
    jw_str(w, t.text);
    jw_raw(w, ",\"ts\":");
    jw_str(w, t.ts);
    jw_printf(w, ",\"age_s\":%lld,\"retained\":%s,\"raw\":%s}",
              (long long)((now - t.received_us) / 1000000LL), jbool(t.retained), jbool(t.raw));
}

static esp_err_t poll_get(httpd_req_t *req)
{
    uint32_t a_log = query_u32(req, "log"), a_cmd = query_u32(req, "cmd");
    uint32_t a_ev = query_u32(req, "ev"), a_msg = query_u32(req, "msg");
    char *state = ctrl_state_json();
    if (!state) {
        httpd_resp_send_err(req, HTTPD_500_INTERNAL_SERVER_ERROR, "out of memory");
        return ESP_FAIL;
    }
    httpd_resp_set_type(req, "application/json");
    httpd_resp_set_hdr(req, "Cache-Control", "no-store");

    static jw_t w;  // 780 bytes; httpd runs one handler at a time
    w.req = req;
    w.err = ESP_OK;
    w.n = 0;
    int64_t now = esp_timer_get_time();

    jw_raw(&w, "{\"state\":");
    jw_raw(&w, state);
    cJSON_free(state);
    jw_raw(&w, ",\"log\":");
    write_lines(&w, a_log);
    jw_raw(&w, ",\"cmds\":");
    write_cmds(&w, a_cmd);
    jw_raw(&w, ",\"routes\":");
    write_routes(&w, a_ev);
    jw_raw(&w, ",\"messages\":");
    write_messages(&w, a_msg, now);

    jw_raw(&w, ",\"ai\":{\"latest\":[");
    for (int i = 0; i < NODE_COUNT; i++) {
        ai_msg_t m;
        jw_raw(&w, i ? "," : "");
        if (ai_message_latest(i, &m)) {
            write_msg(&w, &m, now);
        } else {
            jw_raw(&w, "null");
        }
    }
    jw_raw(&w, "]");
    write_ai_topic(&w, "summary", "text", AI_TOPIC_SUMMARY, now);
    write_ai_topic(&w, "inference", "state", AI_TOPIC_INFERENCE, now);
    jw_raw(&w, "}}");
    jw_flush(&w);
    if (w.err != ESP_OK) {
        return w.err;
    }
    return httpd_resp_send_chunk(req, NULL, 0);
}

// ---------------------------------------------------------------------------
// GET /api/history?tier=10m|1h : binary, little-endian (layout in ai/emulator/GUIDE.md)

static void put16(uint8_t *p, uint16_t v)
{
    p[0] = (uint8_t)v;
    p[1] = (uint8_t)(v >> 8);
}

static void put32(uint8_t *p, uint32_t v)
{
    put16(p, (uint16_t)v);
    put16(p + 2, (uint16_t)(v >> 16));
}

static esp_err_t history_get(httpd_req_t *req)
{
    char t[8] = "10m";
    query_param(req, "tier", t, sizeof(t));
    hist_tier_id_t tier;
    if (strcmp(t, "10m") == 0) {
        tier = HIST_FINE;
    } else if (strcmp(t, "1h") == 0) {
        tier = HIST_COARSE;
    } else {
        httpd_resp_send_err(req, HTTPD_400_BAD_REQUEST, "tier must be 10m or 1h");
        return ESP_FAIL;
    }

    nodes_lock();
    hist_view_t v = history_view(tier);
    nodes_unlock();

    uint32_t interval = tier == HIST_FINE ? (app_config_publish_interval_ms() + 500) / 1000
                                          : history_bucket_s();
    uint8_t hdr[24 + 2 * HIST_SERIES];
    memcpy(hdr, "EMUH", 4);
    hdr[4] = 1;  // format version
    hdr[5] = (uint8_t)tier;
    hdr[6] = HIST_NODES;
    hdr[7] = HIST_SERIES;
    put16(hdr + 8, v.count);
    put16(hdr + 10, (uint16_t)interval);
    put32(hdr + 12, (uint32_t)(esp_timer_get_time() / 1000000LL));
    put32(hdr + 16, time_is_synced() ? (uint32_t)time(NULL) : 0);
    put32(hdr + 20, 0);
    for (int s = 0; s < HIST_SERIES; s++) {
        put16(hdr + 24 + 2 * s, history_scale(s));
    }

    httpd_resp_set_type(req, "application/octet-stream");
    httpd_resp_set_hdr(req, "Cache-Control", "no-store");
    esp_err_t err = httpd_resp_send_chunk(req, (const char *)hdr, sizeof(hdr));

    // RISC-V is little-endian, so the copied arrays go out as they are.
    enum { PIECE = 64 };
    uint32_t times[PIECE];
    for (uint16_t pos = 0; err == ESP_OK && pos < v.count; pos += PIECE) {
        uint16_t n = v.count - pos < PIECE ? v.count - pos : PIECE;
        nodes_lock();
        history_copy_times(tier, v, pos, n, times);
        nodes_unlock();
        err = httpd_resp_send_chunk(req, (const char *)times, n * sizeof(uint32_t));
    }
    uint16_t vals[PIECE];
    for (int node = 0; node < HIST_NODES; node++) {
        for (int s = 0; s < HIST_SERIES; s++) {
            for (uint16_t pos = 0; err == ESP_OK && pos < v.count; pos += PIECE) {
                uint16_t n = v.count - pos < PIECE ? v.count - pos : PIECE;
                nodes_lock();
                history_copy_series(tier, v, node, s, pos, n, vals);
                nodes_unlock();
                err = httpd_resp_send_chunk(req, (const char *)vals, n * sizeof(uint16_t));
            }
        }
    }
    if (err != ESP_OK) {
        return err;
    }
    return httpd_resp_send_chunk(req, NULL, 0);
}

// ---------------------------------------------------------------------------
// /api/config : MQTT broker and publish interval (password is write-only)

static void add_config(cJSON *o, const app_config_t *c)
{
    cJSON_AddStringToObject(o, "host", c->host);
    cJSON_AddNumberToObject(o, "port", c->port);
    cJSON_AddStringToObject(o, "username", c->username);
    cJSON_AddBoolToObject(o, "password_set", c->password[0] != '\0');
    cJSON_AddNumberToObject(o, "keepalive_s", c->keepalive_s);
    cJSON_AddNumberToObject(o, "publish_interval_ms", c->publish_interval_ms);
}

static esp_err_t config_get(httpd_req_t *req)
{
    app_config_t c, d;
    app_config_get(&c);
    app_config_defaults(&d);
    cJSON *root = cJSON_CreateObject();
    add_config(root, &c);
    cJSON_AddBoolToObject(root, "saved", app_config_saved());
    add_config(cJSON_AddObjectToObject(root, "defaults"), &d);
    char *json = cJSON_PrintUnformatted(root);
    cJSON_Delete(root);
    return send_json(req, json);
}

static void restart_cb(void *arg)
{
    esp_restart();
}

static void schedule_restart(void)
{
    static esp_timer_handle_t t;
    if (!t) {
        const esp_timer_create_args_t args = {.callback = restart_cb, .name = "restart"};
        esp_timer_create(&args, &t);
    }
    esp_timer_start_once(t, 800 * 1000);  // let the HTTP response go out first
}

// Applies a settings change: reconnect MQTT, or reboot when a login is cleared (esp-mqtt
// can't drop a username/password from a running client). Writes the outcome into msg.
static void apply_config(const app_config_t *old, const app_config_t *now, cJSON *res, char *msg,
                         size_t len)
{
    bool cleared = (old->username[0] && !now->username[0]) ||
                   (old->password[0] && !now->password[0]);
    bool mqtt = app_config_mqtt_differs(old, now);
    bool reboot = mqtt && cleared;
    size_t n = strlen(msg);
    snprintf(msg + n, len - n, "; broker mqtt://%s:%u%s, publish every %lu ms%s", now->host,
             (unsigned)now->port, now->username[0] ? " (login set)" : " (anonymous)",
             (unsigned long)now->publish_interval_ms,
             reboot ? "; rebooting to apply" : mqtt ? "; reconnecting" : "");
    cJSON_AddBoolToObject(res, "reconnecting", mqtt && !reboot);
    cJSON_AddBoolToObject(res, "rebooting", reboot);
    if (reboot) {
        schedule_restart();
    } else if (mqtt) {
        mqtt_reconfigure_async();
    }
}

static bool get_str_field(const cJSON *root, const char *key, char *dst, size_t size, char *msg,
                          size_t len)
{
    const cJSON *it = cJSON_GetObjectItemCaseSensitive(root, key);
    if (!it) {
        return true;
    }
    if (!cJSON_IsString(it) || strlen(it->valuestring) >= size) {
        snprintf(msg, len, "%s must be a string of at most %u characters", key,
                 (unsigned)size - 1);
        return false;
    }
    strlcpy(dst, it->valuestring, size);
    return true;
}

static bool get_num_field(const cJSON *root, const char *key, double lo, double hi, double *out,
                          char *msg, size_t len)
{
    const cJSON *it = cJSON_GetObjectItemCaseSensitive(root, key);
    if (!it) {
        return true;
    }
    if (!cJSON_IsNumber(it) || it->valuedouble < lo || it->valuedouble > hi ||
        it->valuedouble != floor(it->valuedouble)) {
        snprintf(msg, len, "%s must be a whole number %.0f..%.0f", key, lo, hi);
        return false;
    }
    *out = it->valuedouble;
    return true;
}

static esp_err_t config_post(httpd_req_t *req)
{
    size_t blen;
    char *body = read_body(req, 1024, &blen);
    if (!body) {
        return ESP_FAIL;
    }
    char msg[192] = "";
    bool ok = false;
    app_config_t old, c;
    app_config_get(&old);
    c = old;
    cJSON *res = cJSON_CreateObject();
    cJSON *root = cJSON_ParseWithLength(body, blen);
    free(body);

    static const char *const known[] = {"host", "port", "username", "password", "keepalive_s",
                                        "publish_interval_ms"};
    if (!cJSON_IsObject(root)) {
        snprintf(msg, sizeof(msg), "invalid JSON");
        goto done;
    }
    const cJSON *it;
    cJSON_ArrayForEach(it, root) {
        bool found = false;
        for (size_t i = 0; i < sizeof(known) / sizeof(known[0]); i++) {
            found = found || strcmp(it->string, known[i]) == 0;
        }
        if (!found) {
            snprintf(msg, sizeof(msg), "unknown setting \"%.32s\"", it->string);
            goto done;
        }
    }
    double port = c.port, keep = c.keepalive_s, pub = c.publish_interval_ms;
    if (!get_str_field(root, "host", c.host, sizeof(c.host), msg, sizeof(msg)) ||
        !get_str_field(root, "username", c.username, sizeof(c.username), msg, sizeof(msg)) ||
        !get_str_field(root, "password", c.password, sizeof(c.password), msg, sizeof(msg)) ||
        !get_num_field(root, "port", 1, 65535, &port, msg, sizeof(msg)) ||
        !get_num_field(root, "keepalive_s", 0, 65535, &keep, msg, sizeof(msg)) ||
        !get_num_field(root, "publish_interval_ms", 0, 4294967295.0, &pub, msg, sizeof(msg))) {
        goto done;
    }
    c.port = (uint16_t)port;
    c.keepalive_s = (uint16_t)keep;
    c.publish_interval_ms = (uint32_t)pub;
    if (!app_config_save(&c, msg, sizeof(msg))) {
        goto done;
    }
    snprintf(msg, sizeof(msg), "settings saved");
    apply_config(&old, &c, res, msg, sizeof(msg));
    ok = true;

done:
    cJSON_Delete(root);
    ctrl_record("web", ok, msg);
    return send_result(req, ok, msg, res);
}

static esp_err_t config_reset_post(httpd_req_t *req)
{
    char msg[192];
    app_config_t old, c;
    app_config_get(&old);
    cJSON *res = cJSON_CreateObject();
    bool ok = app_config_reset(msg, sizeof(msg));
    if (ok) {
        app_config_get(&c);
        snprintf(msg, sizeof(msg), "settings reset to defaults");
        apply_config(&old, &c, res, msg, sizeof(msg));
    }
    ctrl_record("web", ok, msg);
    return send_result(req, ok, msg, res);
}

// ---------------------------------------------------------------------------

void web_server_start(void)
{
    static httpd_handle_t server;
    if (server) {
        return;
    }
    httpd_config_t cfg = HTTPD_DEFAULT_CONFIG();
    cfg.stack_size = 8192;  // cJSON state document
    cfg.lru_purge_enable = true;
    cfg.max_uri_handlers = 16;
    if (httpd_start(&server, &cfg) != ESP_OK) {
        ESP_LOGE(TAG, "Failed to start the web server");
        return;
    }
    const httpd_uri_t uris[] = {
        {.uri = "/", .method = HTTP_GET, .handler = index_get},
        {.uri = "/guide.md", .method = HTTP_GET, .handler = guide_get},
        {.uri = "/api/meta", .method = HTTP_GET, .handler = meta_get},
        {.uri = "/api/state", .method = HTTP_GET, .handler = state_get},
        {.uri = "/api/messages", .method = HTTP_GET, .handler = messages_get},
        {.uri = "/api/cmd", .method = HTTP_POST, .handler = cmd_post},
        {.uri = "/api/poll", .method = HTTP_GET, .handler = poll_get},
        {.uri = "/api/history", .method = HTTP_GET, .handler = history_get},
        {.uri = "/api/config", .method = HTTP_GET, .handler = config_get},
        {.uri = "/api/config", .method = HTTP_POST, .handler = config_post},
        {.uri = "/api/config/reset", .method = HTTP_POST, .handler = config_reset_post},
    };
    for (size_t i = 0; i < sizeof(uris) / sizeof(uris[0]); i++) {
        httpd_register_uri_handler(server, &uris[i]);
    }
    ESP_LOGI(TAG, "Control panel running on port 80");
}
