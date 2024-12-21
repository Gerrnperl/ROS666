//! 处理器模块

// 引入核心库中的 Borrow trait 和 RefCell 结构体
use core::borrow::Borrow;
use core::cell::RefCell;

// 引入分配库中的 Arc（原子引用计数）
use alloc::sync::Arc;
// 引入 lazy_static 宏，用于定义全局静态变量
use lazy_static::lazy_static;

// 引入项目中的模块和结构体
use crate::task::context::TaskCtx; // 任务上下文
use crate::task::task::ProcessControlBlock; // 进程控制块
use crate::trap::context::TrapCtx; // 陷阱上下文
use crate::utils::safety::SyncRefCell; // 同步引用计数的 RefCell

// 引入同一模块中的其他部分
use super::manager::TaskManager; // 任务管理器
use super::switch::__switch; // 任务切换函数
use super::task::ProcessStatus; // 进程状态

lazy_static! {
    // 定义一个全局的处理器实例
    pub static ref PROCESSOR: SyncRefCell<Processor> = {
        SyncRefCell {
            ref_cell: RefCell::new(Processor::new()),
        }
    };
}

// 处理器结构体
pub struct Processor {
    current: Option<Arc<SyncRefCell<ProcessControlBlock>>>, // 当前正在运行的任务
    idle_ctx: TaskCtx,                                      // 空闲任务上下文
}

impl Processor {
    // 创建一个新的处理器实例
    pub fn new() -> Self {
        Self {
            current: None,
            idle_ctx: TaskCtx::default(),
        }
    }

    // 获取空闲任务上下文的指针
    fn get_idle_ctx_ptr(&mut self) -> *mut TaskCtx {
        &mut self.idle_ctx as *mut _
    }

    // 运行任务
    pub fn run_tasks() {
        loop {
            let mut this = PROCESSOR.inner_borrow_mut();
            if let Some(task) = TaskManager::get_task() {
                let idle_ctx = this.get_idle_ctx_ptr();
                let mut pcb = task.inner_borrow_mut();
                let next_ctx = &pcb.ctx as *const TaskCtx;
                pcb.status = ProcessStatus::Running;
                drop(pcb);
                this.current = Some(task);
                drop(this);
                unsafe {
                    __switch(idle_ctx, next_ctx);
                }
            }
        }
    }

    // 调度任务
    pub fn schedule(switched_task_cx_ptr: *mut TaskCtx) {
        let mut this = PROCESSOR.inner_borrow_mut();
        let idle_ctx = this.get_idle_ctx_ptr();
        drop(this);
        unsafe {
            __switch(switched_task_cx_ptr, idle_ctx);
        }
    }

    // 获取当前任务并将其从处理器中移除
    pub fn take_current() -> Option<Arc<SyncRefCell<ProcessControlBlock>>> {
        PROCESSOR.ref_cell.borrow_mut().current.take()
    }

    // 获取当前任务的引用
    pub fn get_current() -> Option<Arc<SyncRefCell<ProcessControlBlock>>> {
        PROCESSOR
            .ref_cell
            .borrow()
            .current
            .as_ref()
            .map(|pcb| Arc::clone(pcb))
    }

    // 获取当前用户的令牌
    pub fn current_user_token() -> Option<usize> {
        let current = Self::get_current()?;
        let token = current.ref_cell.borrow_mut().get_user_token();
        Some(token)
    }

    // 获取当前陷阱上下文
    pub fn current_trap_cx() -> Option<&'static mut TrapCtx> {
        let current = Self::get_current()?;
        let trap_cx = current.ref_cell.borrow_mut().get_trap_cx();
        Some(trap_cx)
    }
}
