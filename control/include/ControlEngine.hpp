#ifndef CONTROL_ENGINE_HPP
#define CONTROL_ENGINE_HPP

#include <vector>
#include <memory>
#include <atomic>
#include "Device.hpp"

class PolicyEngine;
class CommandQueue;
class SafetyManager;
class DependencyGraph;

class ControlEngine {
public:
    ControlEngine();
    ~ControlEngine();

    void initialize();
    void update();
    void shutdown();
    void register_device(Device* d);

private:
    void poll_state();
    void evaluatePolicies();
    void enforceSafety();
    void updateDevices();
    void dispatchCommands();

    std::vector<Device*> devices;
    std::unique_ptr<PolicyEngine> policy_engine;
    std::unique_ptr<CommandQueue> command_queue;
    std::unique_ptr<SafetyManager> safety_manager;
    std::unique_ptr<DependencyGraph> dependency_graph;

    std::atomic<bool> running;
};

#endif
