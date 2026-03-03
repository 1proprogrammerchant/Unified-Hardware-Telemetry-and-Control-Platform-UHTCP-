#include "../include/ThresholdPolicy.hpp"
#include "../include/SimpleDevice.hpp"
#include "../include/CommandQueue.hpp"
#include "../include/Logger.hpp"

ThresholdPolicy::ThresholdPolicy(const std::string &target_id_, double threshold_) : target_id(target_id_), threshold(threshold_) {}

void ThresholdPolicy::evaluate(const std::vector<Device*>& devices, CommandQueue* queue) {
    for (auto d : devices) {
        auto s = dynamic_cast<SimpleDevice*>(d);
        if (!s) continue;
        if (s->id == target_id) {
            double m = s->read_measurement();
            if (m > threshold) {
                Logger::info("ThresholdPolicy: device " + s->id + " exceeded threshold " + std::to_string(m));
                if (queue) {
                    ControlCommand cmd; cmd.target = s->id; cmd.type = "throttle"; cmd.value = 0;
                    queue->push(cmd);
                }
            }
        }
    }
}
