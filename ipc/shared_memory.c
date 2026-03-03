#include "shared_memory.h"
#include <sys/mman.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <unistd.h>
#include <string.h>

int ipc_shm_create(const char* name, size_t size) {
    int fd = shm_open(name, O_CREAT | O_RDWR, 0666);
    if (fd < 0) return -1;
    if (ftruncate(fd, size) != 0) { close(fd); return -1; }
    close(fd);
    return 0;
}

void* ipc_shm_map(const char* name, size_t size) {
    int fd = shm_open(name, O_RDWR, 0666);
    if (fd < 0) return NULL;
    void* p = mmap(NULL, size, PROT_READ | PROT_WRITE, MAP_SHARED, fd, 0);
    close(fd);
    return (p == MAP_FAILED) ? NULL : p;
}

int ipc_shm_unlink(const char* name) { return shm_unlink(name); }
