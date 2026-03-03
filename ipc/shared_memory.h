#ifndef UHTCP_SHARED_MEMORY_H
#define UHTCP_SHARED_MEMORY_H

#include <stddef.h>

int ipc_shm_create(const char* name, size_t size);
void* ipc_shm_map(const char* name, size_t size);
int ipc_shm_unlink(const char* name);

#endif
