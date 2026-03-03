#ifndef UHTCP_HAL_H
#define UHTCP_HAL_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct {
    float cpu_temperature;
    uint64_t uptime;
    uint32_t gpio_state;
} hal_snapshot_t;

int hal_init(void);
int hal_read_snapshot(hal_snapshot_t* snapshot);
int hal_write_gpio(uint32_t pin, uint32_t value);
void hal_shutdown(void);

#ifdef __cplusplus
}
#endif

#endif
