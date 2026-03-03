#[repr(C)]
pub struct hal_snapshot_t {
    pub cpu_temperature: f32,
    pub uptime: u64,
    pub gpio_state: u32,
}

extern "C" {
    pub fn hal_init() -> i32;
    pub fn hal_read_snapshot(snapshot: *mut hal_snapshot_t) -> i32;
    pub fn hal_write_gpio(pin: u32, value: u32) -> i32;
    pub fn hal_shutdown();
}
