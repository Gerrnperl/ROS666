//! 任务上下文

use crate::trap::handler::trap_return;

/// 任务上下文
///
/// 保存任务的寄存器信息
/// > 不用保存其它寄存器是因为：其它寄存器中，属于调用者保存的寄存器是
/// > 由编译器在高级语言编写的调用函数中自动生成的代码来完成保存的；
/// > 还有一些寄存器属于临时寄存器，不需要保存和恢复。
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct TaskCtx {
    ra: usize,
    sp: usize,
    s: [usize; 12],
}

impl Default for TaskCtx {
    /// 默认任务上下文
    ///
    /// 初始化任务上下文的寄存器信息为0
    fn default() -> Self {
        Self {
            ra: 0,
            sp: 0,
            s: [0; 12],
        }
    }
}

impl TaskCtx {
    /// 创建一个新的 `TaskCtx` 实例，并将其初始化为陷阱返回状态。
    ///
    /// ## 参数
    /// * `kernel_stack_ptr` - 内核栈指针的地址。
    /// ## 返回值
    /// 返回一个新的 `TaskCtx` 实例，其中：
    /// * `ra` 被设置为 `trap_return` 函数的地址。
    /// * `sp` 被设置为传入的 `kernel_stack_ptr`。
    /// * `s` 寄存器数组被初始化为 0。
    /// ## 示例
    /// ```no_run
    /// let task_ctx = TaskCtx::goto_trap_return(kernel_stack_ptr);
    /// ```
    pub fn goto_trap_return(kernel_stack_ptr: usize) -> Self {
        Self {
            ra: trap_return as usize,
            sp: kernel_stack_ptr,
            s: [0; 12],
        }
    }
}
