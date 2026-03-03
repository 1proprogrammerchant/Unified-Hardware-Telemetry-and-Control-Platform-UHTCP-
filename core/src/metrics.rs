use serde::Serialize;
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Serialize, Clone)]
pub struct MetricsSnapshot {
    pub counters: HashMap<String, u64>,
    pub gauges: HashMap<String, f64>,
}

pub struct Metrics {
    inner: Mutex<(HashMap<String, u64>, HashMap<String, f64>)>,
}

impl Metrics {
    pub fn new() -> Self {
        Metrics { inner: Mutex::new((HashMap::new(), HashMap::new())) }
    }

    pub fn incr(&self, key: &str, v: u64) {
        let mut guard = self.inner.lock().unwrap();
        let c = &mut guard.0;
        *c.entry(key.to_string()).or_insert(0) = c.get(&key.to_string()).unwrap_or(&0).saturating_add(v);
    }

    pub fn gauge(&self, key: &str, v: f64) {
        let mut guard = self.inner.lock().unwrap();
        let g = &mut guard.1;
        g.insert(key.to_string(), v);
    }

    pub fn snapshot(&self) -> MetricsSnapshot {
        let guard = self.inner.lock().unwrap();
        MetricsSnapshot { counters: guard.0.clone(), gauges: guard.1.clone() }
    }
}
