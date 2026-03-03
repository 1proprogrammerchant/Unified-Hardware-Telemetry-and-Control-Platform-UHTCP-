use std::thread;
use std::time::Duration;

fn monitor_loop() {
    loop {
        println!("Core engine: polling hardware...");
        thread::sleep(Duration::from_secs(2));
    }
}

fn main() {
    let _ = thread::spawn(|| monitor_loop());
    println!("Core engine started. Press Ctrl+C to exit.");
    loop {
        thread::sleep(Duration::from_secs(60));
    }
}
