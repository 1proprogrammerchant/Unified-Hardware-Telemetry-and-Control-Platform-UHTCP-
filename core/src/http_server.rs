use std::sync::{Arc, RwLock, Mutex};
use std::thread;
use tiny_http::{Server, Response, Request, Header};

use crate::state::HardwareState;
use crate::registry::DeviceRegistry;
use crate::health::HealthMonitor;
use crate::metrics::Metrics;
use crate::ipc;

pub fn start_server(port: u16, state: Arc<RwLock<HardwareState>>, registry: Arc<Mutex<DeviceRegistry>>, health: Option<Arc<HealthMonitor>>, metrics: Option<Arc<Metrics>>, _ipc: Option<Arc<Mutex<ipc::IpcBus>>>) {
    let addr = format!("0.0.0.0:{}", port);
    thread::spawn(move || {
        let server = match Server::http(&addr) {
            Ok(s) => s,
            Err(e) => { eprintln!("http_server: failed to bind {}: {}", addr, e); return; }
        };
        println!("http_server: listening on {}", addr);
        for req in server.incoming_requests() {
            handle_request(req, state.clone(), registry.clone(), health.clone(), metrics.clone());
        }
    });
}

fn handle_request(request: Request, state: Arc<RwLock<HardwareState>>, _registry: Arc<Mutex<DeviceRegistry>>, health: Option<Arc<HealthMonitor>>, metrics: Option<Arc<Metrics>>) {
    let url = request.url().to_string();
    match url.as_str() {
        "/health" | "/api/v1/health" => {
                if let Some(h) = health {
                    let snap = h.snapshot();
                    if let Ok(js) = serde_json::to_string(&snap) {
                        let hdr = Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap();
                        let resp = Response::from_string(js).with_header(hdr);
                        let _ = request.respond(resp);
                        return;
                    }
            }
            let hdr = Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap();
            let resp = Response::from_string("{\"system\":\"unknown\"}").with_header(hdr);
            let _ = request.respond(resp);
        }
        "/state" | "/api/v1/state" => {
            let s = state.read().unwrap();
            if let Ok(js) = s.get_snapshot_json() {
                let hdr = Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap();
                let resp = Response::from_string(js).with_header(hdr);
                let _ = request.respond(resp);
                return;
            }
            let hdr = Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap();
            let resp = Response::from_string("{}").with_header(hdr);
            let _ = request.respond(resp);
        }
        "/api/v1/metrics" | "/metrics" => {
            if let Some(m) = metrics {
                let snap = m.snapshot();
                if let Ok(js) = serde_json::to_string(&snap) {
                    let hdr = Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap();
                    let resp = Response::from_string(js).with_header(hdr);
                    let _ = request.respond(resp);
                    return;
                }
            }
            let hdr = Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap();
            let resp = Response::from_string("{}" ).with_header(hdr);
            let _ = request.respond(resp);
        }
        _ => {
            let resp = Response::from_string("not found").with_status_code(404);
            let _ = request.respond(resp);
        }
    }
}
