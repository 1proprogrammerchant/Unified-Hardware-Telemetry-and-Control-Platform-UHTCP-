#include <stdio.h>
#include <stdlib.h>
#include <time.h>

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
