// Rust helper: write_shm_sample
// Provides a small Rust helper to write a sample shared-memory file using the
// same layout used by `ShmRegion` (8-byte BE version | 4-byte BE len | payload).
// This makes it easy to produce deterministic test files that the Ruby
// inspector (`automation/read_shm.rb`) and the C++ reader
// (`control/include/shm_reader.hpp`) can consume.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

pub fn write_shm_sample<P: AsRef<Path>>(path: P, version: u64, json: &str) -> std::io::Result<()> {
	let mut f = OpenOptions::new().create(true).write(true).truncate(true).open(path)?;
	let mut buf = Vec::new();
	buf.extend_from_slice(&version.to_be_bytes());
	let payload = json.as_bytes();
	let len = (payload.len() as u32).to_be_bytes();
	buf.extend_from_slice(&len);
	buf.extend_from_slice(payload);
	f.write_all(&buf)?;
	f.sync_all()?;
	Ok(())
}

#[cfg(test)]
mod tests {
	use super::write_shm_sample;

	#[test]
	fn create_sample_shm() {
		let _ = std::fs::remove_file("/tmp/uhtcp_sample.bin");
		write_shm_sample("/tmp/uhtcp_sample.bin", 42, r#"{"example":true}"#).unwrap();
		let s = std::fs::read_to_string("/tmp/uhtcp_sample.bin").unwrap();
		assert!(s.len() > 0);
		let _ = std::fs::remove_file("/tmp/uhtcp_sample.bin");
	}
}

