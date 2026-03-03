use crate::ffi;
use crate::state::HardwareState;
use crate::logger;
use crate::registry::DeviceRegistry;
use crate::ipc;
use crate::errors::FailureTracker;
use crate::health::HealthMonitor;
use crate::metrics::Metrics;

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::sync::{Arc, Mutex, RwLock, Condvar};
use std::thread;
use std::time::{Duration, Instant};

// Priority for tasks. Lower number == higher priority.
pub type Priority = u8;

#[derive(Clone, Debug)]
pub struct ScheduledTask {
    pub id: u64,
    pub name: String,
    pub interval: Duration,
    pub priority: Priority,
    pub last_run: Option<Instant>,
}

impl ScheduledTask {
    pub fn next_run_in(&self) -> Duration {
        match self.last_run {
            None => Duration::from_secs(0),
            Some(t) => {
                let elapsed = t.elapsed();
                if elapsed >= self.interval { Duration::from_secs(0) } else { self.interval - elapsed }
            }
        }
    }
}

// Internal queue item for priority scheduling
#[derive(Clone)]
struct QueueItem {
    next_run: Instant,
    priority: Priority,
    task: ScheduledTask,
}

impl Eq for QueueItem {}
impl PartialEq for QueueItem {
    fn eq(&self, other: &Self) -> bool {
        self.next_run == other.next_run && self.priority == other.priority && self.task.id == other.task.id
    }
}

impl Ord for QueueItem {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse order because BinaryHeap is a max-heap; we want earliest next_run and higher priority
        match other.next_run.cmp(&self.next_run) {
            Ordering::Equal => other.priority.cmp(&self.priority),
            o => o,
        }
    }
}

impl PartialOrd for QueueItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}

pub struct Scheduler {
    poll_interval: Duration,
    queue: Arc<Mutex<BinaryHeap<QueueItem>>>,
    tasks: Arc<Mutex<HashMap<u64, ScheduledTask>>>,
    running: Arc<(Mutex<bool>, Condvar)>,
    state: Arc<RwLock<HardwareState>>,
    registry: Arc<Mutex<DeviceRegistry>>,
    ipc: Option<Arc<Mutex<ipc::IpcBus>>>,
    failure_tracker: Arc<FailureTracker>,
    health: Option<Arc<HealthMonitor>>,
    metrics: Option<Arc<Metrics>>,
}

impl Scheduler {
    pub fn new(poll_interval: Duration, state: Arc<RwLock<HardwareState>>, registry: Arc<Mutex<DeviceRegistry>>, ipc: Option<Arc<Mutex<ipc::IpcBus>>>, health: Option<Arc<HealthMonitor>>, metrics: Option<Arc<Metrics>>) -> Self {
        Scheduler {
            poll_interval,
            queue: Arc::new(Mutex::new(BinaryHeap::new())),
            tasks: Arc::new(Mutex::new(HashMap::new())),
            running: Arc::new((Mutex::new(false), Condvar::new())),
            state,
            registry,
            ipc,
            failure_tracker: Arc::new(FailureTracker::new()),
            health,
            metrics,
        }
    }

    pub fn start(&self) {
        let (lock, cvar) = &*self.running;
        {
            let mut started = lock.lock().unwrap();
            *started = true;
            cvar.notify_all();
        }

        let queue = self.queue.clone();
        let tasks = self.tasks.clone();
        let running = self.running.clone();
        let poll = self.poll_interval;
        let state = self.state.clone();
        let registry = self.registry.clone();
        let ipc = self.ipc.clone();

        let failure_tracker = self.failure_tracker.clone();
        let health_clone = self.health.clone();
        let metrics = self.metrics.clone();

        thread::spawn(move || {
            logger::log("scheduler: thread started");
            loop {
                {
                    let (lk, _cv) = &*running;
                    let started = lk.lock().unwrap();
                    if !*started { break; }
                    drop(started);
                }

                let res = std::panic::catch_unwind(|| {
                    let now = Instant::now();
                    // Execute due tasks
                    let mut to_execute: Vec<ScheduledTask> = Vec::new();
                    {
                        let mut q = queue.lock().unwrap();
                        while let Some(item) = q.peek() {
                            if item.next_run <= now {
                                let item = q.pop().unwrap();
                                to_execute.push(item.task.clone());
                            } else { break; }
                        }
                    }

                    for task in to_execute {
                        // run in separate thread to avoid blocking
                        let state_clone = state.clone();
                        let reg_clone = registry.clone();
                        let ipc_clone = ipc.clone();
                        let task_for_thread = task.clone();
                        let ft = failure_tracker.clone();
                        let health_inner = health_clone.clone();
                        let metrics_inner = metrics.clone();
                        thread::spawn(move || {
                            let task_name = task_for_thread.name.clone();
                            let exec_res = std::panic::catch_unwind(move || {
                                Scheduler::execute_task(task_for_thread, state_clone, reg_clone, ipc_clone, ft.clone(), health_inner.clone(), metrics_inner.clone());
                            });
                            if exec_res.is_err() {
                                logger::log(&format!("scheduler: task '{}' panicked", task_name));
                            }
                        });
                        // reschedule using the original task value
                        let mut tks = tasks.lock().unwrap();
                        if let Some(t) = tks.get_mut(&task.id) {
                            t.last_run = Some(Instant::now());
                            let next = QueueItem { next_run: Instant::now() + t.interval, priority: t.priority, task: t.clone() };
                            let mut q = queue.lock().unwrap();
                            q.push(next);
                        }
                    }
                }); // end catch_unwind for loop body

                if res.is_err() {
                    logger::log("scheduler: internal panic recovered, continuing loop");
                }

                // sleep deterministic amount
                thread::sleep(poll);
            }
            logger::log("scheduler: thread exiting");
        });
    }

    pub fn stop(&self) {
        let (lock, cvar) = &*self.running;
        let mut started = lock.lock().unwrap();
        *started = false;
        cvar.notify_all();
    }

    pub fn register_task(&self, mut task: ScheduledTask) -> u64 {
        let id = task.id;
        task.last_run = None;
        {
            let mut tks = self.tasks.lock().unwrap();
            tks.insert(id, task.clone());
        }
        let item = QueueItem { next_run: Instant::now() + task.interval, priority: task.priority, task };
        let mut q = self.queue.lock().unwrap();
        q.push(item);
        id
    }

    pub fn unregister_task(&self, id: u64) -> bool {
        let mut tks = self.tasks.lock().unwrap();
        tks.remove(&id).is_some()
    }

    fn execute_task(task: ScheduledTask, state: Arc<RwLock<HardwareState>>, _registry: Arc<Mutex<DeviceRegistry>>, ipc: Option<Arc<Mutex<ipc::IpcBus>>>, failure_tracker: Arc<FailureTracker>, health: Option<Arc<HealthMonitor>>, metrics: Option<Arc<Metrics>>) {
        // central execution point for tasks; tasks are meant to be small, non-blocking
        logger::log(&format!("execute_task: {} ({})", task.name, task.id));
        // example: if task name matches a known action, perform an action
        if task.name == "poll_hal" {
            // check circuit breaker / backoff
            if !failure_tracker.should_allow("hal") {
                logger::log("execute_task: hal is in backoff/circuit-open, skipping poll_hal");
                if let Some(h) = health { h.record_scheduler_drift(0.0); }
                return;
            }
            // call FFI to read snapshot and update state
            let start = Instant::now();
            unsafe {
                let mut snap = ffi::hal_snapshot_t { cpu_temperature: 0.0, uptime: 0, gpio_state: 0 };
                if ffi::hal_read_snapshot(&mut snap as *mut ffi::hal_snapshot_t) == 0 {
                    let mut s = state.write().unwrap();
                    s.update_from_snapshot(&snap);
                    failure_tracker.record_success("hal");
                    let elapsed = start.elapsed().as_secs_f64() * 1000.0;
                    if let Some(m) = &metrics { m.gauge("poll_hal_duration_ms", elapsed); m.incr("polls_success", 1); }
                    // publish state via IPC if present
                    if let Some(ipc_bus) = ipc {
                        if let Ok(js) = s.get_snapshot_json() {
                            if let Ok(mut bus) = ipc_bus.lock() {
                                let ipc_start = Instant::now();
                                bus.publish_state(&js);
                                if let Some(m) = &metrics { m.gauge("ipc_publish_latency_ms", ipc_start.elapsed().as_secs_f64() * 1000.0); m.incr("ipc_publish", 1); }
                            }
                        }
                    }
                } else {
                    logger::log("execute_task: hal_read_snapshot failed");
                    failure_tracker.record_failure("hal");
                    if let Some(h) = health { h.record_hal_failure(); }
                    if let Some(m) = &metrics { m.incr("polls_failed", 1); }
                }
            }
        } else {
            // placeholder: invoke automation hooks or device-specific callbacks
            logger::log(&format!("execute_task: unknown task '{}'", task.name));
        }
    }

    pub fn handle_error(&self, err: &str) {
        logger::log(&format!("scheduler error: {}", err));
    }

    // convenience: register a simple poll task
    pub fn register_poll_task(&self, id: u64, interval: Duration) {
        let task = ScheduledTask { id, name: "poll_hal".into(), interval, priority: 128, last_run: None };
        self.register_task(task);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::HardwareState;
    use crate::registry::DeviceRegistry;
    use std::sync::{Arc, Mutex, RwLock};

    #[test]
    fn test_scheduler_register_unregister() {
        let state = Arc::new(RwLock::new(HardwareState::default()));
        let reg = Arc::new(Mutex::new(DeviceRegistry::new()));
        let sched = Scheduler::new(Duration::from_millis(10), state, reg, None, None, None);
        sched.start();
        let id = sched.register_task(ScheduledTask { id: 1, name: "t1".into(), interval: Duration::from_millis(20), priority: 1, last_run: None });
        assert_eq!(id, 1);
        assert!(sched.unregister_task(1));
        sched.stop();
    }
}
