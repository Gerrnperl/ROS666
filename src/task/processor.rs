//! 处理器
//!
//! 负责任务的调度和切换，从任务管理器中获取任务并运行。

use core::cell::RefCell;

use alloc::sync::Arc;
use lazy_static::lazy_static;

use crate::task::context::TaskCtx;
use crate::task::task::ProcessControlBlock;
use crate::trap::context::TrapCtx;
use crate::utils::safety::SyncRefCell;

use super::manager::TaskManager;
use super::switch::__switch;
use super::task::ProcessStatus;

lazy_static! {
    /// 全局处理器
    pub static ref PROCESSOR: SyncRefCell<Processor> = {
        SyncRefCell {
            ref_cell: RefCell::new(Processor::new()),
        }
    };
}

/// 处理器
pub struct Processor {
    /// 当前正在运行的任务
    current: Option<Arc<SyncRefCell<ProcessControlBlock>>>,
    /// 空闲任务上下文
    idle_ctx: TaskCtx,
}

impl Processor {
    /// 创建一个新的处理器实例
    pub fn new() -> Self {
        Self {
            current: None,
            idle_ctx: TaskCtx::default(),
        }
    }

    /// 获取空闲任务上下文的指针
    fn get_idle_ctx_ptr(&mut self) -> *mut TaskCtx {
        &mut self.idle_ctx as *mut _
    }

    /// 运行任务
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

    /// 调度任务
    pub fn schedule(switched_task_cx_ptr: *mut TaskCtx) {
        let mut this = PROCESSOR.inner_borrow_mut();
        let idle_ctx = this.get_idle_ctx_ptr();
        drop(this);
        unsafe {
            __switch(switched_task_cx_ptr, idle_ctx);
        }
    }

    /// 获取当前任务并将其从处理器中移除
    pub fn take_current() -> Option<Arc<SyncRefCell<ProcessControlBlock>>> {
        PROCESSOR.ref_cell.borrow_mut().current.take()
    }

    /// 获取当前任务的引用
    pub fn get_current() -> Option<Arc<SyncRefCell<ProcessControlBlock>>> {
        PROCESSOR
            .ref_cell
            .borrow()
            .current
            .as_ref()
            .map(|pcb| Arc::clone(pcb))
    }

    /// 获取当前用户的 satp 寄存器的值
    pub fn current_user_token() -> Option<usize> {
        let current = Self::get_current()?;
        let token = current.ref_cell.borrow_mut().get_user_token();
        Some(token)
    }

    /// 获取当前陷入上下文
    pub fn current_trap_cx() -> Option<&'static mut TrapCtx> {
        let current = Self::get_current()?;
        let trap_cx = current.ref_cell.borrow_mut().get_trap_cx();
        Some(trap_cx)
    }
}
