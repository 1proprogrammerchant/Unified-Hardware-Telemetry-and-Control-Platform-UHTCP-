use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;

use super::message::IpcMessage;

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

pub struct UnixIpcServer {
    path: String,
    listener: Option<UnixListener>,
    clients: Arc<Mutex<Vec<UnixStream>>>,
}

impl UnixIpcServer {
    pub fn bind<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let path = path.as_ref().to_string_lossy().to_string();
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
                                    Err(_) => { break; }
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

pub fn connect_unix<P: AsRef<Path>>(path: P) -> std::io::Result<UnixStream> {
    UnixStream::connect(path)
}

pub fn send_unix_message(stream: &mut UnixStream, msg: &IpcMessage) -> std::io::Result<()> {
    let payload = msg.to_bytes();
    write_framed(stream, &payload)
}
