#include "../include/hal_memory.h"
#include <stdint.h>
#include <stdio.h>

int hal_memory_stats(uint64_t* total, uint64_t* free) {
    if (!total || !free) return -1;
    // best-effort parsing from /proc/meminfo on Linux
    FILE* f = fopen("/proc/meminfo", "r");
    if (!f) return -1;
    char key[64];
    unsigned long val = 0;
    *total = 0; *free = 0;
    while (fscanf(f, "%63s %lu kB\n", key, &val) == 2) {
        if (strncmp(key, "MemTotal:", 9) == 0) *total = val * 1024ULL;
        if (strncmp(key, "MemFree:", 8) == 0) *free = val * 1024ULL;
    }
    fclose(f);
    return 0;
}
