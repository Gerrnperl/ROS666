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
        let pid = PidAllocator::alloc_pid();
        let kernel_stack = KernelStack::new(&pid);
        let kernel_stack_top = kernel_stack.top();

        let pcb = Self {
            pid,
            kernel_stack,
            status: ProcessStatus::Ready,
            ctx: TaskCtx::goto_trap_return(kernel_stack_top),
            memory_set,
            trap_ctx_ppn,
            base_size: user_sp,
            parent: None,
            children: Vec::new(),
            exit_code: 0,
        };
        let ctx = pcb.get_trap_cx();
        *ctx = TrapCtx::init_app_context(
            entry,
            user_sp,
            KERNEL_SPACE.ref_cell.borrow_mut().token(),
            kernel_stack_top,
            trap_handler as usize,
        );
        pcb
    }
}

impl SyncRefCell<ProcessControlBlock> {
    pub fn fork(
        self: &Arc<SyncRefCell<ProcessControlBlock>>,
    ) -> Arc<SyncRefCell<ProcessControlBlock>> {
        let mut parent = self.inner_borrow_mut();
        let memory_set = MemorySet::from_existed_user(&parent.memory_set);
        let trap_ctx_ppn = PhysicalPageNumber::from(
            &memory_set
                .translate(VirtualPageNumber::from(VirtualAddress::from(TRAP_CONTEXT)))
                .unwrap(),
        );
        let pid = PidAllocator::alloc_pid();
        let kernel_stack = KernelStack::new(&pid);
        let kernel_stack_top = kernel_stack.top();
        let child = Arc::new(SyncRefCell::new(ProcessControlBlock {
            pid,
            kernel_stack,
            status: ProcessStatus::Ready,
            ctx: TaskCtx::goto_trap_return(kernel_stack_top),
            memory_set,
            trap_ctx_ppn,
            base_size: parent.base_size,
            parent: Some(Arc::downgrade(self)),
            children: Vec::new(),
            exit_code: 0,
        }));
        parent.children.push(child.clone());

        let ctx = child.inner_borrow().get_trap_cx();
        ctx.kernel_virt_sp = kernel_stack_top;

        child
    }

    pub fn exec(self: &Arc<SyncRefCell<ProcessControlBlock>>, app_data: AppData) {
        let (memory_set, user_sp, entry) = MemorySet::from_elf_app(app_data);
        let trap_ctx_ppn = PhysicalPageNumber::from(
            &memory_set
                .translate(VirtualPageNumber::from(VirtualAddress::from(TRAP_CONTEXT)))
                .unwrap(),
        );
        let mut pcb = self.inner_borrow_mut();
        pcb.memory_set = memory_set;
        pcb.trap_ctx_ppn = trap_ctx_ppn;
        let ctx = pcb.get_trap_cx();
        *ctx = TrapCtx::init_app_context(
            entry,
            user_sp,
            KERNEL_SPACE.ref_cell.borrow_mut().token(),
            pcb.kernel_stack.top(),
            trap_handler as usize,
        );
    }
}
