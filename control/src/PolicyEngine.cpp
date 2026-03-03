#include "../include/PolicyEngine.hpp"
#include "../include/Logger.hpp"

PolicyEngine::PolicyEngine() {}
PolicyEngine::~PolicyEngine() {}

void PolicyEngine::add_policy(std::unique_ptr<Policy> p) {
    policies.push_back(std::move(p));
}

void PolicyEngine::add_device(Device* d) {
    devices.push_back(d);
}

void PolicyEngine::set_command_queue(CommandQueue* q) {
    queue = q;
}

void PolicyEngine::evaluate() {
    Logger::debug("PolicyEngine: evaluate");
    for (auto &p : policies) {
        p->evaluate(devices, queue);
    }
}
