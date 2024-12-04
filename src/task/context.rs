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
    fn default() -> Self {
        Self {
            ra: 0,
            sp: 0,
            s: [0; 12],
        }
    }
}

impl TaskCtx {
    pub fn restore_to_kernel(kernel_stack_ptr: usize) -> Self {
        unsafe extern "C" {
            fn __restore_trap();
        }
        Self {
            ra: __restore_trap as usize,
            sp: kernel_stack_ptr,
            s: [0; 12],
        }
    }
}
