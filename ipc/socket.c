#include "socket.h"
#include <sys/socket.h>
#include <sys/un.h>
#include <unistd.h>
#include <string.h>

int ipc_socket_create(const char* path) {
    int s = socket(AF_UNIX, SOCK_STREAM, 0);
    if (s < 0) return -1;
    struct sockaddr_un addr;
    memset(&addr, 0, sizeof(addr));
    addr.sun_family = AF_UNIX;
    strncpy(addr.sun_path, path, sizeof(addr.sun_path)-1);
    unlink(path);
    if (bind(s, (struct sockaddr*)&addr, sizeof(addr)) != 0) { close(s); return -1; }
    listen(s, 5);
    return s;
}
