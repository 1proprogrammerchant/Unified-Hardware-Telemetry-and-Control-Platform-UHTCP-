#ifndef UHTCP_HAL_SERIAL_H
#define UHTCP_HAL_SERIAL_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

int hal_serial_open(const char* path, int baud);
int hal_serial_read(int fd, void* buf, int len);
int hal_serial_write(int fd, const void* buf, int len);
void hal_serial_close(int fd);

#ifdef __cplusplus
}
#endif

#endif
