use crate::ffi;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct DeviceState {
    pub id: u32,
    pub name: String,
    pub device_type: String,
    pub last_seen: u64, // unix epoch seconds
    pub metadata: HashMap<String, String>,
}

impl DeviceState {
    pub fn new(id: u32, name: impl Into<String>, device_type: impl Into<String>) -> Self {
        DeviceState {
            id,
            name: name.into(),
            device_type: device_type.into(),
            last_seen: now_secs(),
            metadata: HashMap::new(),
        }
    }

    pub fn touch(&mut self) { self.last_seen = now_secs(); }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct HardwareState {
    // core metrics
    pub cpu_temperature: f32,
    pub memory_total: u64,
    pub memory_free: u64,
    pub uptime: u64,

    // GPIO states (sparse map for safety)
    pub gpio_states: HashMap<u32, bool>,

    // registered devices (mirrors registry but kept here for snapshotting)
    pub devices: HashMap<u32, DeviceState>,

    // last update timestamp
    pub last_update: u64,

    // optional internal mutex for non-atomic operations
    #[serde(skip)]
    lock: Arc<Mutex<()>>,
}

fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

impl Default for HardwareState {
    fn default() -> Self {
        HardwareState {
            cpu_temperature: 0.0,
            memory_total: 0,
            memory_free: 0,
            uptime: 0,
            gpio_states: HashMap::new(),
            devices: HashMap::new(),
            last_update: now_secs(),
            lock: Arc::new(Mutex::new(())),
        }
    }
}

impl HardwareState {
    pub fn snapshot(&self) -> HardwareState {
        self.clone()
    }

    pub fn serialize(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn deserialize(s: &str) -> Result<HardwareState, serde_json::Error> {
        serde_json::from_str(s)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.cpu_temperature.is_nan() || self.cpu_temperature < -100.0 || self.cpu_temperature > 1000.0 {
            return Err(format!("invalid cpu temperature: {}", self.cpu_temperature));
        }
        if self.memory_total < self.memory_free {
            return Err(format!("memory free > total: {} > {}", self.memory_free, self.memory_total));
        }
        Ok(())
    }

    pub fn update_cpu_temp(&mut self, temp: f32) {
        let _g = self.lock.lock().unwrap();
        self.cpu_temperature = temp;
        self.last_update = now_secs();
    }

    pub fn update_memory(&mut self, total: u64, free: u64) {
        let _g = self.lock.lock().unwrap();
        self.memory_total = total;
        self.memory_free = free;
        self.last_update = now_secs();
    }

    pub fn update_gpio(&mut self, pin: u32, value: bool) {
        let _g = self.lock.lock().unwrap();
        self.gpio_states.insert(pin, value);
        self.last_update = now_secs();
    }

    pub fn set_gpio_mask(&mut self, mask: u32) {
        let _g = self.lock.lock().unwrap();
        self.gpio_states.clear();
        for pin in 0..32u32 {
            let v = ((mask >> pin) & 1) != 0;
            self.gpio_states.insert(pin, v);
        }
        self.last_update = now_secs();
    }

    pub fn get_snapshot_json(&self) -> Result<String, serde_json::Error> {
        self.serialize()
    }

    pub fn register_device(&mut self, dev: DeviceState) {
        let _g = self.lock.lock().unwrap();
        self.devices.insert(dev.id, dev);
        self.last_update = now_secs();
    }

    pub fn remove_device(&mut self, id: u32) -> Option<DeviceState> {
        let _g = self.lock.lock().unwrap();
        let r = self.devices.remove(&id);
        self.last_update = now_secs();
        r
    }

    pub fn find_device(&self, id: u32) -> Option<DeviceState> {
        self.devices.get(&id).cloned()
    }

    pub fn enumerate_devices(&self) -> Vec<DeviceState> {
        self.devices.values().cloned().collect()
    }

    pub fn device_exists(&self, id: u32) -> bool {
        self.devices.contains_key(&id)
    }

    /// Update state from a HAL snapshot (FFI-provided)
    pub fn update_from_snapshot(&mut self, snap: &ffi::hal_snapshot_t) {
        // we already have &mut self so no internal lock is required here
        // copy cpu temp and uptime
        self.cpu_temperature = snap.cpu_temperature;
        self.uptime = snap.uptime;
        // apply gpio mask via public setter (it will lock internally)
        self.set_gpio_mask(snap.gpio_state);
        self.last_update = now_secs();
    }

    /// Merge another state in (used for restores or IPC merging)
    pub fn merge(&mut self, other: &HardwareState) {
        let _g = self.lock.lock().unwrap();
        if other.cpu_temperature != 0.0 { self.cpu_temperature = other.cpu_temperature; }
        if other.memory_total != 0 { self.memory_total = other.memory_total; }
        if other.memory_free != 0 { self.memory_free = other.memory_free; }
        for (k, v) in &other.gpio_states { self.gpio_states.insert(*k, *v); }
        for (id, dev) in &other.devices { self.devices.insert(*id, dev.clone()); }
        self.last_update = now_secs();
    }

    /// Lightweight read accessor for languages binding via FFI if needed
    pub fn get_cpu_temp(&self) -> f32 { self.cpu_temperature }
    pub fn get_uptime(&self) -> u64 { self.uptime }
    pub fn get_gpio_state_mask(&self) -> u32 {
        let mut mask = 0u32;
        for (&pin, &val) in &self.gpio_states { if val { mask |= 1u32 << pin; } }
        mask
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_and_serialize() {
        let mut s = HardwareState::default();
        s.update_cpu_temp(55.5);
        s.update_memory(1024*1024*1024, 256*1024*1024);
        s.update_gpio(4, true);
        s.register_device(DeviceState::new(1, "CPU", "temperature"));
        let js = s.serialize().unwrap();
        let parsed = HardwareState::deserialize(&js).unwrap();
        assert_eq!(parsed.cpu_temperature, 55.5);
        assert!(parsed.device_exists(1));
    }

    #[test]
    fn test_update_from_snapshot() {
        let mut s = HardwareState::default();
        let snap = ffi::hal_snapshot_t { cpu_temperature: 11.0, uptime: 123, gpio_state: 0b1 << 7 };
        s.update_from_snapshot(&snap);
        assert_eq!(s.get_cpu_temp(), 11.0);
        assert_eq!(s.get_uptime(), 123);
        assert_eq!(s.get_gpio_state_mask(), 1u32 << 7);
    }
}
