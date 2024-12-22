//! 任务管理器模块，用于管理任务的调度和切换

// 引入核心库中的汇编模块
use core::arch::asm;
// 引入核心库中的Ref和RefCell模块，用于实现内部可变性
use core::cell::{Ref, RefCell};

// 引入alloc库中的VecDeque，用于实现任务队列
use alloc::collections::vec_deque::VecDeque;
// 引入alloc库中的Arc，用于实现引用计数的智能指针
use alloc::sync::Arc;
// 引入alloc库中的Vec，用于动态数组
use alloc::vec::Vec;
// 引入lazy_static库，用于定义静态变量
use lazy_static::lazy_static;

// 引入自定义模块中的函数和结构体
use crate::app_loader::{get_app_count, load_app_data};
use crate::task::context::TaskCtx;
use crate::task::switch::__switch;
use crate::task::task::{ProcessControlBlock, ProcessStatus};
use crate::trap::context::TrapCtx;
use crate::utils::safety::SyncRefCell;
use crate::{info, printkln, sbi::sbi_shutdown};

// 引入当前模块中的INIT_PROC和Processor
use super::INIT_PROC;
use super::processor::Processor;

// 使用lazy_static宏定义一个静态变量TASK_MANAGER，用于管理任务
lazy_static! {
    pub static ref TASK_MANAGER: SyncRefCell<TaskManager> = {
        SyncRefCell {
            ref_cell: RefCell::new(TaskManager::new()),
        }
    };
}

/// 任务管理器结构体
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
