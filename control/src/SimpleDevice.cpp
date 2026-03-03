#include "../include/SimpleDevice.hpp"
#include "../include/Logger.hpp"

SimpleDevice::SimpleDevice(const std::string &id_, const std::string &name_) {
    id = id_; name = name_; state = DeviceState::IDLE; setpoint = 0.0; measurement = 0.0; pid = PIDController(1.0, 0.1, 0.01);
}

void SimpleDevice::synchronize() {
    // simulate measurement drift
    measurement += 0.01;
    Logger::debug("SimpleDevice: synchronize measurement=" + std::to_string(measurement));
}

void SimpleDevice::apply_control() {
    double control = pid.compute(setpoint, measurement, 0.1);
    // simulate applying control
    Logger::info("SimpleDevice: apply_control control=" + std::to_string(control));
}

double SimpleDevice::read_measurement() const { return measurement; }
