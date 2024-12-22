//! 任务管理器模块，
//!
//! 基于时间片轮转的任务调度，实现了任务的创建、删除、切换等操作。

use core::cell::RefCell;

use alloc::collections::vec_deque::VecDeque;
use alloc::sync::Arc;
use lazy_static::lazy_static;

use crate::task::context::TaskCtx;
use crate::task::task::{ProcessControlBlock, ProcessStatus};
use crate::utils::safety::SyncRefCell;

use super::INIT_PROC;
use super::processor::Processor;

lazy_static! {
    /// 全局任务管理器
    pub static ref TASK_MANAGER: SyncRefCell<TaskManager> = {
        SyncRefCell {
            ref_cell: RefCell::new(TaskManager::new()),
        }
    };
}

/// 任务管理器
pub struct TaskManager {
    // 就绪队列，用于存放准备执行的任务
    ready_queue: VecDeque<Arc<SyncRefCell<ProcessControlBlock>>>,
}

impl TaskManager {
    /// 创建一个新的任务管理器实例
    pub fn new() -> Self {
        TaskManager {
            ready_queue: VecDeque::new(),
        }
    }

    /// 将任务放入就绪队列
    pub fn put(&mut self, task: Arc<SyncRefCell<ProcessControlBlock>>) {
        self.ready_queue.push_back(task);
    }

    /// 从就绪队列中取出任务
    pub fn get(&mut self) -> Option<Arc<SyncRefCell<ProcessControlBlock>>> {
        self.ready_queue.pop_front()
    }

    /// 将任务放入全局任务管理器的就绪队列
    pub fn put_task(task: Arc<SyncRefCell<ProcessControlBlock>>) {
        let mut this = TASK_MANAGER.ref_cell.borrow_mut();
        this.put(task);
    }

    /// 从全局任务管理器的就绪队列中取出任务
    pub fn get_task() -> Option<Arc<SyncRefCell<ProcessControlBlock>>> {
        let mut this = TASK_MANAGER.ref_cell.borrow_mut();
        this.get()
    }

    /// 将当前任务切换到下一个任务
    pub fn cycle_to_next() {
        let task = Processor::take_current().unwrap();

        let mut pcb = task.inner_borrow_mut();
        let ctx = &mut pcb.ctx as *mut TaskCtx;
        pcb.status = ProcessStatus::Ready;
        drop(pcb);

        TaskManager::put_task(task);
        Processor::schedule(ctx);
    }

    /// 将当前任务替换为下一个任务，并设置退出码
    pub fn replace_to_next(exit_code: i32) {
        let task = Processor::take_current().unwrap();
        let mut pcb = task.inner_borrow_mut();
        pcb.exit_code = exit_code;
        pcb.status = ProcessStatus::Stopped;
        let mut initproc = INIT_PROC.inner_borrow_mut();
        for child in pcb.children.iter() {
            child.inner_borrow_mut().parent = Some(Arc::downgrade(&INIT_PROC));
            initproc.children.push(child.clone());
        }
        drop(initproc);
        pcb.children.clear();
        pcb.memory_set.recycle();
        drop(pcb);
        drop(task);
        let mut _unused = TaskCtx::default();
        Processor::schedule(&mut _unused as *mut _);
        // let mut this = TASK_MANAGER.ref_cell.borrow_mut();
        // let current = this.current;
        // this.tasks[current].status = crate::task::task::ProcessStatus::Stopped;
        // drop(this);
        // TaskManager::run_next_task();
    }
}
