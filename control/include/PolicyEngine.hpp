#ifndef POLICY_ENGINE_HPP
#define POLICY_ENGINE_HPP

#include <vector>
#include <memory>
#include "Policy.hpp"
class CommandQueue;

class PolicyEngine {
public:
    PolicyEngine();
    ~PolicyEngine();

    void evaluate();
    void add_policy(std::unique_ptr<Policy> p);
    void add_device(class Device* d);
    void set_command_queue(CommandQueue* q);
private:
    std::vector<std::unique_ptr<Policy>> policies;
    std::vector<Device*> devices;
    CommandQueue* queue = nullptr;
};

#endif
