use crate::{
    app_loader::{AppData, kernel_stack_position},
    mm::{
        KERNEL_SPACE,
        address::{PAGE_SIZE_SV39, PhysicalPageNumber, VirtualAddress, VirtualPageNumber},
        memory_set::{self, MapPermission, MemorySet, TRAMPOLINE},
    },
    trap::{context::TrapCtx, handler::trap_handler},
};

use super::context::TaskCtx;

pub const TRAP_CONTEXT: usize = TRAMPOLINE - PAGE_SIZE_SV39;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TaskStatus {
    Create,
    Ready,
    Running,
    Wait,
    Stopped,
}

pub type TaskId = usize;

pub struct TaskControlBlock {
    pub id: TaskId,
    pub status: TaskStatus,
    pub ctx: TaskCtx,
    pub memory_set: MemorySet,
    pub trap_ctx_ppn: PhysicalPageNumber,
    pub base_size: usize,
}

impl TaskControlBlock {
    pub fn get_trap_cx(&self) -> &'static mut TrapCtx {
        self.trap_ctx_ppn.get_mut()
    }
    pub fn get_user_token(&mut self) -> usize {
        let t = self.memory_set.token();
        t
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
