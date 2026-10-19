#include "emulator_ctrl.h"

#include <stdio.h>
#include <string.h>

#include "ai_message.h"
#include "app_config.h"
#include "cJSON.h"
#include "esp_app_desc.h"
#include "esp_log.h"
#include "esp_system.h"
#include "esp_timer.h"
#include "event_log.h"
#include "freertos/FreeRTOS.h"
#include "freertos/semphr.h"
#include "history.h"
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

static struct {
    bool valid;
    bool ok;
    char msg[160];
    char source[8];
    int64_t at_us;
} s_last;

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
        snprintf(msg, msg_len, "missing \"action\" (scenario, set, clear, reset, display)");
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

static void add_reading(cJSON *obj, const sim_reading_t *r)
{
    for (int m = 0; m < METRIC_COUNT; m++) {
        cJSON_AddNumberToObject(obj, METRIC_INFO[m].key, r->v[m]);
    }
    cJSON_AddBoolToObject(obj, "presence", r->presence);
}

static void add_node_state(cJSON *arr, const vnode_t *n, int64_t now)
{
    const sim_state_t *s = &n->sim;
    cJSON *o = cJSON_CreateObject();
    cJSON_AddStringToObject(o, "id", n->id);
    cJSON_AddStringToObject(o, "room", n->profile->name);
    cJSON_AddStringToObject(o, "label", n->profile->label);
    cJSON_AddStringToObject(o, "mqtt", n->mqtt_connected ? "connected" : "disconnected");
    cJSON_AddNumberToObject(o, "mqtt_reconnects", n->mqtt_reconnects);
    cJSON_AddNumberToObject(o, "publish_count", n->publish_count);
    if (n->has_last) {
        cJSON_AddNumberToObject(o, "last_publish_age_s",
                                (double)((now - n->last_publish_us) / 1000000LL));
    } else {
        cJSON_AddNullToObject(o, "last_publish_age_s");
    }
    cJSON_AddStringToObject(o, "mode", sim_mode(s));
    cJSON_AddNumberToObject(o, "occupants", sim_occupants(s));

    if (s->scenario) {
        cJSON_AddStringToObject(o, "scenario", s->scenario->name);
        cJSON_AddStringToObject(o, "scenario_label", s->scenario->label);
        if (s->scenario_forever) {
            cJSON_AddNullToObject(o, "scenario_remaining_s");
        } else {
            cJSON_AddNumberToObject(o, "scenario_remaining_s", (int)s->scenario_left_s);
        }
    } else {
        cJSON_AddNullToObject(o, "scenario");
    }
    if (s->auto_event) {
        cJSON_AddStringToObject(o, "auto_event", s->auto_event->name);
        cJSON_AddNumberToObject(o, "auto_event_remaining_s", (int)s->auto_event_left_s);
    } else {
        cJSON_AddNullToObject(o, "auto_event");
    }

    cJSON *ovr = cJSON_AddObjectToObject(o, "overrides");
    bool any = false;
    for (int m = 0; m < METRIC_COUNT; m++) {
        if (s->ovr[m].active) {
            cJSON *r = cJSON_AddObjectToObject(ovr, METRIC_INFO[m].key);
            cJSON_AddNumberToObject(r, "min", s->ovr[m].min);
            cJSON_AddNumberToObject(r, "max", s->ovr[m].max);
            any = true;
        }
    }
    if (s->presence_ovr >= 0) {
        cJSON_AddBoolToObject(o, "presence_override", s->presence_ovr == 1);
        any = true;
    } else {
        cJSON_AddNullToObject(o, "presence_override");
    }
    if (any && !s->ovr_forever) {
        cJSON_AddNumberToObject(o, "override_remaining_s", (int)s->ovr_left_s);
    } else {
        cJSON_AddNullToObject(o, "override_remaining_s");
    }

    if (n->has_last) {
        add_reading(cJSON_AddObjectToObject(o, "reading"), &n->last);
    } else {
        cJSON_AddNullToObject(o, "reading");
    }
    cJSON_AddItemToArray(arr, o);
}

char *ctrl_state_json(void)
{
    int64_t now = esp_timer_get_time();
    char ip[16];
    wifi_get_ip(ip, sizeof(ip));
    char ts[40];
    bool synced = time_iso8601(ts, sizeof(ts));

    cJSON *root = cJSON_CreateObject();
    cJSON_AddStringToObject(root, "fw", esp_app_get_description()->version);
    cJSON_AddStringToObject(root, "hostname", CONFIG_EMU_HOSTNAME);
    cJSON_AddNumberToObject(root, "uptime_s", (double)(now / 1000000LL));
    cJSON_AddNumberToObject(root, "free_heap", esp_get_free_heap_size());
    cJSON_AddNumberToObject(root, "min_free_heap", esp_get_minimum_free_heap_size());

    cJSON *wifi = cJSON_AddObjectToObject(root, "wifi");
    cJSON_AddBoolToObject(wifi, "connected", wifi_is_connected());
    cJSON_AddStringToObject(wifi, "ssid", CONFIG_EMU_WIFI_SSID);
    cJSON_AddStringToObject(wifi, "ip", ip);
    cJSON_AddNumberToObject(wifi, "rssi", wifi_get_rssi());
    cJSON_AddNumberToObject(wifi, "reconnects", wifi_reconnect_count());

    cJSON *mqtt = cJSON_AddObjectToObject(root, "mqtt");
    char uri[96];
    mqtt_broker_uri(uri, sizeof(uri));
    cJSON_AddStringToObject(mqtt, "broker", uri);
    cJSON_AddBoolToObject(mqtt, "reconfiguring", mqtt_reconfiguring());
    cJSON_AddBoolToObject(mqtt, "emulator_connected", s_connected);

    cJSON *clock = cJSON_AddObjectToObject(root, "time");
    cJSON_AddBoolToObject(clock, "synced", synced);
    if (synced) {
        cJSON_AddStringToObject(clock, "now", ts);
    } else {
        cJSON_AddNullToObject(clock, "now");
    }
    cJSON_AddNumberToObject(clock, "local_hour", time_local_hour());

    cJSON_AddNumberToObject(root, "publish_interval_s", app_config_publish_interval_ms() / 1000.0);
    cJSON_AddNumberToObject(root, "sim_speed", CONFIG_EMU_SIM_SPEED);
    cJSON_AddBoolToObject(root, "config_saved", app_config_saved());

    cJSON *arr = cJSON_AddArrayToObject(root, "nodes");
    nodes_lock();
    for (int i = 0; i < NODE_COUNT; i++) {
        add_node_state(arr, &g_nodes[i], now);
    }
    nodes_unlock();

    xSemaphoreTake(s_cmd_lock, portMAX_DELAY);
    if (s_last.valid) {
        cJSON *lc = cJSON_AddObjectToObject(root, "last_cmd");
        cJSON_AddBoolToObject(lc, "ok", s_last.ok);
        cJSON_AddStringToObject(lc, "msg", s_last.msg);
        cJSON_AddStringToObject(lc, "source", s_last.source);
        cJSON_AddNumberToObject(lc, "age_s", (double)((now - s_last.at_us) / 1000000LL));
    } else {
        cJSON_AddNullToObject(root, "last_cmd");
    }
    xSemaphoreGive(s_cmd_lock);

    char *out = cJSON_PrintUnformatted(root);
    cJSON_Delete(root);
    return out;
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
    char *json = ctrl_state_json();
    if (json) {
        size_t len = strlen(json);
        if (esp_mqtt_client_publish(s_client, TOPIC_EMULATOR_STATE, json, (int)len, 0, 1) >= 0) {
            event_log_route(ROUTE_STATE, TOPIC_EMULATOR_STATE, len);
        }
        cJSON_free(json);
    }
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
