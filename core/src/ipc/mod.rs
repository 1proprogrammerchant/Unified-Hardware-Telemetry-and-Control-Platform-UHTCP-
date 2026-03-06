pub mod message;
pub mod unix;
pub mod shm;
pub mod bus;

pub use message::IpcMessage;
pub use bus::IpcBus;
pub use unix::{connect_unix, send_unix_message};

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    #[test]
    fn test_shm_roundtrip() {
        let tmp = "/tmp/uhtcp_test_shm.bin";
        let _ = std::fs::remove_file(tmp);
        let mut region = shm::ShmRegion::create(tmp, 4096).unwrap();
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
        let mut server = unix::UnixIpcServer::bind(socket).unwrap();
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
