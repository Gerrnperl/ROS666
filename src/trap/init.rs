use core::arch::global_asm;

use riscv::register::stvec;

use crate::extern_global;

global_asm!(include_str!("trap.asm"));

pub fn init() {
    unsafe {
        stvec::write(
            extern_global!(__save_trap) as usize,
            stvec::TrapMode::Direct,
        )
    };
}
