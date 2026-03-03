use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub enum SystemError {
    HalFailure,
    DeviceFailure,
    IpcFailure,
    SerializationFailure,
    AutomationFailure,
}

struct FailureInfo {
    count: u64,
    last_failure: Option<Instant>,
    backoff_until: Option<Instant>,
    circuit_open_until: Option<Instant>,
}

impl FailureInfo {
    fn new() -> Self {
        FailureInfo { count: 0, last_failure: None, backoff_until: None, circuit_open_until: None }
    }
}

/// Simple failure tracker implementing exponential backoff and a circuit breaker per domain.
pub struct FailureTracker {
    inner: Mutex<HashMap<String, FailureInfo>>,
    base_backoff_ms: u64,
    max_backoff_ms: u64,
    circuit_threshold: u64,
    circuit_open_ms: u64,
}

impl FailureTracker {
    pub fn new() -> Self {
        FailureTracker {
            inner: Mutex::new(HashMap::new()),
            base_backoff_ms: 100, // base backoff
            max_backoff_ms: 30_000,
            circuit_threshold: 5, // failures within window to open circuit
            circuit_open_ms: 60_000,
        }
    }

    pub fn should_allow(&self, domain: &str) -> bool {
        let now = Instant::now();
        let map = self.inner.lock().unwrap();
        if let Some(info) = map.get(domain) {
            if let Some(until) = info.circuit_open_until {
                if until > now { return false; }
            }
            if let Some(until) = info.backoff_until {
                if until > now { return false; }
            }
        }
        true
    }

    pub fn record_failure(&self, domain: &str) {
        let mut map = self.inner.lock().unwrap();
        let info = map.entry(domain.to_string()).or_insert_with(FailureInfo::new);
        info.count = info.count.saturating_add(1);
        info.last_failure = Some(Instant::now());

        // compute backoff: base * 2^(count-1)
        let exp = info.count.saturating_sub(1);
        let mut backoff = self.base_backoff_ms.saturating_mul(2u64.saturating_pow(exp as u32));
        if backoff > self.max_backoff_ms { backoff = self.max_backoff_ms; }
        info.backoff_until = Some(Instant::now() + Duration::from_millis(backoff));

        // open circuit if threshold exceeded
        if info.count >= self.circuit_threshold {
            info.circuit_open_until = Some(Instant::now() + Duration::from_millis(self.circuit_open_ms));
        }
    }

    pub fn record_success(&self, domain: &str) {
        let mut map = self.inner.lock().unwrap();
        map.remove(domain);
    }

    pub fn get_failure_count(&self, domain: &str) -> u64 {
        let map = self.inner.lock().unwrap();
        map.get(domain).map(|i| i.count).unwrap_or(0)
    }
}
