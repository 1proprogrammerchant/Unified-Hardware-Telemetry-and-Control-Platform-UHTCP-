#ifndef UHTCP_HAL_CPU_H
#define UHTCP_HAL_CPU_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

int hal_cpu_read_temp(float* out);
uint64_t hal_cpu_uptime(void);

#ifdef __cplusplus
}
#endif

#endif
