use riscv::register::time;

pub fn get_time() -> usize {
    time::read()
}

pub const CLOCK_FREQ: usize = 12500000;

pub fn get_time_us() -> usize {
    get_time() / (CLOCK_FREQ / 1_000_000)
}
