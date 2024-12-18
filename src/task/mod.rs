use alloc::sync::Arc;
use lazy_static::lazy_static;
use manager::TaskManager;
use task::ProcessControlBlock;

use crate::{app_loader::load_app_data_by_name, utils::safety::SyncRefCell};

pub mod context;
pub mod manager;
pub mod pid;
pub mod processor;
pub mod stack;
pub mod switch;
pub mod task;

lazy_static! {
    pub static ref INIT_PROC: Arc<SyncRefCell<ProcessControlBlock>> = {
        Arc::new(SyncRefCell::new(ProcessControlBlock::new(
            load_app_data_by_name("initproc").unwrap(),
        )))
    };
}

pub fn init() {
    TaskManager::put_task(INIT_PROC.clone());
}
