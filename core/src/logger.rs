use std::fs::OpenOptions;
use std::io::Write;

pub fn log(msg: &str) {
    let mut f = OpenOptions::new().create(true).append(true).open("../logs/system.log").ok();
    if let Some(ref mut fh) = f {
        let _ = writeln!(fh, "{}", msg);
    }
}
