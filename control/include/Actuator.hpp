#ifndef ACTUATOR_HPP
#define ACTUATOR_HPP

#include "Device.hpp"

class Actuator : public Device {
public:
    void synchronize() override;
    void apply_control() override;
};

#endif
