#include "engine.h"

#include <math.h>
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "button.h"
#include "demo_home.h"
#include "esp_log.h"
#include "esp_random.h"
#include "esp_timer.h"
#include "event_log.h"
#include "freertos/FreeRTOS.h"
#include "freertos/semphr.h"
#include "freertos/task.h"
#include "library.h"
#include "mqtt_link.h"
#include "settings.h"
#include "time_sync.h"

static const char *TAG = "engine";

#define TICK_MS 100
#define AREA_JSON_MAX 1024

typedef enum { DS_IDLE, DS_LOOPING, DS_ROLLING, DS_HOLD } dstate_t;
static const char *const STATE_NAMES[] = {"idle", "looping", "rolling", "hold"};
static const char *const SRC_NAMES[] = {"presets", "random", "mix"};
static const char *const SPREAD_NAMES[] = {"normal", "uniform"};
static const char *const PRESS_NAMES[] = {"buzzer", "web", "key", "admin"};
static const char *const RESULT_NAMES[] = {"rolling", "busy", "locked"};  // press_result_t

static struct {
    SemaphoreHandle_t lock;
    demo_values_t cur, target;
    dstate_t state;
    bool publish, loop;
    uint16_t interval_s;
    uint8_t source, mix_pct;
    uint16_t roll_ms, hold_s;
    uint8_t spread, abnormal_pct;

    // Where cur came from: a scenario (name, group) or a roll (roll > 0).
    int scenario;  // library index when it was applied; -1 for a roll
    char name[DEMO_NAME_MAX + 1];
    uint8_t group;
    uint32_t roll;
    char expect[DEMO_EXPECT_MAX + 1];  // the scenario's expected AI response ("" for a roll)

    uint32_t rolls, pending_roll, steps;
    uint32_t presses[PRESS_SOURCES];  // since boot
    uint32_t ignored;                 // presses during a roll or the hold
    int64_t changed_us;
    int64_t next_us;  // LOOPING: next step; ROLLING: landing; HOLD: end of hold
    bool roll_from_loop;
    uint32_t seq;
    bool dirty_values, dirty_state, republish;
    uint32_t rng;
    uint32_t published;  // area snapshots sent
    uint32_t doorbells;  // doorbell presses since boot
    uint32_t doorbell_pending;  // press number to publish (0 = none)
    uint32_t press_n;           // presses since boot, any source, for the pages' feedback
    uint8_t press_src, press_result;
    uint16_t press_lock_left;
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

static void values_changed(int64_t now)
{
    D.seq++;
    D.changed_us = now;
    D.dirty_values = true;
    D.dirty_state = true;
}

static bool apply_scenario(int i, int64_t now)
{
    demo_scenario_t s;
    if (!library_get(i, &s)) {
        return false;
    }
    demo_fill_scenario(&D.cur, &s);
    D.scenario = i;
    strlcpy(D.name, s.name, sizeof(D.name));
    strlcpy(D.expect, s.expect, sizeof(D.expect));
    D.group = s.group;
    D.roll = 0;
    D.steps++;
    values_changed(now);
    if (s.flags & SCEN_DOORBELL) {
        D.doorbell_pending = ++D.doorbells;
    }
    return true;
}

static void start_roll(bool from_loop, int64_t now)
{
    // Draw the result now, so every page lands on the same values; publish on landing.
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
    int p = library_pick(&D.rng, D.scenario);
    if (p >= 0) {
        apply_scenario(p, now);
    }
    D.next_us = now + (int64_t)D.interval_s * 1000000;
}

static void tick(int64_t now)
{
    switch (D.state) {
    case DS_ROLLING:
        if (now >= D.next_us) {
            D.cur = D.target;
            D.scenario = -1;
            D.roll = D.pending_roll;
            snprintf(D.name, sizeof(D.name), "Random roll #%lu", (unsigned long)D.roll);
            D.expect[0] = '\0';
            values_changed(now);
            if (D.roll_from_loop) {
                D.state = D.loop ? DS_LOOPING : DS_IDLE;
                D.next_us = now + (int64_t)D.interval_s * 1000000;
            } else {
                D.state = DS_HOLD;  // after the buzzer: hold, ignore presses, then resume
                D.next_us = now + (int64_t)D.hold_s * 1000000;
            }
        }
        break;
    case DS_HOLD:
        if (now >= D.next_us) {
            D.state = D.loop ? DS_LOOPING : DS_IDLE;
            D.next_us = now;  // the loop resumes straight away
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
    return D.state == DS_HOLD ? left_s(D.next_us, now) : 0;
}

// Saves the demo fields to settings, so they're the state at the next boot.
static void save_demo_settings(void)
{
    dm_settings_t s;
    settings_get(&s);
    s.publish = D.publish;
    s.loop = D.loop;
    s.interval_s = D.interval_s;
    s.source = D.source;
    s.mix_pct = D.mix_pct;
    s.roll_ms = D.roll_ms;
    s.hold_s = D.hold_s;
    s.spread = D.spread;
    s.abnormal_pct = D.abnormal_pct;
    char msg[96];
    if (!settings_save(&s, msg, sizeof(msg))) {
        ESP_LOGW(TAG, "Demo settings not saved: %s", msg);
    }
}

// ---------------------------------------------------------------------------
// Publishing (no lock held: works on a copy)

// One retained message per area on <prefix>/<area key> (ai/docs/demo-device-inventory.md).
static void publish_areas(const demo_values_t *v, const char *ts)
{
    char *buf = malloc(AREA_JSON_MAX);
    if (!buf) {
        ESP_LOGE(TAG, "out of memory for the area payloads");
        return;
    }
    for (int a = 0; a < DEMO_AREA_COUNT; a++) {
        int len = demo_area_json(v, a, ts, buf, AREA_JSON_MAX);
        if (len > 0) {
            mqtt_link_publish(DEMO_AREAS[a].key, buf, len, true);
        } else {
            ESP_LOGE(TAG, "%s payload over %d bytes", DEMO_AREAS[a].key, AREA_JSON_MAX);
        }
    }
    free(buf);
}

// <prefix>/home/doorbell: an event, not retained. event_id stays unique across reboots.
static void publish_doorbell(uint32_t press, const char *ts)
{
    char buf[160];
    int n = snprintf(buf, sizeof(buf),
                     "{\"event\":\"doorbell_pressed\",\"event_id\":\"boot-%lu-press-%lu\"",
                     (unsigned long)settings_boot_count(), (unsigned long)press);
    if (ts) {
        n += snprintf(buf + n, sizeof(buf) - n, ",\"ts\":\"%s\"", ts);
    }
    n += snprintf(buf + n, sizeof(buf) - n, "}");
    mqtt_link_publish("home/doorbell", buf, n, false);
}

static void engine_task(void *arg)
{
    static demo_values_t v;  // only this task uses it
    bool led = false;
    for (;;) {
        vTaskDelay(pdMS_TO_TICKS(TICK_MS));
        int64_t now = esp_timer_get_time();

        lock();
        uint32_t seq_before = D.seq;
        tick(now);
        bool landed = D.seq != seq_before && D.scenario < 0;
        bool values = D.dirty_values, all = D.republish;
        bool publish = D.publish, rolling = D.state == DS_ROLLING;
        uint32_t ring = D.doorbell_pending;
        D.doorbell_pending = 0;
        v = D.cur;
        bool online = mqtt_link_connected();
        if (online || !publish) {
            // Offline while publishing: keep the flags; on_connected() republishes anyway.
            D.dirty_values = D.republish = false;
        }
        if (landed) {
            event_log("Roll #%lu landed", (unsigned long)D.roll);
        }
        unlock();

        // The LED blinks while the house is rolling.
        bool want = rolling && (now / 150000) % 2;
        if (want != led) {
            led = want;
            button_led_set(led);
        }

        char ts[40];
        bool has_ts = time_iso8601(ts, sizeof(ts));
        if (ring) {
            if (online && publish) {
                publish_doorbell(ring, has_ts ? ts : NULL);
            } else {
                event_log("Doorbell #%lu not sent (%s)", (unsigned long)ring,
                          publish ? "MQTT offline" : "publishing off");
            }
        }
        if (online && publish && (values || all)) {
            publish_areas(&v, has_ts ? ts : NULL);
            D.published++;
        }
    }
}

// ---------------------------------------------------------------------------

void engine_init(void)
{
    D.lock = xSemaphoreCreateMutex();
    configASSERT(D.lock);
    dm_settings_t s;
    settings_get(&s);
    D.publish = s.publish;
    D.loop = s.loop;
    D.interval_s = s.interval_s;
    D.source = s.source;
    D.mix_pct = s.mix_pct;
    D.roll_ms = s.roll_ms;
    D.hold_s = s.hold_s;
    D.spread = s.spread;
    D.abnormal_pct = s.abnormal_pct;
    D.rng = esp_random() | 1u;
    int64_t now = esp_timer_get_time();
    D.scenario = -1;
    int first = library_pick(&D.rng, -1);
    if (first >= 0) {
        apply_scenario(first, now);
        D.steps = 0;
        D.doorbells = D.doorbell_pending = 0;  // the boot scenario never rings
    }
    D.state = D.loop ? DS_LOOPING : DS_IDLE;
    D.next_us = now + (int64_t)D.interval_s * 1000000;
    BaseType_t ok = xTaskCreate(engine_task, "engine", 6144, NULL, 4, NULL);
    configASSERT(ok == pdPASS);
    event_log("Demo ready: %d scenarios, loop %s, publishing %s", library_count(),
              D.loop ? "on" : "off", D.publish ? "on" : "off");
}

void engine_on_connected(void)
{
    lock();
    D.republish = D.publish;
    unlock();
}

void engine_republish(void)
{
    engine_on_connected();
}

press_result_t engine_press(press_src_t src, int *lock_left)
{
    int64_t now = esp_timer_get_time();
    press_result_t r;
    lock();
    D.presses[src]++;
    int left = lock_left_s(now);
    if (D.state == DS_ROLLING) {
        r = PRESS_BUSY;
        D.ignored++;
    } else if (left > 0 && src != PRESS_ADMIN) {
        r = PRESS_LOCKED;
        D.ignored++;
    } else {
        start_roll(false, now);
        r = PRESS_ROLLING;
    }
    uint32_t roll = D.pending_roll;
    D.press_n++;  // the pages show feedback for presses they didn't make themselves
    D.press_src = (uint8_t)src;
    D.press_result = (uint8_t)r;
    D.press_lock_left = (uint16_t)left;
    unlock();
    if (lock_left) {
        *lock_left = left;
    }
    if (src != PRESS_ADMIN) {
        settings_count_press();
    }
    if (r == PRESS_ROLLING) {
        event_log("Press (%s): rolling the house, roll #%lu", PRESS_NAMES[src], (unsigned long)roll);
    } else {
        event_log("Press (%s) ignored: %s", PRESS_NAMES[src],
                  r == PRESS_BUSY ? "already rolling" : "locked after a roll");
    }
    return r;
}

// ---------------------------------------------------------------------------
// Admin commands

static bool get_bool(const cJSON *root, const char *key, int *out, char *msg, size_t len)
{
    const cJSON *it = cJSON_GetObjectItemCaseSensitive(root, key);
    if (!it) {
        return true;
    }
    if (!cJSON_IsBool(it)) {
        snprintf(msg, len, "\"%s\" must be true or false", key);
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
        snprintf(msg, len, "\"%s\" must be a whole number %d..%d", key, lo, hi);
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
    snprintf(msg, len, "unknown \"%s\"", key);
    return false;
}

static bool cmd_publish(const cJSON *root, char *msg, size_t len)
{
    int on = -1;
    if (!get_bool(root, "on", &on, msg, len) || on < 0) {
        snprintf(msg, len, "publish needs \"on\": true or false");
        return false;
    }
    if (on && !D.publish) {
        D.republish = true;
    }
    D.publish = on;
    snprintf(msg, len, on ? "publishing the demo home"
                          : "publishing stopped (retained values stay on the broker)");
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
            D.next_us = now + (int64_t)D.interval_s * 1000000;
        } else if (!on && D.state == DS_LOOPING) {
            D.state = DS_IDLE;
        }
    }
    D.dirty_state = true;
    snprintf(msg, len, "loop %s, every %u s, %s", D.loop ? "on" : "off", (unsigned)D.interval_s,
             SRC_NAMES[D.source]);
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
    snprintf(msg, len, "roll %d s, hold %d s, spread %s, abnormal %d %%", roll_s, hold,
             SPREAD_NAMES[spread], abnormal);
    return true;
}

static bool cmd_play(const cJSON *root, int64_t now, char *msg, size_t len)
{
    const cJSON *id = cJSON_GetObjectItemCaseSensitive(root, "id");
    int n = library_count();
    if (!cJSON_IsNumber(id) || id->valuedouble < 1 || id->valuedouble > n ||
        id->valuedouble != floor(id->valuedouble)) {
        snprintf(msg, len, "play needs \"id\" 1..%d", n);
        return false;
    }
    if (D.state == DS_ROLLING) {
        snprintf(msg, len, "wait for the roll to land");
        return false;
    }
    apply_scenario((int)id->valuedouble - 1, now);
    // Held like a roll, so the AI has time to react; then the loop goes on.
    D.state = D.hold_s ? DS_HOLD : (D.loop ? DS_LOOPING : DS_IDLE);
    D.next_us = now + (int64_t)(D.hold_s ? D.hold_s : D.interval_s) * 1000000;
    snprintf(msg, len, "playing #%d \"%s\"", (int)id->valuedouble, D.name);
    return true;
}

bool engine_command(const cJSON *root, char *msg, size_t len)
{
    const cJSON *op = cJSON_GetObjectItemCaseSensitive(root, "op");
    if (!cJSON_IsString(op)) {
        snprintf(msg, len, "missing \"op\"");
        return false;
    }
    const char *o = op->valuestring;
    if (strcmp(o, "roll") == 0) {
        int left = 0;
        press_result_t r = engine_press(PRESS_ADMIN, &left);
        snprintf(msg, len, r == PRESS_ROLLING ? "rolling the house" : "already rolling");
        return r == PRESS_ROLLING;
    }
    int64_t now = esp_timer_get_time();
    bool ok, persist = false;
    lock();
    if (strcmp(o, "publish") == 0) {
        ok = persist = cmd_publish(root, msg, len);
    } else if (strcmp(o, "loop") == 0) {
        ok = persist = cmd_loop(root, now, msg, len);
    } else if (strcmp(o, "settings") == 0) {
        ok = persist = cmd_settings(root, msg, len);
    } else if (strcmp(o, "play") == 0) {
        ok = cmd_play(root, now, msg, len);
    } else if (strcmp(o, "next") == 0) {
        ok = D.state != DS_ROLLING;
        if (ok) {
            D.state = D.loop ? DS_LOOPING : DS_IDLE;
            loop_step(now);
            if (!D.loop && D.state != DS_ROLLING) {
                D.state = DS_IDLE;
            }
            snprintf(msg, len, "next step");
        } else {
            snprintf(msg, len, "wait for the roll to land");
        }
    } else if (strcmp(o, "doorbell") == 0) {
        D.doorbell_pending = ++D.doorbells;
        ok = true;
        snprintf(msg, len, "doorbell #%lu", (unsigned long)D.doorbells);
    } else if (strcmp(o, "republish") == 0) {
        D.republish = D.publish;
        ok = true;
        snprintf(msg, len, D.publish ? "republishing" : "publishing is off");
    } else if (strcmp(o, "reset_counts") == 0) {
        memset(D.presses, 0, sizeof(D.presses));
        D.ignored = 0;
        ok = true;
        snprintf(msg, len, "press counters reset");
    } else {
        snprintf(msg, len, "unknown op \"%.24s\"", o);
        ok = false;
    }
    unlock();
    if (persist) {
        save_demo_settings();
    }
    if (ok && strcmp(o, "reset_counts") == 0) {
        settings_reset_presses();
    }
    if (ok) {
        event_log("Admin: %s", msg);
    }
    return ok;
}

// ---------------------------------------------------------------------------
// Read access for the web server

void engine_write_state(jw_t *w, uint32_t since)
{
    // Copied under the lock, written after it (the writer may be sending to a socket).
    int64_t now = esp_timer_get_time();
    demo_values_t *vals = malloc(sizeof(demo_values_t));
    lock();
    dstate_t state = D.state;
    bool publish = D.publish, loop = D.loop, from_loop = D.roll_from_loop;
    uint32_t seq = D.seq, roll = D.roll;
    int scenario = D.scenario;
    char name[DEMO_NAME_MAX + 1];
    strlcpy(name, D.name, sizeof(name));
    uint8_t group = D.group;
    int step_left = left_s(D.next_us, now), lock_left = lock_left_s(now);
    int roll_left_ms = state == DS_ROLLING && D.next_us > now ? (int)((D.next_us - now) / 1000) : 0;
    uint16_t roll_ms = D.roll_ms, interval_s = D.interval_s, hold_s = D.hold_s;
    uint32_t doorbells = D.doorbells;
    uint32_t press_n = D.press_n;
    uint8_t press_src = D.press_src, press_result = D.press_result;
    uint16_t press_lock_left = D.press_lock_left;
    bool with_values = seq != since && vals;
    if (with_values) {
        *vals = D.cur;
    }
    unlock();

    dm_settings_t s;
    settings_get(&s);
    jw_raw(w, "{\"seq\":");
    jw_num(w, seq);
    jw_key(w, false, "state");
    jw_str(w, STATE_NAMES[state]);
    jw_key(w, false, "publish");
    jw_raw(w, jbool(publish));
    jw_key(w, false, "loop");
    jw_raw(w, jbool(loop));
    jw_key(w, false, "scenario");
    jw_str(w, name);
    jw_key(w, false, "scenario_id");
    if (roll == 0 && scenario >= 0) {
        jw_num(w, scenario + 1);
    } else {
        jw_raw(w, "null");
    }
    jw_key(w, false, "group");
    jw_str(w, roll ? "random" : DEMO_GROUPS[group]);
    jw_key(w, false, "group_label");
    jw_str(w, roll ? "Random roll" : DEMO_GROUP_LABELS[group]);
    jw_key(w, false, "roll");
    jw_num(w, roll);
    jw_key(w, false, "step_left_s");
    if (state == DS_IDLE) {
        jw_raw(w, "null");
    } else {
        jw_num(w, step_left);
    }
    jw_key(w, false, "lock_left_s");
    jw_num(w, lock_left);
    jw_key(w, false, "roll_left_ms");
    jw_num(w, roll_left_ms);
    jw_key(w, false, "roll_ms");
    jw_num(w, roll_ms);
    jw_key(w, false, "roll_from_loop");
    jw_raw(w, jbool(state == DS_ROLLING && from_loop));
    jw_key(w, false, "interval_s");
    jw_num(w, interval_s);
    jw_key(w, false, "hold_s");
    jw_num(w, hold_s);
    jw_key(w, false, "presses");
    jw_num(w, settings_presses());
    jw_key(w, false, "doorbells");
    jw_num(w, doorbells);
    jw_raw(w, ",\"press\":{\"n\":");
    jw_num(w, press_n);
    jw_key(w, false, "src");
    jw_str(w, PRESS_NAMES[press_src]);
    jw_key(w, false, "result");
    jw_str(w, RESULT_NAMES[press_result]);
    jw_key(w, false, "lock_left_s");
    jw_num(w, press_lock_left);
    jw_raw(w, "}");
    jw_key(w, false, "buzzer_held");
    jw_raw(w, jbool(button_is_held()));
    jw_key(w, false, "mqtt");
    jw_raw(w, jbool(mqtt_link_connected()));
    jw_raw(w, ",\"look\":{\"theme\":");
    jw_str(w, s.theme ? "light" : "dark");
    jw_key(w, false, "animation");
    jw_raw(w, jbool(s.animation));
    jw_key(w, false, "fahrenheit");
    jw_raw(w, jbool(s.fahrenheit));
    jw_raw(w, "},\"lib\":");
    jw_num(w, library_version());
    if (with_values) {
        jw_raw(w, ",\"v\":[");
        for (int e = 0; e < DEMO_ENTITY_COUNT; e++) {
            if (e) {
                jw_raw(w, ",");
            }
            if (DEMO_ENTITIES[e].type == DEMO_NUM) {
                jw_fixed(w, vals->v[e], DEMO_ENTITIES[e].dec);
            } else {
                jw_num(w, (int)vals->v[e]);
            }
        }
        jw_raw(w, "]");
    }
    jw_raw(w, "}");
    free(vals);
}

void engine_write_admin(jw_t *w)
{
    lock();
    uint32_t presses[PRESS_SOURCES];
    memcpy(presses, D.presses, sizeof(presses));
    uint32_t ignored = D.ignored, rolls = D.rolls, steps = D.steps, published = D.published;
    uint32_t doorbells = D.doorbells;
    int64_t changed = D.changed_us;
    uint8_t source = D.source, mix = D.mix_pct, spread = D.spread, abnormal = D.abnormal_pct;
    char expect[DEMO_EXPECT_MAX + 1];
    strlcpy(expect, D.expect, sizeof(expect));
    unlock();
    jw_raw(w, "{\"expect\":");
    jw_str(w, expect);
    jw_raw(w, ",\"presses_total\":");
    jw_num(w, settings_presses());
    jw_raw(w, ",\"presses\":{");
    for (int i = 0; i < PRESS_SOURCES; i++) {
        jw_key(w, i == 0, PRESS_NAMES[i]);
        jw_num(w, presses[i]);
    }
    jw_raw(w, "},\"ignored\":");
    jw_num(w, ignored);
    jw_key(w, false, "rolls");
    jw_num(w, rolls);
    jw_key(w, false, "steps");
    jw_num(w, steps);
    jw_key(w, false, "published");
    jw_num(w, published);
    jw_key(w, false, "doorbells");
    jw_num(w, doorbells);
    jw_key(w, false, "changed_ago_s");
    jw_num(w, (double)((esp_timer_get_time() - changed) / 1000000LL));
    jw_key(w, false, "source");
    jw_str(w, SRC_NAMES[source]);
    jw_key(w, false, "mix_pct");
    jw_num(w, mix);
    jw_key(w, false, "spread");
    jw_str(w, SPREAD_NAMES[spread]);
    jw_key(w, false, "abnormal_pct");
    jw_num(w, abnormal);
    jw_raw(w, "}");
}
