#include "json_writer.h"

#include <float.h>
#include <limits.h>
#include <math.h>
#include <stdarg.h>
#include <stdio.h>
#include <string.h>

void jw_init(jw_t *w, char *buf, size_t cap, jw_sink_t sink, void *ctx)
{
    w->buf = buf;
    w->cap = cap;
    w->n = 0;
    w->sink = sink;
    w->ctx = ctx;
    w->err = false;
}

bool jw_flush(jw_t *w)
{
    if (w->sink && w->n && !w->err) {
        w->err = !w->sink(w->ctx, w->buf, w->n);
        w->n = 0;
    }
    return !w->err;
}

void jw_mem(jw_t *w, const char *s, size_t len)
{
    while (len && !w->err) {
        if (w->n == w->cap) {
            if (!w->sink) {
                w->err = true;  // fixed buffer is full
                return;
            }
            jw_flush(w);
            continue;
        }
        size_t k = w->cap - w->n;
        k = len < k ? len : k;
        memcpy(w->buf + w->n, s, k);
        w->n += k;
        s += k;
        len -= k;
    }
}

void jw_raw(jw_t *w, const char *s)
{
    jw_mem(w, s, strlen(s));
}

void jw_printf(jw_t *w, const char *fmt, ...)
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

void jw_str(jw_t *w, const char *s)
{
    jw_mem(w, "\"", 1);
    if (!s) {
        s = "";
    }
    const char *run = s;
    for (; *s; s++) {
        unsigned char c = (unsigned char)*s;
        if (c > 31 && c != '"' && c != '\\') {
            continue;
        }
        jw_mem(w, run, s - run);
        char esc[8] = {'\\', (char)c, '\0'};
        switch (c) {
        case '\b': esc[1] = 'b'; break;
        case '\f': esc[1] = 'f'; break;
        case '\n': esc[1] = 'n'; break;
        case '\r': esc[1] = 'r'; break;
        case '\t': esc[1] = 't'; break;
        case '"':
        case '\\': break;
        default: snprintf(esc, sizeof(esc), "\\u%04x", c); break;
        }
        jw_raw(w, esc);
        run = s + 1;
    }
    jw_mem(w, run, s - run);
    jw_mem(w, "\"", 1);
}

// cJSON's print_number(), including valueint's saturation
void jw_num(jw_t *w, double d)
{
    char b[26];
    if (isnan(d) || isinf(d)) {
        jw_raw(w, "null");
        return;
    }
    int i = d >= INT_MAX ? INT_MAX : d <= (double)INT_MIN ? INT_MIN : (int)d;
    if (d == (double)i) {
        snprintf(b, sizeof(b), "%d", i);
    } else {
        // 15 digits unless that loses the value, then 17
        snprintf(b, sizeof(b), "%1.15g", d);
        double test = 0.0;
        bool ok = sscanf(b, "%lg", &test) == 1;
        double max = fabs(test) > fabs(d) ? fabs(test) : fabs(d);
        if (!ok || !(fabs(test - d) <= max * DBL_EPSILON)) {
            snprintf(b, sizeof(b), "%1.17g", d);
        }
    }
    jw_raw(w, b);
}

void jw_fixed(jw_t *w, float v, int dec)
{
    jw_printf(w, "%.*f", dec, (double)v);
}

void jw_key(jw_t *w, bool first, const char *key)
{
    jw_raw(w, first ? "\"" : ",\"");
    jw_raw(w, key);
    jw_mem(w, "\":", 2);
}

const char *jbool(bool b)
{
    return b ? "true" : "false";
}
