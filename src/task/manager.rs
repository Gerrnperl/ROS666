use core::arch::asm;
use core::cell::RefCell;

use alloc::vec::Vec;
use lazy_static::lazy_static;

use crate::app_loader::{get_app_count, load_app_data};
use crate::task::context::TaskCtx;
use crate::task::switch::__switch;
use crate::task::task::{TaskControlBlock, TaskStatus};
use crate::trap::context::TrapCtx;
use crate::utils::safety::SyncRefCell;
use crate::{info, printkln, sbi::sbi_shutdown};

lazy_static! {
    pub static ref TASK_MANAGER: SyncRefCell<TaskManager> = {
        SyncRefCell {
            ref_cell: RefCell::new({
                let app_count = get_app_count();
                let mut tasks = Vec::with_capacity(app_count);
                for i in 0..app_count {
                    tasks.push(TaskControlBlock::new(load_app_data(i)));
                }
                TaskManager {
                    app_count,
                    current: 0,
                    tasks,
                }
            }),
        }
    };
}

pub struct TaskManager {
    /// 应用程序总数
    pub app_count: usize,
    /// 当前运行的应用程序编号
    pub current: usize,
    /// 任务控制块数组
    pub tasks: Vec<TaskControlBlock>,
}

impl TaskManager {
    pub fn start() {
        info!("Start at task {}", 0);
        // APP_LOADER.ref_cell.borrow().print_app_info(0);
        let mut this = TASK_MANAGER.ref_cell.borrow_mut();
        this.tasks[0].status = crate::task::task::TaskStatus::Running;
        let next_ptr = &this.tasks[0].ctx as *const TaskCtx;
        drop(this);
        let mut null = TaskCtx::default();
        unsafe {
            __switch(&mut null, next_ptr);
        }
    }

    pub fn switch_to(task_id: usize) {
        // info!("Switch to task {}", task_id);
        // APP_LOADER.ref_cell.borrow().print_app_info(task_id);
        let mut this = TASK_MANAGER.ref_cell.borrow_mut();
        let current = this.current;
        this.tasks[task_id].status = crate::task::task::TaskStatus::Running;
        let current_ptr = &mut this.tasks[current].ctx as *mut TaskCtx;
        let next_ptr = &this.tasks[task_id].ctx as *const TaskCtx;
        this.current = task_id;
        drop(this);
        unsafe {
            __switch(current_ptr, next_ptr);
        }
    }

    pub fn cycle_to_next() {
        let mut this = TASK_MANAGER.ref_cell.borrow_mut();
        let current = this.current;
        this.tasks[current].status = crate::task::task::TaskStatus::Ready;
        drop(this);
        TaskManager::run_next_task();
    }

    pub fn replace_to_next() {
        let mut this = TASK_MANAGER.ref_cell.borrow_mut();
        let current = this.current;
        this.tasks[current].status = crate::task::task::TaskStatus::Stopped;
        drop(this);
        TaskManager::run_next_task();
    }

    pub fn run_next_task() {
        if let Some(next_task_id) = TaskManager::get_next_task() {
            TaskManager::switch_to(next_task_id);
        } else {
            printkln!("All tasks have been run.");
            sbi_shutdown(false);
        }
    }

    pub fn get_next_task() -> Option<usize> {
        let this = TASK_MANAGER.ref_cell.borrow();
        let current = this.current;
        // find a task that is ready to run
        for i in 1..=this.app_count {
            let task_id = (current + i) % this.app_count;
            if this.tasks[task_id].status == crate::task::task::TaskStatus::Ready {
                drop(this);
                return Some(task_id);
            }
        }
        drop(this);
        None
    }

    pub fn current_user_token() -> usize {
        let mut tm = TASK_MANAGER.ref_cell.borrow_mut();
        let current = tm.current;
        tm.tasks[current].get_user_token()
    }

    pub fn current_trap_cx() -> &'static mut TrapCtx {
        let tm = TASK_MANAGER.ref_cell.borrow();
        let current = tm.current;
        tm.tasks[current].get_trap_cx()
    }
}
