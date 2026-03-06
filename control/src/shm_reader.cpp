#include "shm_reader.hpp"
#include <fstream>
#include <stdexcept>
#include <vector>

namespace shm {

std::pair<uint64_t, std::string> read_shm(const std::string& path) {
    std::ifstream f(path, std::ios::binary);
    if (!f) throw std::runtime_error("failed to open file: " + path);
    // read header
    uint8_t hdr[12];
    f.read(reinterpret_cast<char*>(hdr), 12);
    if (f.gcount() < 12) throw std::runtime_error("file too small");
    // big-endian u64
    uint64_t version = 0;
    for (int i = 0; i < 8; ++i) version = (version << 8) | hdr[i];
    uint32_t len = 0;
    for (int i = 8; i < 12; ++i) len = (len << 8) | hdr[i];
    std::vector<char> buf(len);
    f.read(buf.data(), len);
    if (static_cast<uint32_t>(f.gcount()) < len) buf.resize(f.gcount());
    return {version, std::string(buf.begin(), buf.end())};
}

} // namespace shm
