#include "web_server.h"

#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#include "ai_watch.h"
#include "button.h"
#include "cJSON.h"
#include "demo_home.h"
#include "engine.h"
#include "esp_app_desc.h"
#include "esp_http_server.h"
#include "esp_log.h"
#include "esp_system.h"
#include "esp_timer.h"
#include "event_log.h"
#include "freertos/FreeRTOS.h"
#include "freertos/task.h"
#include "json_writer.h"
#include "library.h"
#include "mqtt_link.h"
#include "settings.h"
#include "time_sync.h"
#include "wifi_manager.h"

static const char *TAG = "web";

#define BODY_MAX 4096

// Gzipped at build time (main/CMakeLists.txt).
extern const char demo_html_gz_start[] asm("_binary_demo_html_gz_start");
extern const char demo_html_gz_end[] asm("_binary_demo_html_gz_end");
extern const char admin_html_gz_start[] asm("_binary_admin_html_gz_start");
extern const char admin_html_gz_end[] asm("_binary_admin_html_gz_end");

// ---------------------------------------------------------------------------
// Helpers

static esp_err_t send_result(httpd_req_t *req, bool ok, const char *msg)
{
    cJSON *res = cJSON_CreateObject();
    cJSON_AddBoolToObject(res, "ok", ok);
    cJSON_AddStringToObject(res, "msg", msg);
    char *json = cJSON_PrintUnformatted(res);
    cJSON_Delete(res);
    if (!json) {
        return httpd_resp_send_err(req, HTTPD_500_INTERNAL_SERVER_ERROR, "out of memory");
    }
    if (!ok) {
        httpd_resp_set_status(req, "400 Bad Request");
    }
    httpd_resp_set_type(req, "application/json");
    httpd_resp_set_hdr(req, "Cache-Control", "no-store");
    esp_err_t err = httpd_resp_sendstr(req, json);
    cJSON_free(json);
    return err;
}

// Reads and parses a JSON body. NULL (and the error already sent) on failure.
static cJSON *read_json(httpd_req_t *req)
{
    if (req->content_len == 0 || req->content_len > BODY_MAX) {
        send_result(req, false, "the body must be 1..4096 bytes of JSON");
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
    cJSON *root = cJSON_Parse(body);
    free(body);
    if (!cJSON_IsObject(root)) {
        cJSON_Delete(root);
        send_result(req, false, "invalid JSON");
        return NULL;
    }
    return root;
}

static bool query_param(httpd_req_t *req, const char *key, char *val, size_t len)
{
    char q[96];
    return httpd_req_get_url_query_str(req, q, sizeof(q)) == ESP_OK &&
           httpd_query_key_value(q, key, val, len) == ESP_OK;
}

static uint32_t query_u32(httpd_req_t *req, const char *key)
{
    char v[16];
    return query_param(req, key, v, sizeof(v)) ? (uint32_t)strtoul(v, NULL, 10) : 0;
}

// Chunked JSON responses: big documents go out through a small buffer.
typedef struct {
    httpd_req_t *req;
    esp_err_t err;
} http_sink_t;

static bool http_sink(void *ctx, const char *data, size_t len)
{
    http_sink_t *s = ctx;
    s->err = httpd_resp_send_chunk(s->req, data, len);
    return s->err == ESP_OK;
}

typedef struct {
    jw_t w;
    http_sink_t sink;
    char buf[1024];
} stream_t;

static stream_t *stream_begin(httpd_req_t *req)
{
    stream_t *s = malloc(sizeof(stream_t));
    if (!s) {
        httpd_resp_send_err(req, HTTPD_500_INTERNAL_SERVER_ERROR, "out of memory");
        return NULL;
    }
    s->sink = (http_sink_t){.req = req, .err = ESP_OK};
    jw_init(&s->w, s->buf, sizeof(s->buf), http_sink, &s->sink);
    httpd_resp_set_type(req, "application/json");
    httpd_resp_set_hdr(req, "Cache-Control", "no-store");
    return s;
}

static esp_err_t stream_end(stream_t *s)
{
    bool ok = jw_flush(&s->w);
    esp_err_t err = ok ? httpd_resp_send_chunk(s->sink.req, NULL, 0) : s->sink.err;
    free(s);
    return err;
}

static bool admin_ok(httpd_req_t *req)
{
    char pin[SET_PIN_MAX + 2];
    if (httpd_req_get_hdr_value_str(req, "X-Admin-Pin", pin, sizeof(pin)) == ESP_OK &&
        settings_check_pin(pin)) {
        return true;
    }
    httpd_resp_set_status(req, "401 Unauthorized");
    httpd_resp_set_type(req, "application/json");
    httpd_resp_sendstr(req, "{\"ok\":false,\"msg\":\"wrong PIN\"}");
    return false;
}

// demo_fill_t, as in the JSON's fill_rules
static const char *const FILL_NAMES[] = {"normal", "presence", "light", "lock", "car", "off", "blinds"};
static const char *const KIND_NAMES[] = {"device", "sensor", "context"};  // demo_kind_t

static const char *type_name(int t)
{
    return t == DEMO_NUM ? "number" : t == DEMO_BOOL ? "bool" : "enum";
}

// ---------------------------------------------------------------------------
// Pages

static esp_err_t send_gz_html(httpd_req_t *req, const char *start, const char *end)
{
    httpd_resp_set_type(req, "text/html; charset=utf-8");
    httpd_resp_set_hdr(req, "Content-Encoding", "gzip");
    httpd_resp_set_hdr(req, "Cache-Control", "no-cache");
    return httpd_resp_send(req, start, end - start);
}

static esp_err_t h_demo(httpd_req_t *req)
{
    return send_gz_html(req, demo_html_gz_start, demo_html_gz_end);
}

static esp_err_t h_admin(httpd_req_t *req)
{
    return send_gz_html(req, admin_html_gz_start, admin_html_gz_end);
}

// ---------------------------------------------------------------------------
// Public API

static esp_err_t h_state(httpd_req_t *req)
{
    stream_t *s = stream_begin(req);
    if (!s) {
        return ESP_FAIL;
    }
    char since[16];
    // No "since" at all: always send the values (seq 0 is a real value after boot).
    uint32_t after = query_param(req, "since", since, sizeof(since)) ? query_u32(req, "since")
                                                                      : UINT32_MAX;
    engine_write_state(&s->w, after);
    return stream_end(s);
}

static void write_limit(jw_t *w, const char *name, const demo_limit_t *l, int dec)
{
    jw_raw(w, ",\"");
    jw_raw(w, name);
    jw_raw(w, "\":[");
    jw_fixed(w, l->min, dec);
    jw_raw(w, ",");
    jw_fixed(w, l->max, dec);
    jw_raw(w, ",");
    jw_fixed(w, l->nmin, dec);
    jw_raw(w, ",");
    jw_fixed(w, l->nmax, dec);
    jw_raw(w, "]");
}

static esp_err_t h_meta(httpd_req_t *req)
{
    stream_t *s = stream_begin(req);
    if (!s) {
        return ESP_FAIL;
    }
    jw_t *w = &s->w;
    jw_raw(w, "{\"fw\":");
    jw_str(w, esp_app_get_description()->version);
    jw_raw(w, ",\"areas\":[");
    for (int a = 0; a < DEMO_AREA_COUNT; a++) {
        jw_raw(w, a ? ",{\"key\":" : "{\"key\":");
        jw_str(w, DEMO_AREAS[a].key);
        jw_key(w, false, "label");
        jw_str(w, DEMO_AREAS[a].label);
        jw_key(w, false, "node");
        jw_raw(w, jbool(DEMO_AREAS[a].node_room[0] != '\0'));
        jw_raw(w, "}");
    }
    jw_raw(w, "],\"entities\":[");
    for (int e = 0; e < DEMO_ENTITY_COUNT; e++) {
        const demo_entity_t *d = &DEMO_ENTITIES[e];
        jw_raw(w, e ? ",{\"key\":" : "{\"key\":");
        jw_str(w, d->key);
        jw_key(w, false, "field");
        jw_str(w, d->field);
        jw_key(w, false, "label");
        jw_str(w, d->label);
        jw_key(w, false, "unit");
        jw_str(w, d->unit);
        jw_key(w, false, "area");
        jw_num(w, d->area);
        jw_key(w, false, "type");
        jw_str(w, type_name(d->type));
        jw_key(w, false, "dec");
        jw_num(w, d->dec);
        jw_key(w, false, "derived");
        jw_raw(w, jbool(d->derive != DERIVE_NONE));
        jw_key(w, false, "fill");
        jw_str(w, FILL_NAMES[d->fill]);
        jw_key(w, false, "kind");
        jw_str(w, KIND_NAMES[d->kind]);
        jw_key(w, false, "internal");
        jw_raw(w, jbool(d->internal));
        if (d->n_options) {
            jw_raw(w, ",\"options\":[");
            for (int i = 0; i < d->n_options; i++) {
                if (i) {
                    jw_raw(w, ",");
                }
                jw_str(w, d->options[i]);
            }
            jw_raw(w, "]");
        }
        if (d->type == DEMO_ENUM) {
            jw_key(w, false, "normal_mask");
            jw_num(w, d->normal_mask);
        }
        if (d->type != DEMO_ENUM) {
            demo_limit_t def = {d->min, d->max, d->nmin, d->nmax};
            int dec = d->type == DEMO_BOOL ? 2 : d->dec;
            write_limit(w, "limits", demo_limit(e), dec);
            write_limit(w, "default", &def, dec);
        }
        jw_raw(w, "}");
    }
    jw_raw(w, "],\"groups\":[");
    for (int g = 0; g < DEMO_GROUP_COUNT; g++) {
        jw_raw(w, g ? ",{\"key\":" : "{\"key\":");
        jw_str(w, DEMO_GROUPS[g]);
        jw_key(w, false, "label");
        jw_str(w, DEMO_GROUP_LABELS[g]);
        jw_raw(w, "}");
    }
    jw_raw(w, "],\"scenarios\":");
    library_write_list(w);
    jw_key(w, false, "lib");
    jw_num(w, library_version());
    jw_key(w, false, "factory");
    jw_raw(w, jbool(library_is_factory()));
    jw_key(w, false, "max_sets");
    jw_num(w, DEMO_MAX_SETS);
    jw_key(w, false, "max_scenarios");
    jw_num(w, LIB_MAX);
    jw_raw(w, "}");
    return stream_end(s);
}

static esp_err_t h_press(httpd_req_t *req)
{
    // Any body; {"src":"key"} counts it as a key press.
    press_src_t src = PRESS_WEB;
    char body[48] = "";
    int n = req->content_len > 0 && req->content_len < sizeof(body)
                ? httpd_req_recv(req, body, req->content_len)
                : 0;
    if (n > 0) {
        body[n] = '\0';
        if (strstr(body, "\"key\"")) {
            src = PRESS_KEY;
        }
    }
    int left = 0;
    press_result_t r = engine_press(src, &left);
    char json[96];
    snprintf(json, sizeof(json), "{\"ok\":%s,\"result\":\"%s\",\"lock_left_s\":%d}",
             r == PRESS_ROLLING ? "true" : "false",
             r == PRESS_ROLLING ? "rolling" : r == PRESS_BUSY ? "busy" : "locked", left);
    httpd_resp_set_type(req, "application/json");
    httpd_resp_set_hdr(req, "Cache-Control", "no-store");
    return httpd_resp_sendstr(req, json);
}

// ---------------------------------------------------------------------------
// Admin API

static esp_err_t h_login(httpd_req_t *req)
{
    if (!admin_ok(req)) {
        return ESP_OK;
    }
    return send_result(req, true, "welcome");
}

static esp_err_t h_status(httpd_req_t *req)
{
    if (!admin_ok(req)) {
        return ESP_OK;
    }
    stream_t *s = stream_begin(req);
    if (!s) {
        return ESP_FAIL;
    }
    jw_t *w = &s->w;
    dm_settings_t set;
    settings_get(&set);
    char ip[16], uri[96], err[64], ts[40];
    wifi_get_ip(ip, sizeof(ip));
    mqtt_link_uri(uri, sizeof(uri));
    mqtt_link_last_error(err, sizeof(err));

    jw_raw(w, "{\"fw\":");
    jw_str(w, esp_app_get_description()->version);
    jw_key(w, false, "idf");
    jw_str(w, esp_app_get_description()->idf_ver);
    jw_key(w, false, "uptime_s");
    jw_num(w, (double)(esp_timer_get_time() / 1000000LL));
    jw_key(w, false, "heap_free");
    jw_num(w, esp_get_free_heap_size());
    jw_key(w, false, "heap_min");
    jw_num(w, esp_get_minimum_free_heap_size());
    jw_key(w, false, "time");
    if (time_iso8601(ts, sizeof(ts))) {
        jw_str(w, ts);
    } else {
        jw_raw(w, "null");
    }
    jw_raw(w, ",\"wifi\":{\"connected\":");
    jw_raw(w, jbool(wifi_is_connected()));
    jw_key(w, false, "ip");
    jw_str(w, ip);
    jw_key(w, false, "rssi");
    jw_num(w, wifi_get_rssi());
    jw_key(w, false, "reconnects");
    jw_num(w, wifi_reconnect_count());
    jw_key(w, false, "hostname");
    jw_str(w, CONFIG_DM_HOSTNAME);
    jw_raw(w, "},\"mqtt\":{\"connected\":");
    jw_raw(w, jbool(mqtt_link_connected()));
    jw_key(w, false, "uri");
    jw_str(w, uri);
    jw_key(w, false, "sent");
    jw_num(w, mqtt_link_sent());
    jw_key(w, false, "reconnects");
    jw_num(w, mqtt_link_reconnects());
    jw_key(w, false, "error");
    jw_str(w, err);
    jw_raw(w, "},\"button\":{\"gpio\":");
    jw_num(w, CONFIG_DM_BUTTON_GPIO);
    jw_key(w, false, "held");
    jw_raw(w, jbool(button_is_held()));
    jw_key(w, false, "level");
    jw_raw(w, jbool(button_level()));
    jw_key(w, false, "held_ms");
    jw_num(w, button_held_ms());
    jw_key(w, false, "rejected");
    jw_num(w, button_rejected());
    jw_raw(w, "},\"engine\":");
    engine_write_admin(w);
    jw_raw(w, ",\"ai\":");
    ai_watch_write(w);
    jw_raw(w, ",\"settings\":{\"host\":");
    jw_str(w, set.host);
    jw_key(w, false, "port");
    jw_num(w, set.port);
    jw_key(w, false, "username");
    jw_str(w, set.username);
    jw_key(w, false, "has_password");
    jw_raw(w, jbool(set.password[0] != '\0'));
    jw_key(w, false, "keepalive_s");
    jw_num(w, set.keepalive_s);
    jw_key(w, false, "prefix");
    jw_str(w, set.prefix);
    jw_raw(w, ",\"topics\":[");
    for (int a = 0; a < DEMO_AREA_COUNT; a++) {
        if (a) {
            jw_raw(w, ",");
        }
        jw_str(w, DEMO_AREAS[a].key);
    }
    jw_raw(w, "],\"theme\":");
    jw_str(w, set.theme ? "light" : "dark");
    jw_key(w, false, "animation");
    jw_raw(w, jbool(set.animation));
    jw_key(w, false, "fahrenheit");
    jw_raw(w, jbool(set.fahrenheit));
    jw_raw(w, "},\"events\":");
    event_log_write(w, query_u32(req, "after"));
    jw_raw(w, "}");
    return stream_end(s);
}

static esp_err_t h_cmd(httpd_req_t *req)
{
    if (!admin_ok(req)) {
        return ESP_OK;
    }
    cJSON *root = read_json(req);
    if (!root) {
        return ESP_OK;
    }
    char msg[128];
    bool ok = engine_command(root, msg, sizeof(msg));
    cJSON_Delete(root);
    return send_result(req, ok, msg);
}

static esp_err_t h_scenario_get(httpd_req_t *req)
{
    if (!admin_ok(req)) {
        return ESP_OK;
    }
    int id = (int)query_u32(req, "id");
    if (id < 1 || id > library_count()) {
        return send_result(req, false, "no such scenario");
    }
    stream_t *s = stream_begin(req);
    if (!s) {
        return ESP_FAIL;
    }
    library_write_scenario(&s->w, id - 1);
    return stream_end(s);
}

// {"id":n|null,"name":"...","group":"air","set":{"kitchen.temperature_c":24.5,...}}
static bool parse_scenario(const cJSON *root, demo_scenario_t *out, int *index, char *msg,
                           size_t len)
{
    memset(out, 0, sizeof(*out));
    const cJSON *id = cJSON_GetObjectItemCaseSensitive(root, "id");
    *index = -1;
    if (cJSON_IsNumber(id)) {
        *index = (int)id->valuedouble - 1;
    } else if (id && !cJSON_IsNull(id)) {
        snprintf(msg, len, "\"id\" must be a number or null");
        return false;
    }
    const cJSON *name = cJSON_GetObjectItemCaseSensitive(root, "name");
    if (!cJSON_IsString(name) || strlen(name->valuestring) > DEMO_NAME_MAX) {
        snprintf(msg, len, "\"name\" must be text of at most %d bytes", DEMO_NAME_MAX);
        return false;
    }
    strlcpy(out->name, name->valuestring, sizeof(out->name));
    const cJSON *group = cJSON_GetObjectItemCaseSensitive(root, "group");
    out->group = 0xFF;
    for (int g = 0; cJSON_IsString(group) && g < DEMO_GROUP_COUNT; g++) {
        if (strcmp(group->valuestring, DEMO_GROUPS[g]) == 0) {
            out->group = (uint8_t)g;
        }
    }
    if (out->group == 0xFF) {
        snprintf(msg, len, "unknown \"group\"");
        return false;
    }
    const cJSON *expect = cJSON_GetObjectItemCaseSensitive(root, "expect");
    if (expect && (!cJSON_IsString(expect) || strlen(expect->valuestring) > DEMO_EXPECT_MAX)) {
        snprintf(msg, len, "\"expect\" must be text of at most %d bytes", DEMO_EXPECT_MAX);
        return false;
    }
    if (expect) {
        strlcpy(out->expect, expect->valuestring, sizeof(out->expect));
    }
    const cJSON *bell = cJSON_GetObjectItemCaseSensitive(root, "doorbell");
    if (bell && !cJSON_IsBool(bell)) {
        snprintf(msg, len, "\"doorbell\" must be true or false");
        return false;
    }
    out->flags = cJSON_IsTrue(bell) ? SCEN_DOORBELL : 0;
    const cJSON *set = cJSON_GetObjectItemCaseSensitive(root, "set");
    if (!cJSON_IsObject(set)) {
        snprintf(msg, len, "\"set\" must be an object of entity: value");
        return false;
    }
    const cJSON *it;
    cJSON_ArrayForEach(it, set)
    {
        int e = demo_entity_find(it->string);
        if (e < 0) {
            snprintf(msg, len, "unknown entity %.40s", it->string);
            return false;
        }
        if (out->n_sets >= DEMO_MAX_SETS) {
            snprintf(msg, len, "a scenario sets at most %d values", DEMO_MAX_SETS);
            return false;
        }
        const demo_entity_t *d = &DEMO_ENTITIES[e];
        float v = NAN;
        if (d->type == DEMO_NUM && cJSON_IsNumber(it)) {
            v = (float)it->valuedouble;
        } else if (d->type == DEMO_BOOL && cJSON_IsBool(it)) {
            v = cJSON_IsTrue(it) ? 1 : 0;
        } else if (d->type == DEMO_ENUM && cJSON_IsString(it)) {
            for (int i = 0; i < d->n_options; i++) {
                if (strcmp(it->valuestring, d->options[i]) == 0) {
                    v = (float)i;
                }
            }
        }
        if (isnan(v)) {
            snprintf(msg, len, "%s needs a %s", d->key,
                     d->type == DEMO_NUM ? "number" : d->type == DEMO_BOOL ? "true/false" : "listed option");
            return false;
        }
        out->sets[out->n_sets].entity = (uint8_t)e;
        out->sets[out->n_sets].value = v;
        out->n_sets++;
    }
    return true;
}

static esp_err_t h_scenario_put(httpd_req_t *req)
{
    if (!admin_ok(req)) {
        return ESP_OK;
    }
    cJSON *root = read_json(req);
    if (!root) {
        return ESP_OK;
    }
    char msg[128];
    demo_scenario_t *s = malloc(sizeof(demo_scenario_t));
    int index = -1;
    bool ok = s && parse_scenario(root, s, &index, msg, sizeof(msg)) &&
              library_put(index, s, &index, msg, sizeof(msg));
    if (!s) {
        snprintf(msg, sizeof(msg), "out of memory");
    }
    free(s);
    cJSON_Delete(root);
    if (ok) {
        event_log("Admin: %s", msg);
    }
    return send_result(req, ok, msg);
}

static esp_err_t h_scenario_delete(httpd_req_t *req)
{
    if (!admin_ok(req)) {
        return ESP_OK;
    }
    cJSON *root = read_json(req);
    if (!root) {
        return ESP_OK;
    }
    const cJSON *id = cJSON_GetObjectItemCaseSensitive(root, "id");
    char msg[128];
    bool ok = cJSON_IsNumber(id) && library_delete((int)id->valuedouble - 1, msg, sizeof(msg));
    if (!cJSON_IsNumber(id)) {
        snprintf(msg, sizeof(msg), "delete needs \"id\"");
    }
    cJSON_Delete(root);
    if (ok) {
        event_log("Admin: %s", msg);
    }
    return send_result(req, ok, msg);
}

static esp_err_t h_library_reset(httpd_req_t *req)
{
    if (!admin_ok(req)) {
        return ESP_OK;
    }
    char msg[128];
    bool ok = library_reset(msg, sizeof(msg));
    event_log("Admin: %s", msg);
    return send_result(req, ok, msg);
}

static bool get_float(const cJSON *root, const char *key, float *out)
{
    const cJSON *it = cJSON_GetObjectItemCaseSensitive(root, key);
    if (cJSON_IsNumber(it)) {
        *out = (float)it->valuedouble;
        return true;
    }
    return it == NULL;
}

// {"key":"kitchen.temperature_c","min":..,"max":..,"nmin":..,"nmax":..}; missing = unchanged
static esp_err_t h_limit(httpd_req_t *req)
{
    if (!admin_ok(req)) {
        return ESP_OK;
    }
    cJSON *root = read_json(req);
    if (!root) {
        return ESP_OK;
    }
    char msg[128];
    bool ok = false;
    const cJSON *key = cJSON_GetObjectItemCaseSensitive(root, "key");
    int e = cJSON_IsString(key) ? demo_entity_find(key->valuestring) : -1;
    if (e < 0) {
        snprintf(msg, sizeof(msg), "unknown \"key\"");
    } else {
        demo_limit_t l = *demo_limit(e);
        if (!get_float(root, "min", &l.min) || !get_float(root, "max", &l.max) ||
            !get_float(root, "nmin", &l.nmin) || !get_float(root, "nmax", &l.nmax)) {
            snprintf(msg, sizeof(msg), "limits must be numbers");
        } else {
            ok = library_set_limit(e, &l, msg, sizeof(msg));
        }
    }
    cJSON_Delete(root);
    if (ok) {
        event_log("Admin: %s", msg);
    }
    return send_result(req, ok, msg);
}

static esp_err_t h_limits_reset(httpd_req_t *req)
{
    if (!admin_ok(req)) {
        return ESP_OK;
    }
    char msg[128];
    bool ok = library_reset_limits(msg, sizeof(msg));
    event_log("Admin: %s", msg);
    return send_result(req, ok, msg);
}

static bool copy_str(const cJSON *root, const char *key, char *dst, size_t cap, char *msg,
                     size_t len)
{
    const cJSON *it = cJSON_GetObjectItemCaseSensitive(root, key);
    if (!it) {
        return true;
    }
    if (!cJSON_IsString(it) || strlen(it->valuestring) >= cap) {
        snprintf(msg, len, "\"%s\" must be text of at most %u characters", key, (unsigned)cap - 1);
        return false;
    }
    strlcpy(dst, it->valuestring, cap);
    return true;
}

static bool copy_num(const cJSON *root, const char *key, int lo, int hi, int *out, char *msg,
                     size_t len)
{
    const cJSON *it = cJSON_GetObjectItemCaseSensitive(root, key);
    if (!it) {
        return true;
    }
    if (!cJSON_IsNumber(it) || it->valuedouble < lo || it->valuedouble > hi) {
        snprintf(msg, len, "\"%s\" must be %d..%d", key, lo, hi);
        return false;
    }
    *out = (int)it->valuedouble;
    return true;
}

static bool copy_bool(const cJSON *root, const char *key, bool *out, char *msg, size_t len)
{
    const cJSON *it = cJSON_GetObjectItemCaseSensitive(root, key);
    if (!it) {
        return true;
    }
    if (!cJSON_IsBool(it)) {
        snprintf(msg, len, "\"%s\" must be true or false", key);
        return false;
    }
    *out = cJSON_IsTrue(it);
    return true;
}

// Partial update: only the fields present change. "password" absent = keep it.
static esp_err_t h_settings(httpd_req_t *req)
{
    if (!admin_ok(req)) {
        return ESP_OK;
    }
    cJSON *root = read_json(req);
    if (!root) {
        return ESP_OK;
    }
    dm_settings_t old, s;
    settings_get(&old);
    s = old;
    char msg[128] = "";
    int port = s.port, keepalive = s.keepalive_s;
    bool ok = copy_str(root, "host", s.host, sizeof(s.host), msg, sizeof(msg)) &&
              copy_num(root, "port", 1, 65535, &port, msg, sizeof(msg)) &&
              copy_str(root, "username", s.username, sizeof(s.username), msg, sizeof(msg)) &&
              copy_str(root, "password", s.password, sizeof(s.password), msg, sizeof(msg)) &&
              copy_num(root, "keepalive_s", 5, 120, &keepalive, msg, sizeof(msg)) &&
              copy_str(root, "prefix", s.prefix, sizeof(s.prefix), msg, sizeof(msg)) &&
              copy_bool(root, "animation", &s.animation, msg, sizeof(msg)) &&
              copy_bool(root, "fahrenheit", &s.fahrenheit, msg, sizeof(msg)) &&
              copy_str(root, "pin", s.pin, sizeof(s.pin), msg, sizeof(msg));
    const cJSON *theme = cJSON_GetObjectItemCaseSensitive(root, "theme");
    if (ok && theme) {
        ok = cJSON_IsString(theme) &&
             (strcmp(theme->valuestring, "dark") == 0 || strcmp(theme->valuestring, "light") == 0);
        if (ok) {
            s.theme = strcmp(theme->valuestring, "light") == 0;
        } else {
            snprintf(msg, sizeof(msg), "\"theme\" must be \"dark\" or \"light\"");
        }
    }
    s.port = (uint16_t)port;
    s.keepalive_s = (uint16_t)keepalive;
    cJSON_Delete(root);
    if (ok) {
        ok = settings_save(&s, msg, sizeof(msg));
    }
    if (!ok) {
        return send_result(req, false, msg);
    }
    bool mqtt = strcmp(old.host, s.host) || old.port != s.port || strcmp(old.username, s.username) ||
                strcmp(old.password, s.password) || old.keepalive_s != s.keepalive_s ||
                strcmp(old.prefix, s.prefix);
    event_log("Admin: settings saved%s", mqtt ? ", MQTT reconnecting" : "");
    esp_err_t err = send_result(req, true, mqtt ? "saved; reconnecting to the broker" : "saved");
    if (mqtt) {
        mqtt_link_restart();  // republishes on connect
    }
    return err;
}

static esp_err_t h_settings_reset(httpd_req_t *req)
{
    if (!admin_ok(req)) {
        return ESP_OK;
    }
    char msg[128];
    settings_reset(msg, sizeof(msg));
    event_log("Admin: %s", msg);
    esp_err_t err = send_result(req, true, "settings back to the defaults; reconnecting");
    mqtt_link_restart();
    return err;
}

static void reboot_task(void *arg)
{
    vTaskDelay(pdMS_TO_TICKS(500));
    esp_restart();
}

static esp_err_t h_reboot(httpd_req_t *req)
{
    if (!admin_ok(req)) {
        return ESP_OK;
    }
    event_log("Admin: reboot");
    esp_err_t err = send_result(req, true, "rebooting");
    xTaskCreate(reboot_task, "reboot", 2048, NULL, 5, NULL);
    return err;
}

// ---------------------------------------------------------------------------

void web_server_start(void)
{
    httpd_config_t cfg = HTTPD_DEFAULT_CONFIG();
    cfg.max_uri_handlers = 24;
    cfg.max_open_sockets = 8;
    cfg.lru_purge_enable = true;
    cfg.stack_size = 6144;
    httpd_handle_t srv = NULL;
    if (httpd_start(&srv, &cfg) != ESP_OK) {
        ESP_LOGE(TAG, "HTTP server didn't start");
        return;
    }
    const httpd_uri_t uris[] = {
        {.uri = "/", .method = HTTP_GET, .handler = h_demo},
        {.uri = "/admin", .method = HTTP_GET, .handler = h_admin},
        {.uri = "/api/state", .method = HTTP_GET, .handler = h_state},
        {.uri = "/api/meta", .method = HTTP_GET, .handler = h_meta},
        {.uri = "/api/press", .method = HTTP_POST, .handler = h_press},
        {.uri = "/api/admin/login", .method = HTTP_POST, .handler = h_login},
        {.uri = "/api/admin/status", .method = HTTP_GET, .handler = h_status},
        {.uri = "/api/admin/cmd", .method = HTTP_POST, .handler = h_cmd},
        {.uri = "/api/admin/scenario", .method = HTTP_GET, .handler = h_scenario_get},
        {.uri = "/api/admin/scenario", .method = HTTP_POST, .handler = h_scenario_put},
        {.uri = "/api/admin/scenario/delete", .method = HTTP_POST, .handler = h_scenario_delete},
        {.uri = "/api/admin/library/reset", .method = HTTP_POST, .handler = h_library_reset},
        {.uri = "/api/admin/limit", .method = HTTP_POST, .handler = h_limit},
        {.uri = "/api/admin/limits/reset", .method = HTTP_POST, .handler = h_limits_reset},
        {.uri = "/api/admin/settings", .method = HTTP_POST, .handler = h_settings},
        {.uri = "/api/admin/settings/reset", .method = HTTP_POST, .handler = h_settings_reset},
        {.uri = "/api/admin/reboot", .method = HTTP_POST, .handler = h_reboot},
    };
    for (size_t i = 0; i < sizeof(uris) / sizeof(uris[0]); i++) {
        httpd_register_uri_handler(srv, &uris[i]);
    }
    ESP_LOGI(TAG, "Web server up: / (demo) and /admin");
}
