//! # 任务切换

use core::arch::global_asm;

use super::context::TaskCtx;

// 包含汇编代码
global_asm!(include_str!("switch.asm"));

// 声明外部汇编函数 __switch
// current: 指向当前任务上下文的指针
// next: 指向下一个任务上下文的指针
unsafe extern "C" {
    pub fn __switch(current: *mut TaskCtx, next: *const TaskCtx);
}
