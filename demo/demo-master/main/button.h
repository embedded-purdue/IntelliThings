// The game buzzer input and the status LED.
//
// The buzzer's output isn't a clean switch: a quick tap gives a pulse of a few milliseconds, and
// the signal can pulse with the buzzer's LED or sound. So a press is detected from the edge (an
// interrupt catches even a short pulse), then confirmed by how long the input was pressed in
// total over the next CONFIRM window, or by further pulses. One press counts per touch: the input
// re-arms only after it has been quiet (released, no pulses) for the release time, however long
// someone holds the buzzer. on_press runs from the button task right after the confirmation; the
// engine decides whether it starts a roll (it ignores presses while rolling and during the hold).
#pragma once

#include <stdbool.h>
#include <stdint.h>

typedef void (*button_cb_t)(void);

void button_init(button_cb_t on_press);
// True from a confirmed press until the input has been quiet for the release time, so it stays
// true while someone holds the buzzer, even if its signal pulses.
bool button_is_held(void);
bool button_level(void);         // the raw input level right now (pressed = true)
uint32_t button_held_ms(void);   // how long the current press has lasted (0 when not held)
uint32_t button_rejected(void);  // edges that didn't confirm as a press (noise, too short)
void button_led_set(bool on);
