use serde::{Deserialize, Serialize};
use serde_json;

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
