// SHM layout constants and quick reference for interop (Rust <-> C++ <-> Ruby)
// Layout (bytes, big-endian):
//   0..8   -> u64 version (BE)
//   8..12  -> u32 payload_len (BE)
//   12..   -> payload bytes (UTF-8 JSON)
//
// Example (pseudocode):
//   version = 1
//   payload = b'{"cpu":1}'
//   write(version.to_be_bytes()); write((len(payload) as u32).to_be_bytes()); write(payload)
//
// Ruby helper: automation/read_shm.rb reads the same format and prints the JSON.
// C++ helper: control/include/shm_reader.hpp / control/src/shm_reader.cpp provides
// `shm::read_shm(path) -> std::pair<uint64_t, std::string>`.

// These constants can be copy-pasted into small helpers in other languages.
pub const SHM_HEADER_VERSION_BYTES: usize = 8;
pub const SHM_HEADER_LEN_BYTES: usize = 4;
pub const SHM_HEADER_BYTES: usize = SHM_HEADER_VERSION_BYTES + SHM_HEADER_LEN_BYTES;
