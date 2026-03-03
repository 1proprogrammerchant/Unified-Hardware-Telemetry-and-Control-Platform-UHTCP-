use memmap2::{MmapMut, MmapOptions};
use serde::{Deserialize, Serialize};
use serde_json;

use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;

/// IPC Message definitions
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum IpcMessage {
    StateSnapshot { json: String },
    Command { name: String, payload: serde_json::Value },
    Ack { id: u64 },
    Heartbeat { ts: u64 },
}

impl IpcMessage {
    pub fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap_or_else(|_| b"null".to_vec())
    }

    pub fn from_bytes(b: &[u8]) -> Option<IpcMessage> {
        serde_json::from_slice(b).ok()
    }
}

/// Simple length-prefixed framing: [u32 BE len][payload bytes]
fn write_framed(stream: &mut UnixStream, payload: &[u8]) -> std::io::Result<()> {
    let len = (payload.len() as u32).to_be_bytes();
    stream.write_all(&len)?;
    stream.write_all(payload)?;
    stream.flush()?;
    Ok(())
}

fn read_framed(stream: &mut UnixStream) -> std::io::Result<Vec<u8>> {
    let mut lenb = [0u8; 4];
    stream.read_exact(&mut lenb)?;
    let len = u32::from_be_bytes(lenb) as usize;
    let mut buf = vec![0u8; len];
    stream.read_exact(&mut buf)?;
    Ok(buf)
}

/// Unix domain socket server that accepts connections and dispatches messages to handler
pub struct UnixIpcServer {
    path: String,
    listener: Option<UnixListener>,
    clients: Arc<Mutex<Vec<UnixStream>>>,
}

impl UnixIpcServer {
    pub fn bind<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let path = path.as_ref().to_string_lossy().to_string();
        // remove existing socket if present
        let _ = std::fs::remove_file(&path);
        let listener = UnixListener::bind(&path)?;
        let clients = Arc::new(Mutex::new(Vec::new()));
        Ok(UnixIpcServer { path, listener: Some(listener), clients })
    }

    pub fn start<F>(&mut self, on_message: F) -> std::io::Result<()>
    where
        F: Fn(IpcMessage) + Send + 'static + Clone,
    {
        let listener = self.listener.take().expect("listener already taken");
        let clients = self.clients.clone();
        thread::spawn(move || {
            for stream_res in listener.incoming() {
                match stream_res {
                    Ok(mut stream) => {
                        let on_message = on_message.clone();
                        clients.lock().unwrap().push(stream.try_clone().unwrap());
                        thread::spawn(move || {
                            loop {
                                match read_framed(&mut stream) {
                                    Ok(buf) => {
                                        if let Some(msg) = IpcMessage::from_bytes(&buf) {
                                            on_message(msg);
                                        }
                                    }
                                    Err(_) => {
                                        // connection closed or error
                                        break;
                                    }
                                }
                            }
                        });
                    }
                    Err(_) => continue,
                }
            }
        });
        Ok(())
    }

    pub fn broadcast(&self, msg: &IpcMessage) {
        let payload = msg.to_bytes();
        let mut clients = self.clients.lock().unwrap();
        clients.retain(|c| {
            let mut ok = true;
            if let Err(_) = write_framed(&mut c.try_clone().unwrap(), &payload) { ok = false; }
            ok
        });
    }
}

/// Connect as a client to a unix socket server
pub fn connect_unix<P: AsRef<Path>>(path: P) -> std::io::Result<UnixStream> {
    UnixStream::connect(path)
}

/// Helper to send a message over a UnixStream
pub fn send_unix_message(stream: &mut UnixStream, msg: &IpcMessage) -> std::io::Result<()> {
    let payload = msg.to_bytes();
    write_framed(stream, &payload)
}

/// Memory-mapped file based shared region. This is a simple single-writer, multi-reader region
pub struct ShmRegion {
    file: File,
    map: MmapMut,
    size: usize,
}

impl ShmRegion {
    /// Create or open a file-backed region at `path` with given size
    pub fn create<P: AsRef<Path>>(path: P, size: usize) -> std::io::Result<Self> {
        let p = path.as_ref();
        let file = OpenOptions::new().read(true).write(true).create(true).open(p)?;
        file.set_len(size as u64)?;
        let map = unsafe { MmapOptions::new().len(size).map_mut(&file)? };
        Ok(ShmRegion { file, map, size })
    }

    /// Write a JSON message to the region with a simple header: [u64 version][u32 len][payload]
    pub fn write_json(&mut self, json: &str, version: u64) -> std::io::Result<()> {
        let payload = json.as_bytes();
        let needed = 8 + 4 + payload.len();
        if needed > self.size { return Err(std::io::Error::new(std::io::ErrorKind::Other, "shm too small")); }
        // write version (BE)
        self.map[0..8].copy_from_slice(&version.to_be_bytes());
        let len = (payload.len() as u32).to_be_bytes();
        self.map[8..12].copy_from_slice(&len);
        self.map[12..12+payload.len()].copy_from_slice(payload);
        self.map.flush()?;
        Ok(())
    }

    /// Read current JSON payload and version
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

/// Simple high-level bus that can publish state snapshots into shared memory and notify via unix socket
pub struct IpcBus {
    shm: Option<ShmRegion>,
    unix_server: Option<UnixIpcServer>,
    version: Arc<Mutex<u64>>,
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

    /// Publish a state snapshot (JSON) into shared memory and broadcast an IPC message
    pub fn publish_state(&mut self, json: &str) {
        // increment version
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

    /// Start unix server message handling loop with callback
    pub fn start_unix<F>(&mut self, path: &str, on_message: F) -> std::io::Result<()>
    where F: Fn(IpcMessage) + Send + 'static + Clone
    {
        let mut server = UnixIpcServer::bind(path)?;
        server.start(on_message)?;
        self.unix_server = Some(server);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    #[test]
    fn test_shm_roundtrip() {
        let tmp = "/tmp/uhtcp_test_shm.bin";
        let _ = std::fs::remove_file(tmp);
        let mut region = ShmRegion::create(tmp, 4096).unwrap();
        let json = r#"{"cpu": 1}"#;
        region.write_json(json, 1).unwrap();
        let (v, s) = region.read_json().unwrap();
        assert_eq!(v, 1);
        assert_eq!(s, json.to_string());
        let _ = std::fs::remove_file(tmp);
    }

    #[test]
    fn test_unix_ipc_server_client() {
        let socket = "/tmp/uhtcp_test_socket.sock";
        let _ = std::fs::remove_file(socket);
        let received = Arc::new(Mutex::new(Vec::new()));
        let recv_clone = received.clone();
        let mut server = UnixIpcServer::bind(socket).unwrap();
        server.start(move |msg| {
            let mut v = recv_clone.lock().unwrap();
            v.push(format!("{:?}", msg));
        }).unwrap();
        // give server a moment
        std::thread::sleep(Duration::from_millis(50));
        let mut client = connect_unix(socket).unwrap();
        let msg = IpcMessage::Heartbeat { ts: 123 };
        send_unix_message(&mut client, &msg).unwrap();
        std::thread::sleep(Duration::from_millis(50));
        let v = received.lock().unwrap();
        assert!(!v.is_empty());
        let _ = std::fs::remove_file(socket);
    }
}
