use serde::Serialize;
use std::sync::Mutex;

#[derive(Default, Serialize, Clone)]
pub struct HealthSnapshot {
    pub system: String,
    pub scheduler_drift_ms: f64,
    pub hal_failures: u64,
    pub device_failures: u64,
}

pub struct HealthMonitor {
    inner: Mutex<HealthSnapshot>,
}

impl HealthMonitor {
    pub fn new() -> Self {
        HealthMonitor { inner: Mutex::new(HealthSnapshot { system: "healthy".into(), scheduler_drift_ms: 0.0, hal_failures: 0, device_failures: 0 }) }
    }

    pub fn record_hal_failure(&self) {
        let mut s = self.inner.lock().unwrap();
        s.hal_failures = s.hal_failures.saturating_add(1);
        if s.hal_failures > 0 { s.system = "degraded".into(); }
    }

    pub fn record_device_failure(&self) {
        let mut s = self.inner.lock().unwrap();
        s.device_failures = s.device_failures.saturating_add(1);
        if s.device_failures > 0 { s.system = "degraded".into(); }
    }

    pub fn record_scheduler_drift(&self, ms: f64) {
        let mut s = self.inner.lock().unwrap();
        s.scheduler_drift_ms = ms;
    }

    pub fn snapshot(&self) -> HealthSnapshot {
        self.inner.lock().unwrap().clone()
    }
}
