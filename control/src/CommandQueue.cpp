#include "../include/CommandQueue.hpp"
#include "../include/Logger.hpp"

CommandQueue::CommandQueue() {}

void CommandQueue::push(const ControlCommand& cmd) {
    std::lock_guard<std::mutex> g(mtx);
    q.push(cmd);
}

std::vector<ControlCommand> CommandQueue::pop_batch(size_t max_batch) {
    std::vector<ControlCommand> out;
    std::lock_guard<std::mutex> g(mtx);
    while (!q.empty() && out.size() < max_batch) {
        out.push_back(q.front());
        q.pop();
    }
    return out;
}

void CommandQueue::process_batch() {
    auto batch = pop_batch();
    for (auto &c : batch) {
        // In a full system, commands would be routed to devices or IPC
        Logger::debug("CommandQueue: processing command for " + c.target);
    }
}
