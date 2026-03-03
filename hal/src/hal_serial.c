#include "../include/hal_serial.h"
#include <fcntl.h>
#include <termios.h>
#include <unistd.h>
#include <string.h>

int hal_serial_open(const char* path, int baud) {
    int fd = open(path, O_RDWR | O_NOCTTY | O_SYNC);
    if (fd < 0) return -1;
    // minimal configuration omitted for brevity
    return fd;
}

int hal_serial_read(int fd, void* buf, int len) {
    return (int)read(fd, buf, (size_t)len);
}

int hal_serial_write(int fd, const void* buf, int len) {
    return (int)write(fd, buf, (size_t)len);
}

void hal_serial_close(int fd) { close(fd); }
