#include <vector>
#include "../include/Device.hpp"

class DeviceRegistry {
public:
    std::vector<Device*> devices;
    void add(Device* d) { devices.push_back(d); }
};
