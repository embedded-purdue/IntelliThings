// Small appending JSON writer: big documents (/api/poll, the emulator state) go out piece by
// piece instead of as a cJSON tree. Numbers and strings are printed exactly like
// cJSON_PrintUnformatted, so the output stays byte-for-byte the same.
#pragma once

#include <stdbool.h>
#include <stddef.h>

// Takes a full buffer; false stops the writer (err set, later writes dropped).
typedef bool (*jw_sink_t)(void *ctx, const char *data, size_t len);

typedef struct {
    char *buf;
    size_t cap, n;
    jw_sink_t sink;  // NULL: fixed buffer, overflow sets err
    void *ctx;
    bool err;
} jw_t;

void jw_init(jw_t *w, char *buf, size_t cap, jw_sink_t sink, void *ctx);
bool jw_flush(jw_t *w);  // hands what's buffered to the sink; false on any error so far

void jw_mem(jw_t *w, const char *s, size_t len);
void jw_raw(jw_t *w, const char *s);
void jw_printf(jw_t *w, const char *fmt, ...) __attribute__((format(printf, 2, 3)));
void jw_str(jw_t *w, const char *s);  // quoted and escaped; NULL prints ""
void jw_num(jw_t *w, double v);       // like cJSON: integers plain, others %.15g or %.17g
void jw_fixed(jw_t *w, float v, int dec);  // %.*f

// Object members: ,"key": (the leading comma unless first)
void jw_key(jw_t *w, bool first, const char *key);

const char *jbool(bool b);
