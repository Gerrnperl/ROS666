use riscv::register::time;

use crate::sbi::sbi_set_timer;

pub fn get_time() -> usize {
    time::read()
}

pub const CLOCK_FREQ: usize = 12500000;
pub const MSEC_PER_SEC: usize = 1000;
pub const USEC_PER_SEC: usize = 1_000_000;

pub fn get_time_us() -> usize {
    get_time() / (CLOCK_FREQ / USEC_PER_SEC)
}

pub fn set_next_timeout(timeout_us: usize) {
    sbi_set_timer((get_time() + timeout_us * (CLOCK_FREQ / USEC_PER_SEC)) as u64);
}
