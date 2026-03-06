#include "../include/hal.h"
#include "../include/hal_gpio.h"
#include "../include/hal_cpu.h"
#include <stdio.h>

int hal_init(void) {
    hal_gpio_init();
    return 0;
}

int hal_read_snapshot(hal_snapshot_t* snapshot) {
    if (!snapshot) return -1;
    float t = -1.0f;
    hal_cpu_read_temp(&t);
    snapshot->cpu_temperature = t;
    snapshot->uptime = hal_cpu_uptime();
    uint32_t gs = 0;
    hal_gpio_read(0, &gs); // simple read of gpio state (placeholder)
    snapshot->gpio_state = gs;
    return 0;
}

int hal_write_gpio(uint32_t pin, uint32_t value) {
    return hal_gpio_write(pin, value);
}

void hal_shutdown(void) {
    // cleanup
}
