#ifndef SAFETY_MANAGER_HPP
#define SAFETY_MANAGER_HPP

#include <string>
#include <unordered_map>

class SafetyManager {
public:
    SafetyManager();
    void check_and_enforce();
    void register_limit(const std::string &name, double threshold);
private:
    // simple internal representation; real system would be richer
    std::unordered_map<std::string, double> limits;
};

#endif
