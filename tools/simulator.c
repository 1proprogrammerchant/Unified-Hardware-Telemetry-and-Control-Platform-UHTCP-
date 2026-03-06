#include <stdio.h>
#include <stdlib.h>
#ifndef _POSIX_C_SOURCE
#define _POSIX_C_SOURCE 200809L
#endif
#include <time.h>
#include <unistd.h>

int main() {
    float temp = 40.0f;
    while (1) {
        temp += (rand() % 5) - 2;
        printf("SIM TEMP: %.2f\n", temp);
        fflush(stdout);
        sleep(1);
    }
    return 0;
}
