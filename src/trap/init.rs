use core::arch::global_asm;

use riscv::register::stvec::{self, TrapMode};

use crate::extern_global;

use super::handler::trap_from_kernel;

global_asm!(include_str!("trap.asm"));

pub fn init() {
    set_kernel_trap_entry();
}

pub fn enable_timer_interrupt() {
    unsafe {
        riscv::register::sie::set_stimer();
    }
}

fn set_kernel_trap_entry() {
    unsafe {
        stvec::write(trap_from_kernel as usize, TrapMode::Direct);
    }
}
