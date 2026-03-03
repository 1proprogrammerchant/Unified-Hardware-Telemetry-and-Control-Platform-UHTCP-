#ifndef DEVICE_HPP
#define DEVICE_HPP

#include <string>
#include <mutex>

enum class DeviceState {
    UNINITIALIZED,
    IDLE,
    ACTIVE,
    ERROR,
    RECOVERING
};

class Device {
public:
    virtual ~Device() {}
    virtual void synchronize() = 0;
    virtual void apply_control() = 0;
    virtual void transition(DeviceState next) { std::lock_guard<std::mutex> g(mtx); state = next; }
    virtual void handleError() { std::lock_guard<std::mutex> g(mtx); state = DeviceState::ERROR; }
    virtual void recover() { std::lock_guard<std::mutex> g(mtx); state = DeviceState::RECOVERING; }

    std::string id;
    std::string name;
    DeviceState state = DeviceState::UNINITIALIZED;
protected:
    std::mutex mtx;
};

#endif
