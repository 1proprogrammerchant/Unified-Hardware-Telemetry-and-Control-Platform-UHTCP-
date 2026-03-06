#include "../include/hal_gpio.h"
#include <stdint.h>
#include <stdio.h>

static uint32_t _gpio_state = 0;

int hal_gpio_init(void) {
    _gpio_state = 0;
    return 0;
}

int hal_gpio_read(uint32_t pin, uint32_t* out) {
    if (!out) return -1;
    (void)pin; /* pin is unused in this simple implementation */
    *out = _gpio_state;
    return 0;
}

int hal_gpio_write(uint32_t pin, uint32_t value) {
    if (pin >= 32) return -1;
    if (value) _gpio_state |= (1u << pin);
    else _gpio_state &= ~(1u << pin);
    return 0;
}
