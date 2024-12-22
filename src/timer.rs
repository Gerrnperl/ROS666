//! 定时器相关
use riscv::register::time;

use crate::sbi::sbi_set_timer;

/// 获取当前时间
pub fn get_time() -> usize {
    time::read()
}

/// 时钟频率
pub const CLOCK_FREQ: usize = 12500000;
/// 每秒的毫秒数
pub const MSEC_PER_SEC: usize = 1000;
/// 每秒的微秒数
pub const USEC_PER_SEC: usize = 1_000_000;

/// 获取当前时间（微秒）
pub fn get_time_us() -> usize {
    get_time() / (CLOCK_FREQ / USEC_PER_SEC)
}

/// 设置下一个时钟中断触发时间
pub fn set_next_timeout(timeout_us: usize) {
    sbi_set_timer((get_time() + timeout_us * (CLOCK_FREQ / USEC_PER_SEC)) as u64);
}
