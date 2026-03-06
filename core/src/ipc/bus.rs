use std::path::Path;
use std::sync::{Arc, Mutex};

use super::shm::ShmRegion;
use super::unix::UnixIpcServer;
use super::message::IpcMessage;

pub struct IpcBus {
    pub shm: Option<ShmRegion>,
    pub unix_server: Option<UnixIpcServer>,
    pub version: Arc<Mutex<u64>>,
}

impl IpcBus {
    pub fn new() -> Self { IpcBus { shm: None, unix_server: None, version: Arc::new(Mutex::new(0)) } }

    pub fn with_shm<P: AsRef<Path>>(mut self, path: P, size: usize) -> std::io::Result<Self> {
        let r = ShmRegion::create(path, size)?;
        self.shm = Some(r);
        Ok(self)
    }

    pub fn with_unix<P: AsRef<Path>>(mut self, path: P) -> std::io::Result<Self> {
        let s = UnixIpcServer::bind(path)?;
        self.unix_server = Some(s);
        Ok(self)
    }

    pub fn publish_state(&mut self, json: &str) {
        if let Some(ref mut shm) = self.shm {
            let mut v = self.version.lock().unwrap();
            *v = v.wrapping_add(1);
            let _ = shm.write_json(json, *v);
        }
        if let Some(ref server) = self.unix_server {
            let msg = IpcMessage::StateSnapshot { json: json.to_string() };
            server.broadcast(&msg);
        }
    }

    pub fn start_unix<F>(&mut self, path: &str, on_message: F) -> std::io::Result<()>
    where F: Fn(IpcMessage) + Send + 'static + Clone
    {
        let mut server = UnixIpcServer::bind(path)?;
        server.start(on_message)?;
        self.unix_server = Some(server);
        Ok(())
    }
}
