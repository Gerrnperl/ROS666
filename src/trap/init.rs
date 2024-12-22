//! 初始化陷入

use core::arch::global_asm;

use riscv::register::stvec::{self, TrapMode};

use super::handler::trap_from_kernel;

global_asm!(include_str!("trap.asm"));

/// 初始化陷入
pub fn init() {
    set_kernel_trap_entry();
}

/// 启用定时器中断
pub fn enable_timer_interrupt() {
    unsafe {
        // 设置定时器中断使能位
        riscv::register::sie::set_stimer();
    }
}

/// 设置内核陷入入口地址
fn set_kernel_trap_entry() {
    unsafe {
        // 写入陷入处理函数地址
        stvec::write(trap_from_kernel as usize, TrapMode::Direct);
    }
}
