#ifndef POLICY_HPP
#define POLICY_HPP

#include <vector>

class Device;
class CommandQueue;

class Policy {
public:
    virtual ~Policy() {}
    virtual void evaluate(const std::vector<Device*>& devices, CommandQueue* queue) = 0;
};

#endif
