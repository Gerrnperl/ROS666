use core::borrow::Borrow;

use alloc::{
    sync::{Arc, Weak},
    vec::Vec,
};

use crate::{
    app_loader::AppData,
    mm::{
        KERNEL_SPACE,
        address::{PAGE_SIZE_SV39, PhysicalPageNumber, VirtualAddress, VirtualPageNumber},
        memory_set::{self, MapPermission, MemorySet, TRAMPOLINE},
    },
    trap::{self, context::TrapCtx, handler::trap_handler},
    utils::safety::SyncRefCell,
};

use super::{
    context::TaskCtx,
    pid::{PID_ALLOCATOR, PidAllocator, PidHandler},
    stack::{KernelStack, kernel_stack_position},
};

pub const TRAP_CONTEXT: usize = TRAMPOLINE - PAGE_SIZE_SV39;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProcessStatus {
    Create,
    Ready,
    Running,
    Wait,
    Stopped,
}

pub struct ProcessControlBlock {
    pub pid: PidHandler,
    pub kernel_stack: KernelStack,

    pub status: ProcessStatus,
    pub ctx: TaskCtx,
    pub memory_set: MemorySet,
    pub trap_ctx_ppn: PhysicalPageNumber,
    pub base_size: usize,

    pub parent: Option<Weak<SyncRefCell<ProcessControlBlock>>>,
    pub children: Vec<Arc<SyncRefCell<ProcessControlBlock>>>,

    pub exit_code: i32,
}

impl ProcessControlBlock {
    pub fn get_trap_cx(&self) -> &'static mut TrapCtx {
        self.trap_ctx_ppn.get_mut()
    }
    pub fn get_user_token(&mut self) -> usize {
        let t = self.memory_set.token();
        t
    }
    pub fn get_status(&self) -> ProcessStatus {
        self.status
    }
    pub fn get_pid(&self) -> usize {
        self.pid.0
    }
    pub fn new(app_data: AppData) -> Self {
        let app_id = app_data.app_id;
        let (mut memory_set, user_sp, entry) = MemorySet::from_elf_app(app_data);
        let trap_ctx_ppn = PhysicalPageNumber::from(
            &memory_set
                .translate(VirtualPageNumber::from(VirtualAddress::from(TRAP_CONTEXT)))
                .unwrap(),
        );
        let (kernel_stack_btm, kernel_stack_top) = kernel_stack_position(app_id);
        KERNEL_SPACE.ref_cell.borrow_mut().insert(
            VirtualAddress::from(kernel_stack_btm)..VirtualAddress::from(kernel_stack_top),
            MapPermission::Read | MapPermission::Write,
        );
        let tcb = Self {
            id: app_id,
            status: TaskStatus::Ready,
            ctx: TaskCtx::goto_trap_return(kernel_stack_top),
            memory_set,
            trap_ctx_ppn,
            base_size: user_sp,
        };
        let ctx = tcb.get_trap_cx();
        *ctx = TrapCtx::init_app_context(
            entry,
            user_sp,
            KERNEL_SPACE.ref_cell.borrow_mut().token(),
            kernel_stack_top,
            trap_handler as usize,
        );
        tcb
    }
}
