#ifndef UHTCP_HAL_GPIO_H
#define UHTCP_HAL_GPIO_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

int hal_gpio_init(void);
int hal_gpio_read(uint32_t pin, uint32_t* out);
int hal_gpio_write(uint32_t pin, uint32_t value);

#ifdef __cplusplus
}
#endif

#endif
