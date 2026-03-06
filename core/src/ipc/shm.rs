use memmap2::{MmapMut, MmapOptions};
use std::fs::{File, OpenOptions};
use std::path::Path;

pub struct ShmRegion {
    pub file: File,
    pub map: MmapMut,
    pub size: usize,
}

impl ShmRegion {
    pub fn create<P: AsRef<Path>>(path: P, size: usize) -> std::io::Result<Self> {
        let p = path.as_ref();
        let file = OpenOptions::new().read(true).write(true).create(true).open(p)?;
        file.set_len(size as u64)?;
        let map = unsafe { MmapOptions::new().len(size).map_mut(&file)? };
        Ok(ShmRegion { file, map, size })
    }

    pub fn write_json(&mut self, json: &str, version: u64) -> std::io::Result<()> {
        let payload = json.as_bytes();
        let needed = 8 + 4 + payload.len();
        if needed > self.size { return Err(std::io::Error::new(std::io::ErrorKind::Other, "shm too small")); }
        self.map[0..8].copy_from_slice(&version.to_be_bytes());
        let len = (payload.len() as u32).to_be_bytes();
        self.map[8..12].copy_from_slice(&len);
        self.map[12..12+payload.len()].copy_from_slice(payload);
        self.map.flush()?;
        Ok(())
    }

    pub fn read_json(&self) -> std::io::Result<(u64, String)> {
        let mut vb = [0u8; 8]; vb.copy_from_slice(&self.map[0..8]);
        let version = u64::from_be_bytes(vb);
        let mut lb = [0u8; 4]; lb.copy_from_slice(&self.map[8..12]);
        let len = u32::from_be_bytes(lb) as usize;
        if len == 0 || 12 + len > self.size { return Ok((version, String::new())); }
        let payload = &self.map[12..12+len];
        let s = String::from_utf8_lossy(payload).to_string();
        Ok((version, s))
    }
}
