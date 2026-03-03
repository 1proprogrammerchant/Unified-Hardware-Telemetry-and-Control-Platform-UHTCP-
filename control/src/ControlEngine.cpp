#include "../include/ControlEngine.hpp"
#include "../include/Logger.hpp"
#include "../include/PolicyEngine.hpp"
#include "../include/CommandQueue.hpp"
#include "../include/SafetyManager.hpp"
#include "../include/DependencyGraph.hpp"

ControlEngine::ControlEngine() : running(false) {}

ControlEngine::~ControlEngine() { shutdown(); }

void ControlEngine::initialize() {
    Logger::info("ControlEngine: initialize");
    policy_engine = std::make_unique<PolicyEngine>();
    command_queue = std::make_unique<CommandQueue>();
    // wire command queue into policy engine
    policy_engine->set_command_queue(command_queue.get());
    safety_manager = std::make_unique<SafetyManager>();
    dependency_graph = std::make_unique<DependencyGraph>();
    running.store(true);
}

void ControlEngine::register_device(Device* d) {
    if (d) devices.push_back(d);
    if (policy_engine) policy_engine->add_device(d);
}

void ControlEngine::poll_state() {
    // Placeholder: in a real system this would read state from the Rust core via IPC
    Logger::debug("ControlEngine: poll_state (stub)");
}

void ControlEngine::evaluatePolicies() {
    Logger::debug("ControlEngine: evaluatePolicies");
    if (policy_engine) policy_engine->evaluate();
}

void ControlEngine::enforceSafety() {
    Logger::debug("ControlEngine: enforceSafety");
    if (safety_manager) safety_manager->check_and_enforce();
}

void ControlEngine::updateDevices() {
    Logger::debug("ControlEngine: updateDevices");
    for (auto *d : devices) {
        if (d) d->apply_control();
    }
}

void ControlEngine::dispatchCommands() {
    Logger::debug("ControlEngine: dispatchCommands");
    if (command_queue) command_queue->process_batch();
}

void ControlEngine::update() {
    if (!running.load()) return;
    poll_state();
    evaluatePolicies();
    enforceSafety();
    updateDevices();
    dispatchCommands();
}

void ControlEngine::shutdown() {
    if (!running.exchange(false)) return;
    Logger::info("ControlEngine: shutdown");
}
