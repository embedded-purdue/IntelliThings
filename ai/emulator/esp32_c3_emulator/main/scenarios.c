// Shortcut scenarios for the control panel and the MQTT cmd topic.
// SC_RANGE holds a value inside [a, b]; SC_DELTA adds a to the automatic target.
#include <string.h>

#include "sim_model.h"

const scenario_t SCENARIOS[] = {
    {
        .name = "cooking", .label = "Cook", .icon = "🍳", .default_room = "kitchen",
        .duration_s = 1200, .presence = 1, .occupants = 0, .lights = 1,
        .m = {
            [METRIC_TEMPERATURE] = {SC_DELTA, 2.5f, 0},
            [METRIC_HUMIDITY] = {SC_DELTA, 15.0f, 0},
            [METRIC_VOC] = {SC_RANGE, 250, 350},
            [METRIC_CO2] = {SC_RANGE, 900, 1300},
            [METRIC_LUX] = {SC_RANGE, 350, 600},
            [METRIC_PM2_5] = {SC_RANGE, 40, 80},
        },
    },
    {
        .name = "burnt_food", .label = "Burnt food", .icon = "🔥", .default_room = "kitchen",
        .duration_s = 600, .presence = 1, .occupants = 0, .lights = 1,
        .m = {
            [METRIC_TEMPERATURE] = {SC_DELTA, 3.0f, 0},
            [METRIC_HUMIDITY] = {SC_DELTA, 10.0f, 0},
            [METRIC_VOC] = {SC_RANGE, 400, 500},
            [METRIC_CO2] = {SC_RANGE, 1000, 1400},
            [METRIC_LUX] = {SC_RANGE, 350, 600},
            [METRIC_PM2_5] = {SC_RANGE, 150, 300},
        },
    },
    {
        .name = "sleep", .label = "Sleep", .icon = "😴", .default_room = "bedroom",
        .duration_s = 8 * 3600, .presence = 1, .occupants = 1, .lights = 0,
        .m = {
            [METRIC_LUX] = {SC_RANGE, 0, 3},
        },
    },
    {
        .name = "gathering", .label = "Gathering", .icon = "🛋", .default_room = "living-room",
        .duration_s = 3600, .presence = 1, .occupants = 4, .lights = 1,
        .m = {
            [METRIC_TEMPERATURE] = {SC_DELTA, 1.0f, 0},
            [METRIC_LUX] = {SC_RANGE, 250, 450},
        },
    },
    {
        .name = "hot", .label = "Hot & occupied", .icon = "🥵", .default_room = "",
        .duration_s = 1800, .presence = 1, .occupants = 0, .lights = -1,
        .m = {
            [METRIC_TEMPERATURE] = {SC_RANGE, 28, 30},
            [METRIC_HUMIDITY] = {SC_RANGE, 55, 65},
        },
    },
    {
        .name = "pm_spike", .label = "PM2.5 spike", .icon = "🌫", .default_room = "",
        .duration_s = 600, .presence = -1, .occupants = 0, .lights = -1,
        .m = {
            [METRIC_PM2_5] = {SC_RANGE, 80, 150},
        },
    },
    {
        .name = "voc_spike", .label = "VOC spike", .icon = "🧴", .default_room = "",
        .duration_s = 600, .presence = -1, .occupants = 0, .lights = -1,
        .m = {
            [METRIC_VOC] = {SC_RANGE, 300, 450},
        },
    },
    {
        .name = "stuffy", .label = "Stuffy room", .icon = "😮‍💨", .default_room = "",
        .duration_s = 1800, .presence = 1, .occupants = 0, .lights = -1,
        .m = {
            [METRIC_CO2] = {SC_RANGE, 1200, 1800},
        },
    },
    {
        .name = "window_open", .label = "Open window", .icon = "🪟", .default_room = "",
        .duration_s = 1200, .presence = -1, .occupants = 0, .lights = -1,
        .m = {
            [METRIC_TEMPERATURE] = {SC_DELTA, -2.0f, 0},
            [METRIC_HUMIDITY] = {SC_DELTA, 5.0f, 0},
            [METRIC_CO2] = {SC_RANGE, 420, 500},
            [METRIC_PM2_5] = {SC_RANGE, 8, 15},
        },
    },
    {
        .name = "enter", .label = "Enter", .icon = "🚶", .default_room = "",
        .duration_s = 900, .presence = 1, .occupants = 0, .lights = -1,
    },
    {
        .name = "leave", .label = "Leave", .icon = "🚪", .default_room = "",
        .duration_s = 900, .presence = 0, .occupants = 0, .lights = 0,
    },
    {
        .name = "lights_off", .label = "Lights off", .icon = "🌙", .default_room = "",
        .duration_s = 1800, .presence = -1, .occupants = 0, .lights = 0,
        .m = {
            [METRIC_LUX] = {SC_RANGE, 0, 3},
        },
    },
};

const int SCENARIO_COUNT = sizeof(SCENARIOS) / sizeof(SCENARIOS[0]);

const scenario_t *scenario_find(const char *name)
{
    for (int i = 0; i < SCENARIO_COUNT; i++) {
        if (strcmp(SCENARIOS[i].name, name) == 0) {
            return &SCENARIOS[i];
        }
    }
    return NULL;
}
