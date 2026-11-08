#include "emulator_ctrl.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "ai_message.h"
#include "app_config.h"
#include "cJSON.h"
#include "demo_engine.h"
#include "esp_app_desc.h"
#include "esp_log.h"
#include "esp_system.h"
#include "esp_timer.h"
#include "event_log.h"
#include "freertos/FreeRTOS.h"
#include "freertos/semphr.h"
#include "history.h"
#include "json_writer.h"
#include "mqtt_client.h"
#include "mqtt_node.h"
#include "payload.h"
#include "time_sync.h"
#include "virtual_node.h"
#include "wifi_manager.h"

static const char *TAG = "ctrl";

#define CMD_MAX_LEN 1024
#define DURATION_MAX_S 86400
#define DISPLAY_TEXT_MAX 400

static esp_mqtt_client_handle_t s_client;
static volatile bool s_connected;
static volatile bool s_state_dirty;
static SemaphoreHandle_t s_cmd_lock;

// Inbound message being reassembled (touched only by the control client's MQTT task)
typedef enum { RX_CMD, RX_AI_SUMMARY, RX_AI_INFERENCE, RX_OTHER } rx_kind_t;
static char s_rx[CMD_MAX_LEN + 1];
static size_t s_rx_len;
static bool s_rx_retained;
static rx_kind_t s_rx_kind;

typedef struct {
    bool valid;
    bool ok;
    char msg[160];
    char source[8];
    int64_t at_us;
} last_cmd_t;
static last_cmd_t s_last;

// ---------------------------------------------------------------------------
// Commands

static int resolve_nodes(const cJSON *root, vnode_t **out, char *msg, size_t msg_len)
{
    const cJSON *node = cJSON_GetObjectItemCaseSensitive(root, "node");
    if (!cJSON_IsString(node) || !node->valuestring) {
        snprintf(msg, msg_len, "missing \"node\" (node ID, room name or \"all\")");
        return 0;
    }
    if (strcmp(node->valuestring, "all") == 0) {
        for (int i = 0; i < NODE_COUNT; i++) {
            out[i] = &g_nodes[i];
        }
        return NODE_COUNT;
    }
    vnode_t *n = node_find(node->valuestring);
    if (!n) {
        snprintf(msg, msg_len, "unknown node \"%.40s\"", node->valuestring);
        return 0;
    }
    out[0] = n;
    return 1;
}

static const char *nodes_name(vnode_t **nodes, int count)
{
    return count == 1 ? nodes[0]->id : "all nodes";
}

static bool get_duration(const cJSON *root, int32_t dflt, int32_t *out, char *msg, size_t msg_len)
{
    const cJSON *d = cJSON_GetObjectItemCaseSensitive(root, "duration_s");
    if (!d || cJSON_IsNull(d)) {
        *out = dflt;
        return true;
    }
    if (!cJSON_IsNumber(d) || d->valuedouble < 0 || d->valuedouble > DURATION_MAX_S) {
        snprintf(msg, msg_len, "\"duration_s\" must be 0..%d (0 = until cleared)", DURATION_MAX_S);
        return false;
    }
    *out = (int32_t)d->valuedouble;
    return true;
}

static void describe_duration(char *buf, size_t len, int32_t s)
{
    if (s == 0) {
        snprintf(buf, len, "until cleared");
    } else {
        snprintf(buf, len, "for %ld s", (long)s);
    }
}

static bool cmd_scenario(const cJSON *root, vnode_t **nodes, int count, char *msg, size_t len)
{
    const cJSON *name = cJSON_GetObjectItemCaseSensitive(root, "name");
    if (!cJSON_IsString(name) || !name->valuestring) {
        snprintf(msg, len, "missing scenario \"name\"");
        return false;
    }
    const scenario_t *sc = scenario_find(name->valuestring);
    if (!sc) {
        snprintf(msg, len, "unknown scenario \"%.40s\"", name->valuestring);
        return false;
    }
    int32_t dur;
    if (!get_duration(root, (int32_t)sc->duration_s, &dur, msg, len)) {
        return false;
    }
    nodes_lock();
    for (int i = 0; i < count; i++) {
        sim_start_scenario(&nodes[i]->sim, sc, dur);
    }
    nodes_unlock();
    char d[32];
    describe_duration(d, sizeof(d), dur);
    snprintf(msg, len, "%s: %s %s", nodes_name(nodes, count), sc->label, d);
    return true;
}

static bool parse_range(const cJSON *item, int m, range_override_t *out, char *msg, size_t len)
{
    const metric_info_t *mi = &METRIC_INFO[m];
    float lo, hi;
    if (cJSON_IsNumber(item)) {
        lo = hi = (float)item->valuedouble;
    } else if (cJSON_IsObject(item)) {
        const cJSON *mn = cJSON_GetObjectItemCaseSensitive(item, "min");
        const cJSON *mx = cJSON_GetObjectItemCaseSensitive(item, "max");
        if (!cJSON_IsNumber(mn) || !cJSON_IsNumber(mx)) {
            snprintf(msg, len, "%s: need numeric \"min\" and \"max\"", mi->key);
            return false;
        }
        lo = (float)mn->valuedouble;
        hi = (float)mx->valuedouble;
    } else {
        snprintf(msg, len, "%s: expected a number or {\"min\",\"max\"}", mi->key);
        return false;
    }
    if (lo > hi) {
        snprintf(msg, len, "%s: min is greater than max", mi->key);
        return false;
    }
    if (lo < mi->min || hi > mi->max) {
        snprintf(msg, len, "%s: must be within %g..%g %s", mi->key, (double)mi->min,
                 (double)mi->max, mi->unit);
        return false;
    }
    out->active = true;
    out->min = lo;
    out->max = hi;
    return true;
}

static bool cmd_set(const cJSON *root, vnode_t **nodes, int count, char *msg, size_t len)
{
    sim_override_req_t req;
    memset(&req, 0, sizeof(req));
    req.presence = -1;
    int fields = 0;
    char names[96] = "";

    const cJSON *ranges = cJSON_GetObjectItemCaseSensitive(root, "ranges");
    if (ranges && !cJSON_IsObject(ranges)) {
        snprintf(msg, len, "\"ranges\" must be an object");
        return false;
    }
    const cJSON *it;
    cJSON_ArrayForEach(it, ranges) {
        if (strcmp(it->string, "presence") == 0) {
            if (!cJSON_IsBool(it)) {
                snprintf(msg, len, "presence must be true or false");
                return false;
            }
            req.presence = cJSON_IsTrue(it) ? 1 : 0;
            continue;
        }
        int m = sim_metric_from_key(it->string);
        if (m < 0) {
            snprintf(msg, len, "unknown metric \"%.32s\"", it->string);
            return false;
        }
        if (!parse_range(it, m, &req.m[m], msg, len)) {
            return false;
        }
        fields++;
        strlcat(names, names[0] ? ", " : "", sizeof(names));
        strlcat(names, it->string, sizeof(names));
    }
    const cJSON *presence = cJSON_GetObjectItemCaseSensitive(root, "presence");
    if (presence) {
        if (!cJSON_IsBool(presence)) {
            snprintf(msg, len, "presence must be true or false");
            return false;
        }
        req.presence = cJSON_IsTrue(presence) ? 1 : 0;
    }
    if (req.presence >= 0) {
        strlcat(names, names[0] ? ", " : "", sizeof(names));
        strlcat(names, req.presence ? "presence=true" : "presence=false", sizeof(names));
    }
    if (fields == 0 && req.presence < 0) {
        snprintf(msg, len, "nothing to set: give \"ranges\" and/or \"presence\"");
        return false;
    }
    int32_t dur;
    if (!get_duration(root, 0, &dur, msg, len)) {
        return false;
    }
    bool instant = cJSON_IsTrue(cJSON_GetObjectItemCaseSensitive(root, "instant"));

    nodes_lock();
    for (int i = 0; i < count; i++) {
        sim_apply_override(&nodes[i]->sim, &req, (uint32_t)dur, instant);
    }
    nodes_unlock();
    char d[32];
    describe_duration(d, sizeof(d), dur);
    snprintf(msg, len, "%s: set %s %s%s", nodes_name(nodes, count), names, d,
             instant ? " (instant)" : "");
    return true;
}

static bool cmd_display(const cJSON *root, vnode_t **nodes, int count, char *msg, size_t len)
{
    if (count != 1) {
        snprintf(msg, len, "display needs a single node, not \"all\"");
        return false;
    }
    const cJSON *text = cJSON_GetObjectItemCaseSensitive(root, "text");
    if (!cJSON_IsString(text) || !text->valuestring || !text->valuestring[0]) {
        snprintf(msg, len, "missing \"text\"");
        return false;
    }
    if (strlen(text->valuestring) > DISPLAY_TEXT_MAX) {
        snprintf(msg, len, "text longer than %d characters", DISPLAY_TEXT_MAX);
        return false;
    }
    bool retain = cJSON_IsTrue(cJSON_GetObjectItemCaseSensitive(root, "retain"));

    cJSON *out = cJSON_CreateObject();
    cJSON_AddStringToObject(out, "text", text->valuestring);
    char ts[40];
    if (time_iso8601(ts, sizeof(ts))) {
        cJSON_AddStringToObject(out, "ts", ts);
    }
    char *payload = cJSON_PrintUnformatted(out);
    cJSON_Delete(out);
    if (!payload) {
        snprintf(msg, len, "out of memory");
        return false;
    }

    // Publish like Home Assistant would; the node's own subscription prints it.
    esp_mqtt_client_handle_t via = s_connected ? s_client
                                   : (nodes[0]->mqtt_connected ? nodes[0]->client : NULL);
    int id = via ? esp_mqtt_client_publish(via, nodes[0]->topic_display, payload, 0, 1, retain)
                 : -1;
    size_t bytes = strlen(payload);
    cJSON_free(payload);
    if (id < 0) {
        snprintf(msg, len, "MQTT not connected; message not sent");
        return false;
    }
    event_log_route(ROUTE_DISPLAY_OUT, nodes[0]->topic_display, bytes);
    snprintf(msg, len, "%s: test message published to %s%s", nodes[0]->id,
             nodes[0]->topic_display, retain ? " (retained)" : "");
    return true;
}

void ctrl_record(const char *source, bool ok, const char *msg)
{
    xSemaphoreTake(s_cmd_lock, portMAX_DELAY);  // guards s_last only
    s_last.valid = true;
    s_last.ok = ok;
    strlcpy(s_last.msg, msg, sizeof(s_last.msg));
    strlcpy(s_last.source, source, sizeof(s_last.source));
    s_last.at_us = esp_timer_get_time();
    s_state_dirty = true;
    xSemaphoreGive(s_cmd_lock);

    event_log_cmd(source, ok, msg);
    if (ok) {
        ESP_LOGI(TAG, "[%s] %s", source, msg);
    } else {
        ESP_LOGW(TAG, "[%s] rejected: %s", source, msg);
    }
}

bool ctrl_execute(const char *json, size_t len, const char *source, char *msg, size_t msg_len)
{
    // No lock held while running: sim changes take the nodes lock, and "display" publishes
    // over MQTT, which must never happen while holding a lock another MQTT task may want.
    bool ok = false;
    msg[0] = '\0';

    cJSON *root = NULL;
    const cJSON *action = NULL;
    vnode_t *nodes[NODE_COUNT];
    int count = 0;

    if (len > CMD_MAX_LEN) {
        snprintf(msg, msg_len, "command longer than %d bytes", CMD_MAX_LEN);
        goto done;
    }
    root = cJSON_ParseWithLength(json, len);
    if (!cJSON_IsObject(root)) {
        snprintf(msg, msg_len, "invalid JSON");
        goto done;
    }
    action = cJSON_GetObjectItemCaseSensitive(root, "action");
    if (!cJSON_IsString(action) || !action->valuestring) {
        snprintf(msg, msg_len, "missing \"action\" (scenario, set, clear, reset, display, demo)");
        goto done;
    }
    if (strcmp(action->valuestring, "demo") == 0) {
        ok = demo_engine_command(root, msg, msg_len);  // whole-home demo: no "node"
        goto done;
    }
    count = resolve_nodes(root, nodes, msg, msg_len);
    if (count == 0) {
        goto done;
    }

    const char *a = action->valuestring;
    if (strcmp(a, "scenario") == 0) {
        ok = cmd_scenario(root, nodes, count, msg, msg_len);
    } else if (strcmp(a, "set") == 0) {
        ok = cmd_set(root, nodes, count, msg, msg_len);
    } else if (strcmp(a, "clear") == 0 || strcmp(a, "reset") == 0) {
        bool reset = a[0] == 'r';
        nodes_lock();
        for (int i = 0; i < count; i++) {
            if (reset) {
                sim_reset(&nodes[i]->sim);
            } else {
                sim_clear(&nodes[i]->sim);
            }
        }
        nodes_unlock();
        snprintf(msg, msg_len, "%s: %s", nodes_name(nodes, count),
                 reset ? "reset to baseline" : "back to automatic behaviour");
        ok = true;
    } else if (strcmp(a, "display") == 0) {
        ok = cmd_display(root, nodes, count, msg, msg_len);
    } else {
        snprintf(msg, msg_len, "unknown action \"%.32s\"", a);
    }

done:
    cJSON_Delete(root);
    if (strcmp(source, "mqtt") == 0) {
        event_log_route(ROUTE_CMD_MQTT, TOPIC_EMULATOR_CMD, len);
    } else {
        event_log_route(ROUTE_CMD_HTTP, "POST /api/cmd", len);
    }
    ctrl_record(source, ok, msg);
    return ok;
}

// ---------------------------------------------------------------------------
// State / meta JSON

// One node's part of the state, copied under the nodes lock and written after it is released
// (the writer may be sending to a socket).
typedef struct {
    const vnode_t *n;  // id and profile are fixed after boot
    bool mqtt_connected, has_last;
    uint32_t reconnects, publish_count;
    int64_t last_publish_us;
    const char *mode;
    int occupants;
    const scenario_t *scenario, *auto_event;
    bool scenario_forever, ovr_forever;
    int scenario_left_s, auto_event_left_s, ovr_left_s;
    int8_t presence_ovr;
    range_override_t ovr[METRIC_COUNT];
    sim_reading_t last;
} node_copy_t;

static void copy_node(node_copy_t *c, const vnode_t *n)
{
    const sim_state_t *s = &n->sim;
    c->n = n;
    c->mqtt_connected = n->mqtt_connected;
    c->reconnects = n->mqtt_reconnects;
    c->publish_count = n->publish_count;
    c->has_last = n->has_last;
    c->last_publish_us = n->last_publish_us;
    c->last = n->last;
    c->mode = sim_mode(s);
    c->occupants = sim_occupants(s);
    c->scenario = s->scenario;
    c->scenario_forever = s->scenario_forever;
    c->scenario_left_s = (int)s->scenario_left_s;
    c->auto_event = s->auto_event;
    c->auto_event_left_s = (int)s->auto_event_left_s;
    memcpy(c->ovr, s->ovr, sizeof(c->ovr));
    c->presence_ovr = s->presence_ovr;
    c->ovr_forever = s->ovr_forever;
    c->ovr_left_s = (int)s->ovr_left_s;
}

static void write_node(jw_t *w, const node_copy_t *c, int64_t now)
{
    jw_key(w, true, "id");
    jw_str(w, c->n->id);
    jw_key(w, false, "room");
    jw_str(w, c->n->profile->name);
    jw_key(w, false, "label");
    jw_str(w, c->n->profile->label);
    jw_key(w, false, "mqtt");
    jw_str(w, c->mqtt_connected ? "connected" : "disconnected");
    jw_key(w, false, "mqtt_reconnects");
    jw_num(w, c->reconnects);
    jw_key(w, false, "publish_count");
    jw_num(w, c->publish_count);
    jw_key(w, false, "last_publish_age_s");
    if (c->has_last) {
        jw_num(w, (double)((now - c->last_publish_us) / 1000000LL));
    } else {
        jw_raw(w, "null");
    }
    jw_key(w, false, "mode");
    jw_str(w, c->mode);
    jw_key(w, false, "occupants");
    jw_num(w, c->occupants);

    jw_key(w, false, "scenario");
    if (c->scenario) {
        jw_str(w, c->scenario->name);
        jw_key(w, false, "scenario_label");
        jw_str(w, c->scenario->label);
        jw_key(w, false, "scenario_remaining_s");
        if (c->scenario_forever) {
            jw_raw(w, "null");
        } else {
            jw_num(w, c->scenario_left_s);
        }
    } else {
        jw_raw(w, "null");
    }
    jw_key(w, false, "auto_event");
    if (c->auto_event) {
        jw_str(w, c->auto_event->name);
        jw_key(w, false, "auto_event_remaining_s");
        jw_num(w, c->auto_event_left_s);
    } else {
        jw_raw(w, "null");
    }

    jw_key(w, false, "overrides");
    jw_raw(w, "{");
    bool any = false;
    for (int m = 0; m < METRIC_COUNT; m++) {
        if (c->ovr[m].active) {
            jw_key(w, !any, METRIC_INFO[m].key);
            jw_raw(w, "{\"min\":");
            jw_num(w, c->ovr[m].min);
            jw_raw(w, ",\"max\":");
            jw_num(w, c->ovr[m].max);
            jw_raw(w, "}");
            any = true;
        }
    }
    jw_raw(w, "}");
    jw_key(w, false, "presence_override");
    if (c->presence_ovr >= 0) {
        jw_raw(w, jbool(c->presence_ovr == 1));
        any = true;
    } else {
        jw_raw(w, "null");
    }
    jw_key(w, false, "override_remaining_s");
    if (any && !c->ovr_forever) {
        jw_num(w, c->ovr_left_s);
    } else {
        jw_raw(w, "null");
    }

    jw_key(w, false, "reading");
    if (c->has_last) {
        jw_raw(w, "{");
        for (int m = 0; m < METRIC_COUNT; m++) {
            jw_key(w, m == 0, METRIC_INFO[m].key);
            jw_num(w, c->last.v[m]);
        }
        jw_key(w, false, "presence");
        jw_raw(w, jbool(c->last.presence));
        jw_raw(w, "}");
    } else {
        jw_raw(w, "null");
    }
}

void ctrl_write_state(jw_t *w)
{
    int64_t now = esp_timer_get_time();
    char ip[16];
    wifi_get_ip(ip, sizeof(ip));
    char ts[40];
    bool synced = time_iso8601(ts, sizeof(ts));

    jw_raw(w, "{\"fw\":");
    jw_str(w, esp_app_get_description()->version);
    jw_key(w, false, "hostname");
    jw_str(w, CONFIG_EMU_HOSTNAME);
    jw_key(w, false, "uptime_s");
    jw_num(w, (double)(now / 1000000LL));
    jw_key(w, false, "free_heap");
    jw_num(w, esp_get_free_heap_size());
    jw_key(w, false, "min_free_heap");
    jw_num(w, esp_get_minimum_free_heap_size());

    jw_raw(w, ",\"wifi\":{\"connected\":");
    jw_raw(w, jbool(wifi_is_connected()));
    jw_key(w, false, "ssid");
    jw_str(w, CONFIG_EMU_WIFI_SSID);
    jw_key(w, false, "ip");
    jw_str(w, ip);
    jw_key(w, false, "rssi");
    jw_num(w, wifi_get_rssi());
    jw_key(w, false, "reconnects");
    jw_num(w, wifi_reconnect_count());

    char uri[96];
    mqtt_broker_uri(uri, sizeof(uri));
    jw_raw(w, "},\"mqtt\":{\"broker\":");
    jw_str(w, uri);
    jw_key(w, false, "reconfiguring");
    jw_raw(w, jbool(mqtt_reconfiguring()));
    jw_key(w, false, "emulator_connected");
    jw_raw(w, jbool(s_connected));

    jw_raw(w, "},\"time\":{\"synced\":");
    jw_raw(w, jbool(synced));
    jw_key(w, false, "now");
    if (synced) {
        jw_str(w, ts);
    } else {
        jw_raw(w, "null");
    }
    jw_key(w, false, "local_hour");
    jw_num(w, time_local_hour());

    jw_raw(w, "},\"publish_interval_s\":");
    jw_num(w, app_config_publish_interval_ms() / 1000.0);
    jw_key(w, false, "sim_speed");
    jw_num(w, CONFIG_EMU_SIM_SPEED);
    jw_key(w, false, "config_saved");
    jw_raw(w, jbool(app_config_saved()));

    jw_raw(w, ",\"nodes\":[");
    for (int i = 0; i < NODE_COUNT; i++) {
        node_copy_t c;  // ~170 bytes
        nodes_lock();
        copy_node(&c, &g_nodes[i]);
        nodes_unlock();
        jw_raw(w, i ? ",{" : "{");
        write_node(w, &c, now);
        jw_raw(w, "}");
    }
    jw_raw(w, "]");

    last_cmd_t last;
    xSemaphoreTake(s_cmd_lock, portMAX_DELAY);
    last = s_last;
    xSemaphoreGive(s_cmd_lock);
    jw_key(w, false, "last_cmd");
    if (last.valid) {
        jw_raw(w, "{\"ok\":");
        jw_raw(w, jbool(last.ok));
        jw_key(w, false, "msg");
        jw_str(w, last.msg);
        jw_key(w, false, "source");
        jw_str(w, last.source);
        jw_key(w, false, "age_s");
        jw_num(w, (double)((now - last.at_us) / 1000000LL));
        jw_raw(w, "}");
    } else {
        jw_raw(w, "null");
    }
    jw_key(w, false, "demo");
    demo_engine_write_state(w);
    jw_raw(w, "}");
}

char *ctrl_meta_json(void)
{
    cJSON *root = cJSON_CreateObject();
    cJSON *metrics = cJSON_AddArrayToObject(root, "metrics");
    for (int m = 0; m < METRIC_COUNT; m++) {
        const metric_info_t *mi = &METRIC_INFO[m];
        cJSON *o = cJSON_CreateObject();
        cJSON_AddStringToObject(o, "key", mi->key);
        cJSON_AddStringToObject(o, "label", mi->label);
        cJSON_AddStringToObject(o, "unit", mi->unit);
        cJSON_AddNumberToObject(o, "min", mi->min);
        cJSON_AddNumberToObject(o, "max", mi->max);
        cJSON_AddNumberToObject(o, "decimals", mi->decimals);
        cJSON_AddItemToArray(metrics, o);
    }
    cJSON *scen = cJSON_AddArrayToObject(root, "scenarios");
    for (int i = 0; i < SCENARIO_COUNT; i++) {
        const scenario_t *sc = &SCENARIOS[i];
        cJSON *o = cJSON_CreateObject();
        cJSON_AddStringToObject(o, "name", sc->name);
        cJSON_AddStringToObject(o, "label", sc->label);
        cJSON_AddStringToObject(o, "icon", sc->icon);
        cJSON_AddStringToObject(o, "default_room", sc->default_room);
        cJSON_AddNumberToObject(o, "duration_s", sc->duration_s);
        cJSON_AddItemToArray(scen, o);
    }
    cJSON *nodes = cJSON_AddArrayToObject(root, "nodes");
    for (int i = 0; i < NODE_COUNT; i++) {
        cJSON *o = cJSON_CreateObject();
        cJSON_AddStringToObject(o, "id", g_nodes[i].id);
        cJSON_AddStringToObject(o, "room", g_nodes[i].profile->name);
        cJSON_AddStringToObject(o, "label", g_nodes[i].profile->label);
        cJSON_AddStringToObject(o, "topic_sensors", g_nodes[i].topic_sensors);
        cJSON_AddStringToObject(o, "topic_status", g_nodes[i].topic_status);
        cJSON_AddStringToObject(o, "topic_display", g_nodes[i].topic_display);
        cJSON_AddItemToArray(nodes, o);
    }
    cJSON_AddNumberToObject(root, "publish_interval_s", app_config_publish_interval_ms() / 1000.0);
    cJSON_AddStringToObject(root, "fw", esp_app_get_description()->version);

    // GET /api/history layout: series order, scales and tiers
    cJSON *hist = cJSON_AddObjectToObject(root, "history");
    cJSON *series = cJSON_AddArrayToObject(hist, "series");
    for (int s = 0; s < HIST_SERIES; s++) {
        cJSON *o = cJSON_CreateObject();
        cJSON_AddStringToObject(o, "key", s < METRIC_COUNT ? METRIC_INFO[s].key : "presence");
        cJSON_AddNumberToObject(o, "scale", history_scale(s));
        cJSON_AddItemToArray(series, o);
    }
    cJSON *tiers = cJSON_AddArrayToObject(hist, "tiers");
    const char *tier_names[HIST_TIER_COUNT] = {"10m", "1h"};
    for (int t = 0; t < HIST_TIER_COUNT; t++) {
        cJSON *o = cJSON_CreateObject();
        cJSON_AddStringToObject(o, "name", tier_names[t]);
        cJSON_AddNumberToObject(o, "slots", history_cap((hist_tier_id_t)t));
        cJSON_AddNumberToObject(o, "interval_s", t == HIST_FINE
                                                     ? app_config_publish_interval_ms() / 1000.0
                                                     : history_bucket_s());
        cJSON_AddItemToArray(tiers, o);
    }
    char *out = cJSON_PrintUnformatted(root);
    cJSON_Delete(root);
    return out;
}

// ---------------------------------------------------------------------------
// Emulator MQTT connection

void ctrl_publish_state(bool force)
{
    if (!s_connected || (!force && !s_state_dirty)) {
        return;
    }
    s_state_dirty = false;
    // One bounded buffer per publish (the JSON is ~3 KB); it grows only if the state outgrows it.
    static size_t cap = 4096;
    for (;;) {
        char *buf = malloc(cap);
        if (!buf) {
            ESP_LOGW(TAG, "no memory for the %u-byte state buffer", (unsigned)cap);
            return;
        }
        jw_t w;
        jw_init(&w, buf, cap, NULL, NULL);
        ctrl_write_state(&w);  // takes and releases its locks; none held from here on
        if (!w.err) {
            if (esp_mqtt_client_publish(s_client, TOPIC_EMULATOR_STATE, buf, (int)w.n, 0, 1) >= 0) {
                event_log_route(ROUTE_STATE, TOPIC_EMULATOR_STATE, w.n);
            }
            free(buf);
            return;
        }
        free(buf);
        if (cap >= 16384) {
            ESP_LOGE(TAG, "state JSON over %u bytes; not published", (unsigned)cap);
            return;
        }
        cap *= 2;
        ESP_LOGW(TAG, "state JSON outgrew its buffer; now %u bytes", (unsigned)cap);
    }
}

bool ctrl_publish_demo(const char *topic, const char *data, int len)
{
    if (!s_connected || esp_mqtt_client_publish(s_client, topic, data, len, 1, 1) < 0) {
        return false;
    }
    event_log_route(ROUTE_DEMO, topic, (size_t)len);
    return true;
}

bool ctrl_mqtt_connected(void)
{
    return s_connected;
}

static rx_kind_t rx_kind_for(const char *topic, int len)
{
    struct {
        const char *topic;
        rx_kind_t kind;
    } map[] = {
        {TOPIC_EMULATOR_CMD, RX_CMD},
        {TOPIC_AI_SUMMARY, RX_AI_SUMMARY},
        {TOPIC_AI_INFERENCE, RX_AI_INFERENCE},
    };
    for (size_t i = 0; i < sizeof(map) / sizeof(map[0]); i++) {
        if (len == (int)strlen(map[i].topic) && strncmp(topic, map[i].topic, len) == 0) {
            return map[i].kind;
        }
    }
    return RX_OTHER;
}

static void handle_cmd(const esp_mqtt_event_t *ev)
{
    if (ev->total_data_len == 0) {
        return;  // empty publish = clearing a retained command (-r -n); nothing to run
    }
    if (s_rx_retained) {
        // A retained command would re-run on every reconnect; refuse it.
        ESP_LOGW(TAG, "Ignoring retained message on %s (publish commands without -r)",
                 TOPIC_EMULATOR_CMD);
        return;
    }
    char msg[160];
    if (ev->total_data_len > CMD_MAX_LEN) {
        ctrl_execute(s_rx, CMD_MAX_LEN + 1, "mqtt", msg, sizeof(msg));  // rejected as too long
    } else {
        ctrl_execute(s_rx, s_rx_len, "mqtt", msg, sizeof(msg));
    }
}

static void on_ctrl_event(void *arg, esp_event_base_t base, int32_t id, void *data)
{
    esp_mqtt_event_handle_t ev = data;
    switch ((esp_mqtt_event_id_t)id) {
    case MQTT_EVENT_CONNECTED:
        s_connected = true;
        s_state_dirty = true;
        esp_mqtt_client_publish(ev->client, TOPIC_EMULATOR_STATUS, "online", 0, 1, 1);
        event_log_route(ROUTE_STATUS, TOPIC_EMULATOR_STATUS, 6);
        esp_mqtt_client_subscribe(ev->client, TOPIC_EMULATOR_CMD, 1);
        // Read-only: the agent's summary and thinking state, for the panel and route view.
        esp_mqtt_client_subscribe(ev->client, TOPIC_AI_SUMMARY, 1);
        esp_mqtt_client_subscribe(ev->client, TOPIC_AI_INFERENCE, 1);
        demo_engine_on_connected();  // republish the demo home if demo publishing is on
        ESP_LOGI(TAG, "emulator: connected; listening on %s and intellithings/ai/*",
                 TOPIC_EMULATOR_CMD);
        break;
    case MQTT_EVENT_DISCONNECTED:
        if (s_connected) {
            ESP_LOGW(TAG, "emulator: disconnected; will reconnect");
        }
        s_connected = false;
        break;
    case MQTT_EVENT_DATA: {
        // Large payloads arrive in chunks; only the first carries the topic.
        if (ev->current_data_offset == 0) {
            s_rx_len = 0;
            s_rx_retained = ev->retain;
            s_rx_kind = rx_kind_for(ev->topic, ev->topic_len);
        }
        size_t room = CMD_MAX_LEN - s_rx_len;
        size_t take = (size_t)ev->data_len < room ? (size_t)ev->data_len : room;
        memcpy(s_rx + s_rx_len, ev->data, take);
        s_rx_len += take;
        s_rx[s_rx_len] = '\0';
        if (ev->current_data_offset + ev->data_len < ev->total_data_len) {
            break;  // more chunks to come
        }
        switch (s_rx_kind) {
        case RX_CMD:
            handle_cmd(ev);
            break;
        case RX_AI_SUMMARY:
        case RX_AI_INFERENCE:
            event_log_route(ROUTE_AI_IN,
                            s_rx_kind == RX_AI_SUMMARY ? TOPIC_AI_SUMMARY : TOPIC_AI_INFERENCE,
                            (size_t)ev->total_data_len);
            ai_message_handle_ai_topic(
                s_rx_kind == RX_AI_SUMMARY ? AI_TOPIC_SUMMARY : AI_TOPIC_INFERENCE, s_rx,
                s_rx_len, s_rx_retained);
            break;
        default:
            break;
        }
        break;
    }
    default:
        break;
    }
}

void ctrl_init(void)
{
    s_cmd_lock = xSemaphoreCreateMutex();
    configASSERT(s_cmd_lock);
}

static void ctrl_config(esp_mqtt_client_config_t *cfg, const app_config_t *c, const char *uri)
{
    mqtt_fill_config(cfg, c, uri);
    cfg->credentials.client_id = "intellithings-emulator";
    cfg->session.last_will.topic = TOPIC_EMULATOR_STATUS;
    cfg->session.last_will.msg = "offline";
    cfg->session.last_will.qos = 1;
    cfg->session.last_will.retain = 1;
    cfg->buffer.out_size = 4096;
}

void ctrl_mqtt_restart(const app_config_t *c, const char *uri)
{
    esp_mqtt_client_config_t cfg;
    ctrl_config(&cfg, c, uri);
    if (!s_client) {
        s_client = esp_mqtt_client_init(&cfg);
        if (!s_client) {
            ESP_LOGE(TAG, "emulator: client init failed");
            return;
        }
        esp_mqtt_client_register_event(s_client, ESP_EVENT_ANY_ID, on_ctrl_event, NULL);
    } else if (esp_mqtt_set_config(s_client, &cfg) != ESP_OK) {
        ESP_LOGE(TAG, "emulator: applying the new settings failed");
    }
    esp_mqtt_client_start(s_client);
}

void ctrl_start(void)
{
    app_config_t c;
    app_config_get(&c);
    char uri[96];
    mqtt_broker_uri(uri, sizeof(uri));
    ctrl_mqtt_restart(&c, uri);
}

void ctrl_mqtt_stop(void)
{
    if (!s_client) {
        return;
    }
    if (s_connected) {
        // Leave the old broker cleanly (see mqtt_node.c reconf_task).
        esp_mqtt_client_publish(s_client, TOPIC_EMULATOR_STATUS, "offline", 0, 0, 1);
    }
    s_connected = false;
    esp_mqtt_client_stop(s_client);
    s_connected = false;
}
