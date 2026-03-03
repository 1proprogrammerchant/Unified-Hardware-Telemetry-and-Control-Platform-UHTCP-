#ifndef SIMPLE_DEVICE_HPP
#define SIMPLE_DEVICE_HPP

#include "Device.hpp"
#include "PIDController.hpp"

class SimpleDevice : public Device {
public:
    SimpleDevice(const std::string &id_, const std::string &name_);
    void synchronize() override;
    void apply_control() override;
    void set_setpoint(double s) { setpoint = s; }
    double read_measurement() const;
private:
    double measurement = 0.0;
    double setpoint = 0.0;
    PIDController pid;
};

#endif
