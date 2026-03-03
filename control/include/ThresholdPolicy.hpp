#ifndef THRESHOLD_POLICY_HPP
#define THRESHOLD_POLICY_HPP

#include "Policy.hpp"
#include <string>

class ThresholdPolicy : public Policy {
public:
    ThresholdPolicy(const std::string &target_id, double threshold);
    void evaluate(const std::vector<Device*>& devices, CommandQueue* queue) override;
private:
    std::string target_id;
    double threshold;
};

#endif
