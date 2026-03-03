#ifndef UHTCP_HAL_MEMORY_H
#define UHTCP_HAL_MEMORY_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

int hal_memory_stats(uint64_t* total, uint64_t* free);

#ifdef __cplusplus
}
#endif

#endif
