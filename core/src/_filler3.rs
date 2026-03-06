use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::Path;

pub fn write_shm_sample<P: AsRef<Path>>(path: P, version: u64, json: &str) -> std::io::Result<()> {
	let mut f = OpenOptions::new().create(true).write(true).truncate(true).open(path)?;
	let mut buf = Vec::with_capacity(12 + json.len());
	buf.extend_from_slice(&version.to_be_bytes());
	let payload = json.as_bytes();
	buf.extend_from_slice(&(payload.len() as u32).to_be_bytes());
	buf.extend_from_slice(payload);
	f.write_all(&buf)?;
	f.sync_all()?;
	Ok(())
}

pub fn read_shm<P: AsRef<Path>>(path: P) -> std::io::Result<(u64, String)> {
	let mut data = Vec::new();
	let mut f = OpenOptions::new().read(true).open(path)?;
	f.read_to_end(&mut data)?;
	if data.len() < 12 { return Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "shm file too small")); }
	let mut vb = [0u8;8]; vb.copy_from_slice(&data[0..8]);
	let version = u64::from_be_bytes(vb);
	let mut lb = [0u8;4]; lb.copy_from_slice(&data[8..12]);
	let len = u32::from_be_bytes(lb) as usize;
	if data.len() < 12 + len { return Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "shm payload truncated")); }
	let payload = &data[12..12+len];
	Ok((version, String::from_utf8_lossy(payload).to_string()))
}

#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn write_then_read() {
		let tmp = "/tmp/uhtcp_sample.rs.bin";
		let _ = std::fs::remove_file(tmp);
		write_shm_sample(tmp, 7, r#"{"test":true}"#).unwrap();
		let (v, s) = read_shm(tmp).unwrap();
		assert_eq!(v, 7);
		assert!(s.contains("test"));
		let _ = std::fs::remove_file(tmp);
	}
}

