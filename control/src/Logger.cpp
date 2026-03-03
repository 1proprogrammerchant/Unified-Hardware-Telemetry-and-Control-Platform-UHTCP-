#include "../include/Logger.hpp"
#include <iostream>
#include <chrono>

void Logger::info(const std::string &s) {
    std::cout << "[INFO] " << s << std::endl;
}

void Logger::debug(const std::string &s) {
    std::cout << "[DEBUG] " << s << std::endl;
}

void Logger::warn(const std::string &s) {
    std::cout << "[WARN] " << s << std::endl;
}
