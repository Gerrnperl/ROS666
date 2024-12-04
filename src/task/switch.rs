use core::arch::global_asm;

use super::context::TaskCtx;

global_asm!(include_str!("switch.asm"));

unsafe extern "C" {
    pub fn __switch(current: *mut TaskCtx, next: *const TaskCtx);
}
