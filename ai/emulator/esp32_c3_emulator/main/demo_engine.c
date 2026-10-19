#include "demo_engine.h"

#include <math.h>
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "emulator_ctrl.h"
#include "esp_log.h"
#include "esp_random.h"
#include "esp_timer.h"
#include "freertos/FreeRTOS.h"
#include "freertos/semphr.h"
#include "freertos/task.h"
#include "payload.h"
#include "time_sync.h"
#include "virtual_node.h"

static const char *TAG = "demo";

#define TICK_MS 200
#define HOME_JSON_MAX 3072

typedef enum { DS_IDLE, DS_LOOPING, DS_ROLLING, DS_HOLD } dstate_t;
static const char *const STATE_NAMES[] = {"idle", "looping", "rolling", "hold"};
typedef enum { SRC_PRESETS, SRC_RANDOM, SRC_MIX } src_t;
static const char *const SRC_NAMES[] = {"presets", "random", "mix"};
static const char *const SPREAD_NAMES[] = {"normal", "uniform"};

static struct {
    SemaphoreHandle_t lock;
    demo_values_t cur, target;
    dstate_t state;
    bool publish, loop;
    uint16_t interval_s;
    uint8_t source, mix_pct;
    uint16_t roll_ms, hold_s;
    uint8_t spread, abnormal_pct;

    int preset;            // where cur came from: preset (0-based) or -1 for a roll
    uint32_t roll;         // roll number of cur, when preset == -1
    uint32_t rolls;        // rolls started so far
    uint32_t pending_roll;
    int64_t changed_us;
    int64_t next_us;       // LOOPING: next step; ROLLING: landing; HOLD: end of hold
    bool roll_from_loop, hold_locks;
    uint32_t seq;
    bool dirty_values, dirty_state, republish, off_pending;
    uint32_t rng;

    demo_hist_t hist[DEMO_HISTORY_LEN];
    uint32_t hist_n;
} D;

static void lock(void)
{
    xSemaphoreTake(D.lock, portMAX_DELAY);
}

static void unlock(void)
{
    xSemaphoreGive(D.lock);
}

// ---------------------------------------------------------------------------
// State changes (caller holds the lock)

static void push_history(int64_t now)
{
    demo_hist_t *h = &D.hist[D.hist_n % DEMO_HISTORY_LEN];
    h->seq = D.seq;
    h->t_s = (uint32_t)(now / 1000000LL);
    h->preset = (int8_t)D.preset;
    h->roll = D.roll;
    for (int e = 0; e < DEMO_ENTITY_COUNT; e++) {
        float p = powf(10.0f, DEMO_ENTITIES[e].dec);
        h->v[e] = (int16_t)lroundf(D.cur.v[e] * p);
    }
    D.hist_n++;
}

static void values_changed(int64_t now)
{
    D.seq++;
    D.changed_us = now;
    D.dirty_values = true;
    D.dirty_state = true;
    push_history(now);
}

static void apply_preset(int p, int64_t now)
{
    demo_fill_preset(&D.cur, p);
    D.preset = p;
    D.roll = 0;
    values_changed(now);
}

static void start_roll(bool from_loop, int64_t now)
{
    // Draw the result now, so every panel lands on the same values; publish on landing.
    demo_roll(&D.target, esp_random(), (demo_spread_t)D.spread, D.abnormal_pct);
    D.pending_roll = ++D.rolls;
    D.state = DS_ROLLING;
    D.next_us = now + (int64_t)D.roll_ms * 1000;
    D.roll_from_loop = from_loop;
    D.dirty_state = true;
}

static void loop_step(int64_t now)
{
    bool roll = D.source == SRC_RANDOM ||
                (D.source == SRC_MIX && (esp_random() % 100) < D.mix_pct);
    if (roll) {
        start_roll(true, now);
        return;
    }
    apply_preset(demo_pick_preset(&D.rng, D.preset), now);
    D.next_us = now + (int64_t)D.interval_s * 1000000;
}

static void tick(int64_t now)
{
    switch (D.state) {
    case DS_ROLLING:
        if (now >= D.next_us) {
            D.cur = D.target;
            D.preset = -1;
            D.roll = D.pending_roll;
            values_changed(now);
            if (D.roll_from_loop) {
                D.state = D.loop ? DS_LOOPING : DS_IDLE;
                D.next_us = now + (int64_t)D.interval_s * 1000000;
            } else {
                D.state = DS_HOLD;  // after the buzzer: hold, ignore presses, then resume
                D.next_us = now + (int64_t)D.hold_s * 1000000;
                D.hold_locks = true;
            }
        }
        break;
    case DS_HOLD:
        if (now >= D.next_us) {
            D.state = D.loop ? DS_LOOPING : DS_IDLE;
            D.next_us = now;  // the loop resumes straight away
            D.hold_locks = false;
            D.dirty_state = true;
        }
        break;
    case DS_LOOPING:
        if (now >= D.next_us) {
            loop_step(now);
        }
        break;
    default:
        break;
    }
}

static int left_s(int64_t until, int64_t now)
{
    return until > now ? (int)((until - now + 999999) / 1000000) : 0;
}

static int lock_left_s(int64_t now)
{
    if (D.state == DS_ROLLING && !D.roll_from_loop) {
        return left_s(D.next_us, now) + D.hold_s;
    }
    return (D.state == DS_HOLD && D.hold_locks) ? left_s(D.next_us, now) : 0;
}

// ---------------------------------------------------------------------------
// Publishing (no lock held: works on a copy)

typedef struct {
    demo_values_t v;
    dstate_t state;
    bool publish, loop;
    uint16_t interval_s;
    int preset;
    uint32_t roll, seq;
    int step_left, lock_left;
} pub_copy_t;

static int ap(char *buf, int n, int cap, const char *fmt, ...)
{
    if (n < 0 || n >= cap) {
        return -1;
    }
    va_list a;
    va_start(a, fmt);
    int k = vsnprintf(buf + n, cap - n, fmt, a);
    va_end(a);
    return (k < 0 || n + k >= cap) ? -1 : n + k;
}

static int ap_step(char *buf, int n, int cap, const pub_copy_t *c)
{
    if (c->preset >= 0) {
        const demo_preset_t *p = &DEMO_PRESETS[c->preset];
        return ap(buf, n, cap,
                  "\"source\":\"preset\",\"scenario_id\":%d,\"scenario\":\"%s\",\"group\":\"%s\","
                  "\"roll\":0",
                  c->preset + 1, p->name, DEMO_GROUPS[p->group]);
    }
    return ap(buf, n, cap,
              "\"source\":\"roll\",\"scenario_id\":null,\"scenario\":\"Random roll #%lu\","
              "\"group\":\"random\",\"roll\":%lu",
              (unsigned long)c->roll, (unsigned long)c->roll);
}

static void publish_state(const pub_copy_t *c, const char *ts)
{
    char buf[512];
    int n = ap(buf, 0, sizeof(buf), "{\"state\":\"%s\",\"publish\":%s,\"loop\":%s,\"interval_s\":%u,",
               STATE_NAMES[c->state], c->publish ? "true" : "false", c->loop ? "true" : "false",
               (unsigned)c->interval_s);
    n = ap_step(buf, n, sizeof(buf), c);
    n = ap(buf, n, sizeof(buf), ",\"seq\":%lu,\"step_left_s\":", (unsigned long)c->seq);
    n = c->state == DS_IDLE ? ap(buf, n, sizeof(buf), "null")
                            : ap(buf, n, sizeof(buf), "%d", c->step_left);
    n = ap(buf, n, sizeof(buf), ",\"lock_left_s\":%d", c->lock_left);
    if (ts) {
        n = ap(buf, n, sizeof(buf), ",\"ts\":\"%s\"", ts);
    }
    n = ap(buf, n, sizeof(buf), "}");
    if (n > 0) {
        ctrl_publish_demo(TOPIC_EMULATOR_DEMO "/state", buf, n);
    }
}

static void publish_home(const pub_copy_t *c, const char *ts)
{
    char *buf = malloc(HOME_JSON_MAX);
    if (!buf) {
        ESP_LOGE(TAG, "out of memory for the home payload");
        return;
    }
    int n = ap(buf, 0, HOME_JSON_MAX, "{\"seq\":%lu,", (unsigned long)c->seq);
    n = ap_step(buf, n, HOME_JSON_MAX, c);
    if (ts) {
        n = ap(buf, n, HOME_JSON_MAX, ",\"ts\":\"%s\"", ts);
    }
    n = ap(buf, n, HOME_JSON_MAX, ",\"areas\":{");
    for (int a = 0; a < DEMO_AREA_COUNT; a++) {
        n = ap(buf, n, HOME_JSON_MAX, "%s\"%s\":{", a ? "," : "", DEMO_AREAS[a].key);
        bool first = true;
        for (int e = 0; e < DEMO_ENTITY_COUNT; e++) {
            if (DEMO_ENTITIES[e].area != a) {
                continue;
            }
            char val[32];
            demo_json_value(e, c->v.v[e], val, sizeof(val));
            n = ap(buf, n, HOME_JSON_MAX, "%s\"%s\":%s", first ? "" : ",", DEMO_ENTITIES[e].field,
                   val);
            first = false;
        }
        n = ap(buf, n, HOME_JSON_MAX, "}");
    }
    n = ap(buf, n, HOME_JSON_MAX, "}}");
    if (n > 0) {
        ctrl_publish_demo(TOPIC_EMULATOR_DEMO "/home", buf, n);
    } else {
        ESP_LOGE(TAG, "home payload over %d bytes", HOME_JSON_MAX);
    }

    // Each node room also gets the node payload, so node display code can show demo data.
    for (int a = 0; a < DEMO_AREA_COUNT; a++) {
        sim_reading_t r;
        vnode_t *node = node_find(DEMO_AREAS[a].node_room);
        if (!node || !demo_node_reading(&c->v, a, &r)) {
            continue;
        }
        char topic[96];
        snprintf(topic, sizeof(topic), TOPIC_EMULATOR_DEMO "/%s/sensors", node->id);
        int len = payload_sensors_json(buf, HOME_JSON_MAX, node->id, &r, ts);
        if (len > 0) {
            ctrl_publish_demo(topic, buf, len);
        }
    }
    free(buf);
}

static void demo_task(void *arg)
{
    static pub_copy_t c;  // ~350 bytes; only this task uses it
    for (;;) {
        vTaskDelay(pdMS_TO_TICKS(TICK_MS));
        int64_t now = esp_timer_get_time();

        lock();
        tick(now);
        bool values = D.dirty_values, state = D.dirty_state, all = D.republish;
        bool off = D.off_pending;
        c.v = D.cur;
        c.state = D.state;
        c.publish = D.publish;
        c.loop = D.loop;
        c.interval_s = D.interval_s;
        c.preset = D.preset;
        c.roll = D.roll;
        c.seq = D.seq;
        c.step_left = left_s(D.next_us, now);
        c.lock_left = lock_left_s(now);
        bool online = ctrl_mqtt_connected();
        if (online || !c.publish) {
            // Offline while publishing: keep the flags; on_connected() republishes anyway.
            D.dirty_values = D.dirty_state = D.republish = D.off_pending = false;
        }
        unlock();

        if (!online) {
            continue;
        }
        char ts[40];
        bool has_ts = time_iso8601(ts, sizeof(ts));
        if (c.publish && (values || all)) {
            publish_home(&c, has_ts ? ts : NULL);
        }
        if ((c.publish && (state || values || all)) || off) {
            publish_state(&c, has_ts ? ts : NULL);  // "publish":false tells HA demo is off
        }
    }
}

// ---------------------------------------------------------------------------

void demo_engine_init(void)
{
    D.lock = xSemaphoreCreateMutex();
    configASSERT(D.lock);
    D.interval_s = 90;
    D.source = SRC_PRESETS;
    D.mix_pct = 30;
    D.roll_ms = 4000;
    D.hold_s = 120;
    D.spread = SPREAD_NORMAL;
    D.abnormal_pct = 15;
    D.rng = esp_random() | 1u;
    D.state = DS_IDLE;
    int64_t now = esp_timer_get_time();
    demo_fill_preset(&D.cur, 0);
    D.preset = 0;
    D.changed_us = now;
    push_history(now);
    BaseType_t ok = xTaskCreate(demo_task, "demo", 6144, NULL, 4, NULL);
    configASSERT(ok == pdPASS);
    ESP_LOGI(TAG, "%d areas, %d entities, %d presets; publishing off until turned on",
             DEMO_AREA_COUNT, DEMO_ENTITY_COUNT, DEMO_PRESET_COUNT);
}

void demo_engine_on_connected(void)
{
    lock();
    D.republish = D.publish;
    unlock();
}

// ---------------------------------------------------------------------------
// Commands

static bool get_bool(const cJSON *root, const char *key, int *out, char *msg, size_t len)
{
    const cJSON *it = cJSON_GetObjectItemCaseSensitive(root, key);
    if (!it) {
        return true;
    }
    if (!cJSON_IsBool(it)) {
        snprintf(msg, len, "demo: \"%s\" must be true or false", key);
        return false;
    }
    *out = cJSON_IsTrue(it) ? 1 : 0;
    return true;
}

static bool get_int(const cJSON *root, const char *key, int lo, int hi, int *out, char *msg,
                    size_t len)
{
    const cJSON *it = cJSON_GetObjectItemCaseSensitive(root, key);
    if (!it) {
        return true;
    }
    if (!cJSON_IsNumber(it) || it->valuedouble < lo || it->valuedouble > hi ||
        it->valuedouble != floor(it->valuedouble)) {
        snprintf(msg, len, "demo: \"%s\" must be a whole number %d..%d", key, lo, hi);
        return false;
    }
    *out = (int)it->valuedouble;
    return true;
}

static bool get_choice(const cJSON *root, const char *key, const char *const *names, int n,
                       int *out, char *msg, size_t len)
{
    const cJSON *it = cJSON_GetObjectItemCaseSensitive(root, key);
    if (!it) {
        return true;
    }
    for (int i = 0; cJSON_IsString(it) && i < n; i++) {
        if (strcmp(it->valuestring, names[i]) == 0) {
            *out = i;
            return true;
        }
    }
    int k = snprintf(msg, len, "demo: \"%s\" must be", key);
    for (int i = 0; i < n && k > 0 && (size_t)k < len; i++) {
        k += snprintf(msg + k, len - k, "%s \"%s\"", i ? (i == n - 1 ? " or" : ",") : "", names[i]);
    }
    return false;
}

static bool cmd_publish(const cJSON *root, int64_t now, char *msg, size_t len)
{
    int on = -1;
    if (!get_bool(root, "on", &on, msg, len)) {
        return false;
    }
    if (on < 0) {
        snprintf(msg, len, "demo: publish needs \"on\": true or false");
        return false;
    }
    if (on && !D.publish) {
        D.republish = true;
    } else if (!on && D.publish) {
        D.off_pending = true;
    }
    D.publish = on;
    snprintf(msg, len, on ? "demo: publishing to " TOPIC_EMULATOR_DEMO "/#"
                          : "demo: publishing stopped (retained values stay on the broker)");
    return true;
}

static bool cmd_loop(const cJSON *root, int64_t now, char *msg, size_t len)
{
    int on = -1, interval = D.interval_s, source = D.source, mix = D.mix_pct;
    if (!get_bool(root, "on", &on, msg, len) ||
        !get_int(root, "interval_s", 10, 3600, &interval, msg, len) ||
        !get_choice(root, "source", SRC_NAMES, 3, &source, msg, len) ||
        !get_int(root, "mix_pct", 0, 100, &mix, msg, len)) {
        return false;
    }
    D.interval_s = (uint16_t)interval;
    D.source = (uint8_t)source;
    D.mix_pct = (uint8_t)mix;
    if (on >= 0) {
        D.loop = on;
        if (on && D.state == DS_IDLE) {
            D.state = DS_LOOPING;
            D.next_us = now;  // first step straight away
        } else if (!on && D.state == DS_LOOPING) {
            D.state = DS_IDLE;
        }
    }
    D.dirty_state = true;
    snprintf(msg, len, "demo: loop %s, every %u s, %s%s", D.loop ? "on" : "off",
             (unsigned)D.interval_s, SRC_NAMES[D.source], D.source == SRC_MIX ? " (rolls" : "");
    if (D.source == SRC_MIX) {
        size_t k = strlen(msg);
        snprintf(msg + k, len - k, " %u %%)", (unsigned)D.mix_pct);
    }
    return true;
}

static bool cmd_roll(int64_t now, char *msg, size_t len)
{
    if (D.state == DS_ROLLING) {
        snprintf(msg, len, "demo: roll ignored, already rolling");
        return false;
    }
    int locked = lock_left_s(now);
    if (locked > 0) {
        snprintf(msg, len, "demo: roll ignored, locked for %d s (roll + hold)", locked);
        return false;
    }
    start_roll(false, now);
    snprintf(msg, len, "demo: rolling the house (%.1f s), then holding %u s", D.roll_ms / 1000.0,
             (unsigned)D.hold_s);
    return true;
}

static bool cmd_preset(const cJSON *root, int64_t now, char *msg, size_t len)
{
    const cJSON *id = cJSON_GetObjectItemCaseSensitive(root, "id");
    const cJSON *name = cJSON_GetObjectItemCaseSensitive(root, "name");
    int p = -1;
    if (cJSON_IsNumber(id) && id->valuedouble >= 1 && id->valuedouble <= DEMO_PRESET_COUNT &&
        id->valuedouble == floor(id->valuedouble)) {
        p = (int)id->valuedouble - 1;
    } else if (!id && cJSON_IsString(name)) {
        p = demo_preset_find(name->valuestring);
    }
    if (p < 0) {
        snprintf(msg, len, "demo: preset needs \"id\" 1..%d or an exact \"name\"", DEMO_PRESET_COUNT);
        return false;
    }
    if (D.state == DS_ROLLING) {
        snprintf(msg, len, "demo: preset ignored while rolling");
        return false;
    }
    apply_preset(p, now);
    if (D.hold_s) {
        D.state = DS_HOLD;
        D.next_us = now + (int64_t)D.hold_s * 1000000;
    } else {
        D.state = D.loop ? DS_LOOPING : DS_IDLE;
        D.next_us = now + (int64_t)D.interval_s * 1000000;
    }
    D.hold_locks = false;
    snprintf(msg, len, "demo: preset #%d %s, held %u s", p + 1, DEMO_PRESETS[p].name,
             (unsigned)D.hold_s);
    return true;
}

static bool cmd_settings(const cJSON *root, char *msg, size_t len)
{
    int roll_s = (D.roll_ms + 500) / 1000, hold = D.hold_s, spread = D.spread;
    int abnormal = D.abnormal_pct;
    if (!get_int(root, "roll_s", 1, 15, &roll_s, msg, len) ||
        !get_int(root, "hold_s", 0, 3600, &hold, msg, len) ||
        !get_choice(root, "spread", SPREAD_NAMES, 2, &spread, msg, len) ||
        !get_int(root, "abnormal_pct", 0, 100, &abnormal, msg, len)) {
        return false;
    }
    D.roll_ms = (uint16_t)(roll_s * 1000);
    D.hold_s = (uint16_t)hold;
    D.spread = (uint8_t)spread;
    D.abnormal_pct = (uint8_t)abnormal;
    D.dirty_state = true;
    snprintf(msg, len, "demo: roll %d s, hold %d s, spread %s, abnormal %d %%", roll_s, hold,
             SPREAD_NAMES[spread], abnormal);
    return true;
}

bool demo_engine_command(const cJSON *root, char *msg, size_t len)
{
    const cJSON *op = cJSON_GetObjectItemCaseSensitive(root, "op");
    if (!cJSON_IsString(op)) {
        snprintf(msg, len, "demo: missing \"op\" (publish, loop, roll, preset, settings)");
        return false;
    }
    int64_t now = esp_timer_get_time();
    const char *o = op->valuestring;
    bool ok;
    lock();
    if (strcmp(o, "publish") == 0) {
        ok = cmd_publish(root, now, msg, len);
    } else if (strcmp(o, "loop") == 0) {
        ok = cmd_loop(root, now, msg, len);
    } else if (strcmp(o, "roll") == 0) {
        ok = cmd_roll(now, msg, len);
    } else if (strcmp(o, "preset") == 0) {
        ok = cmd_preset(root, now, msg, len);
    } else if (strcmp(o, "settings") == 0) {
        ok = cmd_settings(root, msg, len);
    } else {
        snprintf(msg, len, "demo: unknown op \"%.24s\" (publish, loop, roll, preset, settings)", o);
        ok = false;
    }
    unlock();
    return ok;
}

// ---------------------------------------------------------------------------
// Read access for the web server

void demo_engine_snapshot(demo_snapshot_t *out)
{
    lock();
    out->seq = D.seq;
    out->values = D.cur;
    out->state = STATE_NAMES[D.state];
    out->publish = D.publish;
    out->loop = D.loop;
    out->preset = D.preset;
    out->roll = D.roll;
    out->changed_us = D.changed_us;
    unlock();
}

bool demo_engine_history(int i, demo_hist_t *out)
{
    bool ok = false;
    lock();
    uint32_t count = D.hist_n < DEMO_HISTORY_LEN ? D.hist_n : DEMO_HISTORY_LEN;
    if (i >= 0 && (uint32_t)i < count) {
        *out = D.hist[(D.hist_n - count + (uint32_t)i) % DEMO_HISTORY_LEN];
        ok = true;
    }
    unlock();
    return ok;
}

void demo_engine_add_state(cJSON *root)
{
    int64_t now = esp_timer_get_time();
    cJSON *d = cJSON_AddObjectToObject(root, "demo");
    lock();
    cJSON_AddStringToObject(d, "state", STATE_NAMES[D.state]);
    cJSON_AddBoolToObject(d, "publish", D.publish);
    cJSON_AddBoolToObject(d, "loop", D.loop);
    cJSON_AddNumberToObject(d, "interval_s", D.interval_s);
    cJSON_AddStringToObject(d, "loop_source", SRC_NAMES[D.source]);
    cJSON_AddNumberToObject(d, "mix_pct", D.mix_pct);
    cJSON_AddNumberToObject(d, "roll_s", D.roll_ms / 1000.0);
    cJSON_AddNumberToObject(d, "hold_s", D.hold_s);
    cJSON_AddStringToObject(d, "spread", SPREAD_NAMES[D.spread]);
    cJSON_AddNumberToObject(d, "abnormal_pct", D.abnormal_pct);
    cJSON_AddNumberToObject(d, "seq", D.seq);
    cJSON *step = cJSON_AddObjectToObject(d, "step");
    if (D.preset >= 0) {
        cJSON_AddStringToObject(step, "source", "preset");
        cJSON_AddNumberToObject(step, "scenario_id", D.preset + 1);
        cJSON_AddStringToObject(step, "scenario", DEMO_PRESETS[D.preset].name);
        cJSON_AddStringToObject(step, "group", DEMO_GROUPS[DEMO_PRESETS[D.preset].group]);
    } else {
        char name[32];
        snprintf(name, sizeof(name), "Random roll #%lu", (unsigned long)D.roll);
        cJSON_AddStringToObject(step, "source", "roll");
        cJSON_AddNullToObject(step, "scenario_id");
        cJSON_AddStringToObject(step, "scenario", name);
        cJSON_AddStringToObject(step, "group", "random");
    }
    cJSON_AddNumberToObject(step, "roll", D.roll);
    if (D.state == DS_IDLE) {
        cJSON_AddNullToObject(d, "step_left_s");
    } else {
        cJSON_AddNumberToObject(d, "step_left_s", left_s(D.next_us, now));
    }
    cJSON_AddNumberToObject(d, "lock_left_s", lock_left_s(now));
    cJSON_AddNumberToObject(d, "roll_left_ms",
                            D.state == DS_ROLLING && D.next_us > now ? (double)((D.next_us - now) / 1000) : 0);
    cJSON_AddBoolToObject(d, "roll_from_loop", D.state == DS_ROLLING && D.roll_from_loop);
    unlock();
}
