
#include "hardware.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

static uint32_t g_gpio_state = 0;

int hal_init(void) {
    // Initialize HAL resources if needed. For now, noop.
    g_gpio_state = 0;
    return 0;
}

static float _read_cpu_temp_internal(void) {
    FILE* f = fopen("/sys/class/thermal/thermal_zone0/temp", "r");
    if (!f) return -1.0f;
    long temp_milli = 0;
    if (fscanf(f, "%ld", &temp_milli) != 1) {
        fclose(f);
        return -1.0f;
    }
    fclose(f);
    return ((float)temp_milli) / 1000.0f;
}

int hal_read_snapshot(hal_snapshot_t* snapshot) {
    if (!snapshot) return -1;

    float cpu = _read_cpu_temp_internal();
    snapshot->cpu_temperature = cpu;

    struct timespec ts;
    if (clock_gettime(CLOCK_MONOTONIC, &ts) == 0) {
        snapshot->uptime = (uint64_t)ts.tv_sec;
    } else {
        snapshot->uptime = 0;
    }

    snapshot->gpio_state = g_gpio_state;

    return 0;
}

int hal_write_gpio(uint32_t pin, uint32_t value) {
    // Very small, deterministic in-memory state change only.
    // A production HAL would write to device files or MMIO here.
    if (pin >= 32) return -1;
    if (value)
        g_gpio_state |= (1u << pin);
    else
        g_gpio_state &= ~(1u << pin);
    return 0;
}

void hal_shutdown(void) {
    // Clean up HAL resources if any.
}

