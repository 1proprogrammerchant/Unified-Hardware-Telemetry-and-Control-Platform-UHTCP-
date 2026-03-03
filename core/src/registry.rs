use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DeviceStatus {
    Unknown,
    Online,
    Offline,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Device {
    pub id: u32,
    pub name: String,
    pub device_type: String,
    pub status: DeviceStatus,
    pub last_seen: u64,
    pub metadata: HashMap<String, String>,
}

impl Device {
    pub fn new(id: u32, name: impl Into<String>, device_type: impl Into<String>) -> Self {
        Device {
            id,
            name: name.into(),
            device_type: device_type.into(),
            status: DeviceStatus::Unknown,
            last_seen: now_secs(),
            metadata: HashMap::new(),
        }
    }

    pub fn touch(&mut self) {
        self.last_seen = now_secs();
        self.status = DeviceStatus::Online;
    }
}

type ListenerId = u64;

#[derive(Clone)]
pub struct DeviceRegistry {
    devices: Arc<RwLock<HashMap<u32, Device>>>,
    next_device_id: Arc<Mutex<u32>>,
    listeners: Arc<Mutex<HashMap<ListenerId, Arc<dyn Fn(&DeviceEvent) + Send + Sync>>>>,
    next_listener_id: Arc<Mutex<ListenerId>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum DeviceEventKind {
    Registered,
    Removed,
    Updated,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeviceEvent {
    pub kind: DeviceEventKind,
    pub device: Device,
    pub when: u64,
}

impl DeviceRegistry {
    pub fn new() -> Self {
        DeviceRegistry {
            devices: Arc::new(RwLock::new(HashMap::new())),
            next_device_id: Arc::new(Mutex::new(1)),
            listeners: Arc::new(Mutex::new(HashMap::new())),
            next_listener_id: Arc::new(Mutex::new(1)),
        }
    }

    /// Register a new device. If `id` is 0, an id will be allocated automatically.
    pub fn register_device(&self, mut dev: Device) -> u32 {
        let mut id_lock = self.next_device_id.lock().unwrap();
        if dev.id == 0 {
            dev.id = *id_lock;
            *id_lock = id_lock.wrapping_add(1);
        } else {
            if dev.id >= *id_lock { *id_lock = dev.id.wrapping_add(1); }
        }
        dev.touch();

        let id = dev.id;
        {
            let mut map = self.devices.write().unwrap();
            map.insert(id, dev.clone());
        }
        self.emit_event(DeviceEventKind::Registered, dev);
        id
    }

    pub fn remove_device(&self, id: u32) -> Option<Device> {
        let removed = { let mut map = self.devices.write().unwrap(); map.remove(&id) };
        if let Some(dev) = removed.clone() { self.emit_event(DeviceEventKind::Removed, dev); }
        removed
    }

    pub fn find_device(&self, id: u32) -> Option<Device> {
        let map = self.devices.read().unwrap(); map.get(&id).cloned()
    }

    pub fn enumerate_devices(&self) -> Vec<Device> {
        let map = self.devices.read().unwrap(); map.values().cloned().collect()
    }

    pub fn device_exists(&self, id: u32) -> bool {
        let map = self.devices.read().unwrap(); map.contains_key(&id)
    }

    pub fn update_device_seen(&self, id: u32) -> Option<()> {
        let mut map = self.devices.write().unwrap();
        if let Some(d) = map.get_mut(&id) { d.touch(); return Some(()); }
        None
    }

    pub fn update_device_metadata(&self, id: u32, k: impl Into<String>, v: impl Into<String>) -> Option<()> {
        let mut map = self.devices.write().unwrap();
        if let Some(d) = map.get_mut(&id) { d.metadata.insert(k.into(), v.into()); self.emit_event(DeviceEventKind::Updated, d.clone()); return Some(()); }
        None
    }

    pub fn reconcile_from_gpio_mask(&self, mask: u32) {
        // simple heuristic: devices with ids matching pins become online
        let mut map = self.devices.write().unwrap();
        for (&id, dev) in map.iter_mut() {
            let pin = id as u32; // mapping assumption
            let online = ((mask >> pin) & 1) != 0;
            dev.status = if online { DeviceStatus::Online } else { DeviceStatus::Offline };
            dev.last_seen = now_secs();
        }
    }

    fn emit_event(&self, kind: DeviceEventKind, dev: Device) {
        let when = now_secs();
        let ev = DeviceEvent { kind, device: dev, when };
        let listeners = self.listeners.lock().unwrap();
        for (_id, cb) in listeners.iter() {
            let cb = cb.clone();
            // invoke asynchronously
            let evc = ev.clone();
            std::thread::spawn(move || {
                (cb)(&evc);
            });
        }
    }

    /// Subscribe to device events. Returns listener id for later removal.
    pub fn register_listener<F>(&self, cb: F) -> ListenerId where F: Fn(&DeviceEvent) + Send + Sync + 'static {
        let mut lid = self.next_listener_id.lock().unwrap();
        let id = *lid;
        *lid = lid.wrapping_add(1);
        let mut listeners = self.listeners.lock().unwrap();
        listeners.insert(id, Arc::new(cb));
        id
    }

    pub fn unregister_listener(&self, id: ListenerId) -> bool {
        let mut listeners = self.listeners.lock().unwrap(); listeners.remove(&id).is_some()
    }

    /// Load devices from a JSON config file. Returns number loaded.
    pub fn load_from_config<P: AsRef<Path>>(&self, path: P) -> Result<usize, String> {
        let path = path.as_ref();
        if !path.exists() { return Ok(0); }
        let mut f = File::open(path).map_err(|e| format!("open config: {}", e))?;
        let mut buf = String::new(); f.read_to_string(&mut buf).map_err(|e| format!("read config: {}", e))?;
        let parsed: serde_json::Value = serde_json::from_str(&buf).map_err(|e| format!("parse json: {}", e))?;
        let arr = parsed.get("devices").and_then(|v| v.as_array()).ok_or("no devices array")?;
        let mut loaded = 0usize;
        for item in arr.iter() {
            if let (Some(id), Some(name), Some(typ)) = (item.get("id"), item.get("name"), item.get("type")) {
                let id = id.as_u64().unwrap_or(0) as u32;
                let name = name.as_str().unwrap_or("").to_string();
                let typ = typ.as_str().unwrap_or("").to_string();
                let dev = Device { id, name, device_type: typ, status: DeviceStatus::Unknown, last_seen: now_secs(), metadata: HashMap::new() };
                self.register_device(dev);
                loaded += 1;
            }
        }
        Ok(loaded)
    }

    /// Save current devices into config file
    pub fn save_to_config<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let devices = self.enumerate_devices();
        let wrapper = serde_json::json!({ "devices": devices });
        let mut f = OpenOptions::new().create(true).write(true).truncate(true).open(path).map_err(|e| format!("open: {}", e))?;
        let s = serde_json::to_string_pretty(&wrapper).map_err(|e| format!("serialize: {}", e))?;
        f.write_all(s.as_bytes()).map_err(|e| format!("write: {}", e))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_find_remove() {
        let reg = DeviceRegistry::new();
        let dev = Device::new(0, "Test", "sensor");
        let id = reg.register_device(dev);
        assert!(reg.device_exists(id));
        let found = reg.find_device(id).unwrap();
        assert_eq!(found.name, "Test");
        let removed = reg.remove_device(id);
        assert!(removed.is_some());
        assert!(!reg.device_exists(id));
    }

    #[test]
    fn listeners_receive_events() {
        let reg = DeviceRegistry::new();
        let received = Arc::new(Mutex::new(Vec::new()));
        let rcv = received.clone();
        let lid = reg.register_listener(move |ev: &DeviceEvent| {
            let mut v = rcv.lock().unwrap();
            v.push(ev.kind.clone());
        });
        let dev = Device::new(0, "L", "a");
        let id = reg.register_device(dev);
        std::thread::sleep(std::time::Duration::from_millis(10));
        let v = received.lock().unwrap();
        assert!(!v.is_empty());
        reg.unregister_listener(lid);
        let _ = reg.remove_device(id);
    }
}
