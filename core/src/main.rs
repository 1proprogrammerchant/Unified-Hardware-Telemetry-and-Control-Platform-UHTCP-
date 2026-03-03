mod engine;
mod scheduler;
mod state;
mod registry;
mod ffi;
mod ipc;
mod logger;
mod errors;
mod health;
mod http_server;
mod metrics;

use state::HardwareState;
use std::sync::{Arc, RwLock, Mutex};
use std::time::Duration;

fn main() {
    // initialize core components
    unsafe { ffi::hal_init(); }
    logger::log("uhtcp core starting");

    let state = Arc::new(RwLock::new(HardwareState::default()));
    let registry = Arc::new(Mutex::new(registry::DeviceRegistry::new()));

    // health monitor and metrics for tracking runtime health and performance
    let health = Arc::new(health::HealthMonitor::new());
    let metrics = Arc::new(metrics::Metrics::new());

    // try to create an IPC bus (best-effort)
    let ipc_bus = match ipc::IpcBus::new().with_shm("/tmp/uhtcp_state.bin", 64 * 1024) {
        Ok(b) => Some(Arc::new(Mutex::new(b)) ),
        Err(_) => None,
    };

    // construct scheduler and register poll task
    let sched = scheduler::Scheduler::new(Duration::from_millis(100), state.clone(), registry.clone(), ipc_bus.clone(), Some(health.clone()), Some(metrics.clone()));
    sched.register_poll_task(1, Duration::from_millis(100));
    sched.start();

    // start a tiny HTTP endpoint to expose health and basic state
    http_server::start_server(9000, state.clone(), registry.clone(), Some(health.clone()), Some(metrics.clone()), ipc_bus.clone());

    // start engine
    engine::start_engine();

    logger::log("uhtcp core running");
    loop { std::thread::sleep(std::time::Duration::from_secs(60)); }
}
