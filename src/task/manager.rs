use core::arch::asm;
use core::cell::{Ref, RefCell};

use alloc::collections::vec_deque::VecDeque;
use alloc::sync::Arc;
use alloc::vec::Vec;
use lazy_static::lazy_static;

use crate::app_loader::{get_app_count, load_app_data};
use crate::task::context::TaskCtx;
use crate::task::switch::__switch;
use crate::task::task::{ProcessControlBlock, ProcessStatus};
use crate::trap::context::TrapCtx;
use crate::utils::safety::SyncRefCell;
use crate::{info, printkln, sbi::sbi_shutdown};

use super::INIT_PROC;
use super::processor::Processor;

lazy_static! {
    pub static ref TASK_MANAGER: SyncRefCell<TaskManager> = {
        SyncRefCell {
            ref_cell: RefCell::new(TaskManager::new()),
        }
    };
}

pub struct TaskManager {
    ready_queue: VecDeque<Arc<SyncRefCell<ProcessControlBlock>>>,
}

impl TaskManager {
    pub fn new() -> Self {
        TaskManager {
            ready_queue: VecDeque::new(),
        }
    }

    pub fn put(&mut self, task: Arc<SyncRefCell<ProcessControlBlock>>) {
        self.ready_queue.push_back(task);
    }

    pub fn get(&mut self) -> Option<Arc<SyncRefCell<ProcessControlBlock>>> {
        self.ready_queue.pop_front()
    }

    pub fn put_task(task: Arc<SyncRefCell<ProcessControlBlock>>) {
        let mut this = TASK_MANAGER.ref_cell.borrow_mut();
        this.put(task);
    }

    pub fn get_task() -> Option<Arc<SyncRefCell<ProcessControlBlock>>> {
        let mut this = TASK_MANAGER.ref_cell.borrow_mut();
        this.get()
    }

    pub fn cycle_to_next() {
        let task = Processor::take_current().unwrap();

        let mut pcb = task.inner_borrow_mut();
        let ctx = &mut pcb.ctx as *mut TaskCtx;
        pcb.status = ProcessStatus::Ready;
        drop(pcb);

        TaskManager::put_task(task);
        Processor::schedule(ctx);
    }

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
