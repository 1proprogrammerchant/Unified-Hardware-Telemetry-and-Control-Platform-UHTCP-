#pragma once
#include <string>
#include <utility>
#include <cstdint>

namespace shm {
    // Read the SHM layout: 8-byte BE version, 4-byte BE len, then payload bytes.
    // Returns pair(version, payload_string). Throws std::runtime_error on error.
    std::pair<uint64_t, std::string> read_shm(const std::string& path);
}
