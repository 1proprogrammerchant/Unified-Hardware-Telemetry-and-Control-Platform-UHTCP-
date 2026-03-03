#include "Device.hpp"

extern "C" {
#include "../hal/hardware.h"
}

#include <iostream>

class TempSensor : public Device {
public:
    void read() override {
        float t = read_cpu_temp();
        if (t < 0) std::cout << "Temp: unavailable\n";
        else std::cout << "CPU Temp: " << t << " C\n";
    }

    void write(int) override {}
};

int main() {
    TempSensor s;
    s.read();
    return 0;
}
