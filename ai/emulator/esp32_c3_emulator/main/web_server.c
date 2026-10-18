#include "web_server.h"

#include <stdlib.h>
#include <string.h>

#include "ai_message.h"
#include "cJSON.h"
#include "emulator_ctrl.h"
#include "esp_http_server.h"
#include "esp_log.h"

static const char *TAG = "web";

#define BODY_MAX 2048

extern const char index_html_start[] asm("_binary_index_html_start");
extern const char index_html_end[] asm("_binary_index_html_end");

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

static esp_err_t index_get(httpd_req_t *req)
{
    httpd_resp_set_type(req, "text/html; charset=utf-8");
    // EMBED_TXTFILES appends a NUL; don't send it.
    return httpd_resp_send(req, index_html_start, index_html_end - index_html_start - 1);
}

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
    if (req->content_len == 0 || req->content_len > BODY_MAX) {
        httpd_resp_send_err(req, HTTPD_400_BAD_REQUEST, "body must be 1..2048 bytes of JSON");
        return ESP_FAIL;
    }
    char *body = malloc(req->content_len + 1);
    if (!body) {
        httpd_resp_send_err(req, HTTPD_500_INTERNAL_SERVER_ERROR, "out of memory");
        return ESP_FAIL;
    }
    size_t got = 0;
    while (got < req->content_len) {
        int r = httpd_req_recv(req, body + got, req->content_len - got);
        if (r == HTTPD_SOCK_ERR_TIMEOUT) {
            continue;
        }
        if (r <= 0) {
            free(body);
            return ESP_FAIL;
        }
        got += r;
    }
    body[got] = '\0';

    char msg[160];
    bool ok = ctrl_execute(body, got, "web", msg, sizeof(msg));
    free(body);

    cJSON *res = cJSON_CreateObject();
    cJSON_AddBoolToObject(res, "ok", ok);
    cJSON_AddStringToObject(res, "msg", msg);
    char *json = cJSON_PrintUnformatted(res);
    cJSON_Delete(res);
    if (!ok) {
        httpd_resp_set_status(req, "400 Bad Request");
    }
    return send_json(req, json);
}

void web_server_start(void)
{
    static httpd_handle_t server;
    if (server) {
        return;
    }
    httpd_config_t cfg = HTTPD_DEFAULT_CONFIG();
    cfg.stack_size = 8192;  // cJSON state document
    cfg.lru_purge_enable = true;
    cfg.max_uri_handlers = 8;
    if (httpd_start(&server, &cfg) != ESP_OK) {
        ESP_LOGE(TAG, "Failed to start the web server");
        return;
    }
    const httpd_uri_t uris[] = {
        {.uri = "/", .method = HTTP_GET, .handler = index_get},
        {.uri = "/api/meta", .method = HTTP_GET, .handler = meta_get},
        {.uri = "/api/state", .method = HTTP_GET, .handler = state_get},
        {.uri = "/api/messages", .method = HTTP_GET, .handler = messages_get},
        {.uri = "/api/cmd", .method = HTTP_POST, .handler = cmd_post},
    };
    for (size_t i = 0; i < sizeof(uris) / sizeof(uris[0]); i++) {
        httpd_register_uri_handler(server, &uris[i]);
    }
    ESP_LOGI(TAG, "Control panel running on port 80");
}
