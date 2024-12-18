use core::borrow::Borrow;
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
    pub static ref PROCESSOR: SyncRefCell<Processor> = {
        SyncRefCell {
            ref_cell: RefCell::new(Processor::new()),
        }
    };
}

pub struct Processor {
    current: Option<Arc<SyncRefCell<ProcessControlBlock>>>,
    idle_ctx: TaskCtx,
}

impl Processor {
    pub fn new() -> Self {
        Self {
            current: None,
            idle_ctx: TaskCtx::default(),
        }
    }

}
