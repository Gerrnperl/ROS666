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
    }

    pub fn run_next_task() {
        if let Some(next_task_id) = TaskManager::get_next_task() {
            TaskManager::switch_to(next_task_id);
        } else {
            printkln!("All tasks have been run.");
            sbi_shutdown(false);
        }
    }

    pub fn replace_to_next(exit_code: i32) {
    }
}
