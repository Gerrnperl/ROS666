//! 初始化模块

use core::arch::global_asm;

use riscv::register::stvec::{self, TrapMode};

use super::handler::trap_from_kernel;

// 包含汇编代码
global_asm!(include_str!("trap.asm"));

/// 初始化内核陷阱函数
pub fn init() {
    set_kernel_trap_entry(); // 设置内核陷阱入口
}

/// 启用定时器中断
pub fn enable_timer_interrupt() {
    unsafe {
        riscv::register::sie::set_stimer(); // 设置定时器中断使能位
    }
}

/// 设置内核陷阱入口地址
fn set_kernel_trap_entry() {
    unsafe {
        stvec::write(trap_from_kernel as usize, TrapMode::Direct); // 写入陷阱处理函数地址
    }
}
