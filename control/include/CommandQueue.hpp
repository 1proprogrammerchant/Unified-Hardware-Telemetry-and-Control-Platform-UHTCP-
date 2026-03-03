#ifndef COMMAND_QUEUE_HPP
#define COMMAND_QUEUE_HPP

#include <queue>
#include <mutex>
#include <vector>
#include <string>

struct ControlCommand {
    std::string target;
    std::string type;
    int value;
};

class CommandQueue {
public:
    CommandQueue();
    void push(const ControlCommand& cmd);
    std::vector<ControlCommand> pop_batch(size_t max_batch = 64);
    void process_batch();
private:
    std::queue<ControlCommand> q;
    std::mutex mtx;
};

#endif
