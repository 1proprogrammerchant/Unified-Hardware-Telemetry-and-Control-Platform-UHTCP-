#ifndef _POSIX_C_SOURCE
#define _POSIX_C_SOURCE 200809L
#endif
#include "../include/hal_cpu.h"
#include <stdio.h>
#include <time.h>

int hal_cpu_read_temp(float* out) {
    if (!out) return -1;
    // Try reading Linux sysfs, else return -1
    FILE* f = fopen("/sys/class/thermal/thermal_zone0/temp", "r");
    if (!f) { *out = -1.0f; return -1; }
    long t = 0;
    if (fscanf(f, "%ld", &t) != 1) { fclose(f); *out = -1.0f; return -1; }
    fclose(f);
    *out = ((float)t) / 1000.0f;
    return 0;
}

uint64_t hal_cpu_uptime(void) {
    struct timespec ts;
    if (clock_gettime(CLOCK_MONOTONIC, &ts) == 0) return (uint64_t)ts.tv_sec;
    return 0;
}
