#include "button.h"

#include "driver/gpio.h"
#include "esp_log.h"
#include "esp_rom_sys.h"
#include "esp_timer.h"
#include "event_log.h"
#include "freertos/FreeRTOS.h"
#include "freertos/task.h"

static const char *TAG = "button";

#define PIN CONFIG_DM_BUTTON_GPIO
#define PRESSED_LEVEL (CONFIG_DM_BUTTON_ACTIVE_HIGH ? 1 : 0)
#define SAMPLE_US 250                                  // confirmation sampling step
#define CONFIRM_US 20000                               // look at the input this long after an edge
#define MIN_PRESS_US (CONFIG_DM_BUTTON_MIN_PRESS_MS * 1000)
#define PULSE_EDGES 2                                  // further edges that also confirm a press
#define RELEASE_US (CONFIG_DM_BUTTON_RELEASE_MS * 1000)
#define POLL_MS 10                                     // release watch
#define STUCK_US (30 * 1000000LL)                      // warn once if "held" this long

static TaskHandle_t s_task;
static button_cb_t s_on_press;
static volatile uint32_t s_edges;  // pressed-going edges seen by the interrupt
static volatile bool s_held;
static volatile int64_t s_held_since;
static uint32_t s_rejected;

static void IRAM_ATTR on_edge(void *arg)
{
    s_edges++;
    BaseType_t woke = pdFALSE;
    vTaskNotifyGiveFromISR(s_task, &woke);
    portYIELD_FROM_ISR(woke);
}

bool button_level(void)
{
    return gpio_get_level(PIN) == PRESSED_LEVEL;
}

bool button_is_held(void)
{
    return s_held;
}

uint32_t button_held_ms(void)
{
    return s_held ? (uint32_t)((esp_timer_get_time() - s_held_since) / 1000) : 0;
}

uint32_t button_rejected(void)
{
    return s_rejected;
}

void button_led_set(bool on)
{
#if CONFIG_DM_LED_GPIO >= 0
    gpio_set_level(CONFIG_DM_LED_GPIO, on);
#endif
}

// Right after an edge: a press if the input was pressed for MIN_PRESS_US in total within
// CONFIRM_US, or if PULSE_EDGES more edges arrive (a signal that pulses). A lone spike from
// noise does neither. Busy-waits at most CONFIRM_US (this task's only job).
// edges_at_wake: the edge count when the task woke, the waking edge included.
static bool confirm(uint32_t edges_at_wake)
{
    int pressed_us = 0;
    for (int t = 0; t < CONFIRM_US; t += SAMPLE_US) {
        if (button_level()) {
            pressed_us += SAMPLE_US;
            if (pressed_us >= MIN_PRESS_US) {
                return true;
            }
        }
        if (s_edges - edges_at_wake >= PULSE_EDGES) {
            return true;
        }
        esp_rom_delay_us(SAMPLE_US);
    }
    return false;
}

// Waits until the input has been released, with no new edges, for RELEASE_US.
static void wait_for_quiet(void)
{
    int64_t quiet_since = esp_timer_get_time();
    uint32_t edges = s_edges;
    bool warned = false;
    for (;;) {
        vTaskDelay(pdMS_TO_TICKS(POLL_MS));
        int64_t now = esp_timer_get_time();
        if (button_level() || s_edges != edges) {
            edges = s_edges;
            quiet_since = now;
        } else if (now - quiet_since >= RELEASE_US) {
            return;
        }
        if (!warned && now - s_held_since >= STUCK_US) {
            warned = true;
            event_log("Buzzer held for 30 s: someone's holding it, or the input is stuck");
        }
    }
}

static void button_task(void *arg)
{
    for (;;) {
        ulTaskNotifyTake(pdTRUE, portMAX_DELAY);
        if (!confirm(s_edges)) {
            s_rejected++;
            continue;
        }
        s_held_since = esp_timer_get_time();
        s_held = true;
        if (s_on_press) {
            s_on_press();
        }
        wait_for_quiet();
        uint32_t held_ms = button_held_ms();
        s_held = false;
        if (held_ms >= 1000) {
            event_log("Buzzer released after %.1f s", held_ms / 1000.0);
        }
        ulTaskNotifyTake(pdTRUE, 0);  // drop the edges of this press
    }
}

void button_init(button_cb_t on_press)
{
    s_on_press = on_press;
#if CONFIG_DM_LED_GPIO >= 0
    gpio_config_t led = {
        .pin_bit_mask = 1ULL << CONFIG_DM_LED_GPIO,
        .mode = GPIO_MODE_OUTPUT,
    };
    gpio_config(&led);
    gpio_set_level(CONFIG_DM_LED_GPIO, 0);
#endif
    gpio_config_t in = {
        .pin_bit_mask = 1ULL << PIN,
        .mode = GPIO_MODE_INPUT,
        .pull_up_en = PRESSED_LEVEL ? GPIO_PULLUP_DISABLE : GPIO_PULLUP_ENABLE,
        .pull_down_en = PRESSED_LEVEL ? GPIO_PULLDOWN_ENABLE : GPIO_PULLDOWN_DISABLE,
        .intr_type = PRESSED_LEVEL ? GPIO_INTR_POSEDGE : GPIO_INTR_NEGEDGE,
    };
    ESP_ERROR_CHECK(gpio_config(&in));
    BaseType_t ok = xTaskCreate(button_task, "button", 4096, NULL, 6, &s_task);  // on_press writes NVS
    configASSERT(ok == pdPASS);
    ESP_ERROR_CHECK(gpio_install_isr_service(0));
    ESP_ERROR_CHECK(gpio_isr_handler_add(PIN, on_edge, NULL));
    ESP_LOGI(TAG, "Buzzer on GPIO%d (pressed = %s, min press %d ms, release %d ms)", PIN,
             PRESSED_LEVEL ? "high" : "low", CONFIG_DM_BUTTON_MIN_PRESS_MS,
             CONFIG_DM_BUTTON_RELEASE_MS);
}
