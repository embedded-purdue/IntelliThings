#include "library.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "esp_log.h"
#include "esp_partition.h"
#include "esp_random.h"
#include "esp_rom_crc.h"
#include "freertos/FreeRTOS.h"
#include "freertos/semphr.h"

static const char *TAG = "library";

#define PART_LABEL "demolib"
#define MAGIC 0x314C4D44u  // "DML1"
#define FORMAT 3  // 2: per-area demo home (Nov 15), scenario flags; 3: expected AI response

typedef struct {
    uint32_t magic;
    uint16_t format;
    uint16_t scenario_size;  // sizeof(demo_scenario_t) when written
    uint32_t schema;         // demo_schema_hash() when written
    uint16_t n_entities;
    uint16_t n_scenarios;
    uint32_t len;  // payload bytes after this header
    uint32_t crc;  // CRC-32 of the payload
} hdr_t;

static demo_scenario_t s_lib[LIB_MAX];
static int s_n;
static uint32_t s_version = 1;
static bool s_factory = true;
static SemaphoreHandle_t s_lock;
static const esp_partition_t *s_part;

static void lock(void)
{
    xSemaphoreTake(s_lock, portMAX_DELAY);
}

static void unlock(void)
{
    xSemaphoreGive(s_lock);
}

static void load_builtin(void)
{
    s_n = DEMO_BUILTIN_COUNT < LIB_MAX ? DEMO_BUILTIN_COUNT : LIB_MAX;
    for (int i = 0; i < s_n; i++) {
        demo_scenario_from_builtin(&s_lib[i], i);
    }
}

// Reads the saved library into s_lib and the limits. False (nothing changed) if there's none.
static bool load_saved(void)
{
    hdr_t h;
    if (!s_part || esp_partition_read(s_part, 0, &h, sizeof(h)) != ESP_OK || h.magic != MAGIC) {
        return false;
    }
    if (h.format != FORMAT || h.scenario_size != sizeof(demo_scenario_t) ||
        h.schema != demo_schema_hash() || h.n_entities != DEMO_ENTITY_COUNT ||
        h.n_scenarios > LIB_MAX ||
        h.len != h.n_entities * sizeof(demo_limit_t) + h.n_scenarios * sizeof(demo_scenario_t) ||
        sizeof(h) + h.len > s_part->size) {
        ESP_LOGW(TAG, "Saved library is for another demo home or firmware; using the built-in one");
        return false;
    }
    uint8_t *buf = malloc(h.len);
    if (!buf) {
        ESP_LOGE(TAG, "No memory to load the saved library");
        return false;
    }
    bool ok = esp_partition_read(s_part, sizeof(h), buf, h.len) == ESP_OK &&
              esp_rom_crc32_le(0, buf, h.len) == h.crc;
    if (!ok) {
        ESP_LOGW(TAG, "Saved library is damaged (CRC); using the built-in one");
        free(buf);
        return false;
    }
    const demo_limit_t *lim = (const demo_limit_t *)buf;
    char msg[96];
    for (int e = 0; e < DEMO_ENTITY_COUNT; e++) {
        demo_limit_t l = lim[e];
        if (DEMO_ENTITIES[e].type != DEMO_ENUM && !demo_limit_set(e, &l, msg, sizeof(msg))) {
            ESP_LOGW(TAG, "Saved limit ignored: %s", msg);
        }
    }
    memcpy(s_lib, buf + h.n_entities * sizeof(demo_limit_t), h.n_scenarios * sizeof(demo_scenario_t));
    s_n = h.n_scenarios;
    for (int i = 0; i < s_n; i++) {
        s_lib[i].name[DEMO_NAME_MAX] = '\0';
        s_lib[i].expect[DEMO_EXPECT_MAX] = '\0';
        if (s_lib[i].n_sets > DEMO_MAX_SETS) {
            s_lib[i].n_sets = DEMO_MAX_SETS;
        }
        if (s_lib[i].group >= DEMO_GROUP_COUNT) {
            s_lib[i].group = 0;
        }
    }
    free(buf);
    return true;
}

// Writes the current library and limits (caller holds the lock).
static bool save(char *msg, size_t len)
{
    if (!s_part) {
        snprintf(msg, len, "no \"" PART_LABEL "\" partition: changes last until a reboot");
        return false;
    }
    size_t lim_len = DEMO_ENTITY_COUNT * sizeof(demo_limit_t);
    size_t payload = lim_len + s_n * sizeof(demo_scenario_t);
    uint8_t *buf = malloc(sizeof(hdr_t) + payload);
    if (!buf) {
        snprintf(msg, len, "out of memory, not saved");
        return false;
    }
    uint8_t *p = buf + sizeof(hdr_t);
    for (int e = 0; e < DEMO_ENTITY_COUNT; e++) {
        memcpy(p + e * sizeof(demo_limit_t), demo_limit(e), sizeof(demo_limit_t));
    }
    memcpy(p + lim_len, s_lib, s_n * sizeof(demo_scenario_t));
    hdr_t h = {
        .magic = MAGIC,
        .format = FORMAT,
        .scenario_size = sizeof(demo_scenario_t),
        .schema = demo_schema_hash(),
        .n_entities = (uint16_t)DEMO_ENTITY_COUNT,
        .n_scenarios = (uint16_t)s_n,
        .len = (uint32_t)payload,
        .crc = esp_rom_crc32_le(0, p, payload),
    };
    memcpy(buf, &h, sizeof(h));
    size_t total = sizeof(h) + payload;
    size_t erase = (total + 4095) & ~(size_t)4095;
    esp_err_t err = esp_partition_erase_range(s_part, 0, erase);
    if (err == ESP_OK) {
        err = esp_partition_write(s_part, 0, buf, total);
    }
    free(buf);
    if (err != ESP_OK) {
        snprintf(msg, len, "couldn't save to flash (%s)", esp_err_to_name(err));
        return false;
    }
    return true;
}

static void changed(void)
{
    s_version++;
    s_factory = false;
}

void library_init(void)
{
    s_lock = xSemaphoreCreateMutex();
    configASSERT(s_lock);
    s_part = esp_partition_find_first(ESP_PARTITION_TYPE_DATA, ESP_PARTITION_SUBTYPE_ANY, PART_LABEL);
    if (!s_part) {
        ESP_LOGE(TAG, "No \"" PART_LABEL "\" partition; edits won't survive a reboot");
    }
    if (load_saved()) {
        s_factory = false;
        ESP_LOGI(TAG, "Loaded the saved library: %d scenarios", s_n);
    } else {
        load_builtin();
        ESP_LOGI(TAG, "Built-in library: %d scenarios", s_n);
    }
}

int library_count(void)
{
    return s_n;
}

bool library_get(int i, demo_scenario_t *out)
{
    lock();
    bool ok = i >= 0 && i < s_n;
    if (ok) {
        *out = s_lib[i];
    }
    unlock();
    return ok;
}

int library_pick(uint32_t *rng, int last)
{
    lock();
    int n = s_n, p;
    unlock();
    if (n <= 0) {
        return -1;
    }
    if (n == 1) {
        return 0;
    }
    do {
        uint32_t x = *rng ? *rng : esp_random() | 1u;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        *rng = x;
        p = (int)(x % (uint32_t)n);
    } while (p == last);
    return p;
}

uint32_t library_version(void)
{
    return s_version;
}

bool library_is_factory(void)
{
    return s_factory;
}

static bool check(const demo_scenario_t *s, int self, char *msg, size_t len)
{
    if (!s->name[0]) {
        snprintf(msg, len, "the scenario needs a name");
        return false;
    }
    for (const char *c = s->name; *c; c++) {
        if ((unsigned char)*c < 0x20) {
            snprintf(msg, len, "the name can't contain control characters");
            return false;
        }
    }
    for (const char *c = s->expect; *c; c++) {
        if ((unsigned char)*c < 0x20) {
            snprintf(msg, len, "the expected response can't contain control characters");
            return false;
        }
    }
    for (int i = 0; i < s_n; i++) {
        if (i != self && strcmp(s_lib[i].name, s->name) == 0) {
            snprintf(msg, len, "another scenario is already called \"%s\"", s->name);
            return false;
        }
    }
    if (s->group >= DEMO_GROUP_COUNT) {
        snprintf(msg, len, "unknown group");
        return false;
    }
    if (s->n_sets > DEMO_MAX_SETS) {
        snprintf(msg, len, "a scenario sets at most %d values", DEMO_MAX_SETS);
        return false;
    }
    for (int i = 0; i < s->n_sets; i++) {
        for (int j = 0; j < i; j++) {
            if (s->sets[i].entity == s->sets[j].entity) {
                snprintf(msg, len, "%s is set twice", DEMO_ENTITIES[s->sets[i].entity].key);
                return false;
            }
        }
        if (!demo_value_check(s->sets[i].entity, s->sets[i].value, msg, len)) {
            return false;
        }
    }
    return true;
}

bool library_put(int i, const demo_scenario_t *s, int *index, char *msg, size_t len)
{
    lock();
    bool ok = false;
    if (i >= s_n || i < -1) {
        snprintf(msg, len, "no scenario #%d", i + 1);
    } else if (i == -1 && s_n >= LIB_MAX) {
        snprintf(msg, len, "the library is full (%d scenarios)", LIB_MAX);
    } else if (check(s, i, msg, len)) {
        int at = i == -1 ? s_n++ : i;
        s_lib[at] = *s;
        s_lib[at].name[DEMO_NAME_MAX] = '\0';
        s_lib[at].expect[DEMO_EXPECT_MAX] = '\0';
        changed();
        ok = save(msg, len);
        if (ok) {
            snprintf(msg, len, "scenario #%d \"%s\" saved", at + 1, s_lib[at].name);
        }
        if (index) {
            *index = at;
        }
    }
    unlock();
    return ok;
}

bool library_delete(int i, char *msg, size_t len)
{
    lock();
    bool ok = false;
    if (i < 0 || i >= s_n) {
        snprintf(msg, len, "no scenario #%d", i + 1);
    } else if (s_n <= 1) {
        snprintf(msg, len, "the library needs at least one scenario");
    } else {
        char name[DEMO_NAME_MAX + 1];
        strlcpy(name, s_lib[i].name, sizeof(name));
        memmove(&s_lib[i], &s_lib[i + 1], (s_n - i - 1) * sizeof(demo_scenario_t));
        s_n--;
        changed();
        ok = save(msg, len);
        if (ok) {
            snprintf(msg, len, "scenario \"%s\" deleted", name);
        }
    }
    unlock();
    return ok;
}

bool library_reset(char *msg, size_t len)
{
    lock();
    load_builtin();
    demo_limits_reset();
    s_version++;
    s_factory = true;
    esp_err_t err = s_part ? esp_partition_erase_range(s_part, 0, 4096) : ESP_OK;
    unlock();
    if (err != ESP_OK) {
        snprintf(msg, len, "reset, but couldn't clear the saved copy (%s)", esp_err_to_name(err));
        return false;
    }
    snprintf(msg, len, "library and limits back to the built-in %d scenarios", DEMO_BUILTIN_COUNT);
    return true;
}

bool library_set_limit(int e, const demo_limit_t *l, char *msg, size_t len)
{
    lock();
    bool ok = demo_limit_set(e, l, msg, len);
    if (ok) {
        changed();
        ok = save(msg, len);
        if (ok) {
            snprintf(msg, len, "%s limits saved", DEMO_ENTITIES[e].key);
        }
    }
    unlock();
    return ok;
}

bool library_reset_limits(char *msg, size_t len)
{
    lock();
    demo_limits_reset();
    changed();
    bool ok = save(msg, len);
    unlock();
    if (ok) {
        snprintf(msg, len, "limits back to the built-in values");
    }
    return ok;
}

void library_write_list(jw_t *w)
{
    // Names are copied one at a time so the lock is never held while writing to a socket.
    jw_raw(w, "[");
    for (int i = 0;; i++) {
        demo_scenario_t s;
        if (!library_get(i, &s)) {
            break;
        }
        jw_raw(w, i ? ",{\"name\":" : "{\"name\":");
        jw_str(w, s.name);
        jw_key(w, false, "group");
        jw_num(w, s.group);
        jw_key(w, false, "n");
        jw_num(w, s.n_sets);
        jw_key(w, false, "doorbell");
        jw_raw(w, jbool(s.flags & SCEN_DOORBELL));
        jw_raw(w, "}");
    }
    jw_raw(w, "]");
}

bool library_write_scenario(jw_t *w, int i)
{
    demo_scenario_t s;
    if (!library_get(i, &s)) {
        return false;
    }
    jw_raw(w, "{\"id\":");
    jw_num(w, i + 1);
    jw_key(w, false, "name");
    jw_str(w, s.name);
    jw_key(w, false, "group");
    jw_num(w, s.group);
    jw_key(w, false, "doorbell");
    jw_raw(w, jbool(s.flags & SCEN_DOORBELL));
    jw_key(w, false, "expect");
    jw_str(w, s.expect);
    jw_raw(w, ",\"set\":{");
    for (int k = 0; k < s.n_sets; k++) {
        char val[32];
        int e = s.sets[k].entity;
        demo_json_value(e, s.sets[k].value, val, sizeof(val));
        if (k) {
            jw_raw(w, ",");
        }
        jw_str(w, DEMO_ENTITIES[e].key);
        jw_raw(w, ":");
        jw_raw(w, val);
    }
    jw_raw(w, "}}");
    return true;
}
