//! # 任务切换

use core::arch::global_asm;

use super::context::TaskCtx;

global_asm!(include_str!("switch.asm"));

unsafe extern "C" {
    /// 切换任务
    ///
    /// ## 参数
    /// - `current`: 当前任务的上下文
    /// - `next`: 下一个任务的上下文
    pub fn __switch(current: *mut TaskCtx, next: *const TaskCtx);
}
