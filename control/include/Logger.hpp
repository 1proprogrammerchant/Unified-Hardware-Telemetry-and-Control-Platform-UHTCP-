#ifndef LOGGER_HPP
#define LOGGER_HPP

#include <string>

class Logger {
public:
    static void info(const std::string &s);
    static void debug(const std::string &s);
    static void warn(const std::string &s);
};

#endif
