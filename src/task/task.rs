//! 进程管理

use alloc::vec;
use alloc::{
    sync::{Arc, Weak},
    vec::Vec,
};

use crate::fs::stdio::{Stdin, Stdout};

use crate::{
    fs::File,
    mm::{
        KERNEL_SPACE,
        address::{PAGE_SIZE_SV39, PhysicalPageNumber, VirtualAddress, VirtualPageNumber},
        memory_set::{MemorySet, TRAMPOLINE},
    },
    trap::{context::TrapCtx, handler::trap_handler},
    utils::safety::SyncRefCell,
};

use super::{
    context::TaskCtx,
    pid::{PidAllocator, PidHandler},
    stack::KernelStack,
};

/// 定义陷入上下文的地址常量
pub const TRAP_CONTEXT: usize = TRAMPOLINE - PAGE_SIZE_SV39;

/// 进程状态
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProcessStatus {
    /// 创建状态
    Create,
    /// 就绪状态
    Ready,
    /// 运行状态
    Running,
    /// 等待状态
    Wait,
    /// 停止状态
    Stopped,
}

/// 进程
pub type FdTable = Vec<Option<Arc<dyn File + Send + Sync>>>;

pub trait FdTableClone {
    /// 克隆文件描述符表
    ///
    /// 从当前文件描述符表中克隆一个新的文件描述符表，增加文件描述符的引用计数
    fn fd_clone(&self) -> Self;
}

impl FdTableClone for FdTable {
    fn fd_clone(&self) -> Self {
        let mut new_table = Vec::new();
        for file in self.iter() {
            new_table.push(if let Some(file) = file {
                Some(Arc::clone(file))
            } else {
                None
            });
        }
        new_table
    }
}

/// 进程控制块
pub struct ProcessControlBlock {
    /// 进程ID
    pub pid: PidHandler,
    /// 内核栈
    pub kernel_stack: KernelStack,
    /// 进程状态
    pub status: ProcessStatus,
    /// 任务上下文
    pub ctx: TaskCtx,
    /// 内存集合
    pub memory_set: MemorySet,
    /// 陷阱上下文的物理页号
    pub trap_ctx_ppn: PhysicalPageNumber,
    /// 基础大小
    pub base_size: usize,
    /// 父进程
    pub parent: Option<Weak<SyncRefCell<ProcessControlBlock>>>,
    /// 子进程
    pub children: Vec<Arc<SyncRefCell<ProcessControlBlock>>>,
    /// 退出码
    pub exit_code: i32,
    /// 文件描述符表
    pub fd_table: FdTable,
}

/// 进程控制块（Process Control Block）实现
impl ProcessControlBlock {
    /// 获取陷入上下文的可变引用
    ///
    /// ## 返回
    /// 返回陷入上下文的可变引用
    pub fn get_trap_cx(&self) -> &'static mut TrapCtx {
        self.trap_ctx_ppn.get_mut()
    }

    /// 获取用户地址空间的 satp 寄存器值
    ///
    /// 即用户地址空间三级页表根节点的地址（并且其第 60-63 位为模式位 8）
    pub fn get_user_token(&mut self) -> usize {
        let t = self.memory_set.token();
        t
    }

    /// 获取进程状态
    pub fn get_status(&self) -> ProcessStatus {
        self.status
    }

    /// 获取进程ID
    pub fn get_pid(&self) -> usize {
        self.pid.0
    }

    /// 创建新的进程控制块
    ///
    /// ## 参数
    /// - `app_data`: 应用程序数据
    /// ## 返回值
    /// 返回一个新的进程控制块实例
    pub fn new(app_data: &[u8]) -> Self {
        let (memory_set, user_sp, entry) = MemorySet::from_elf_app(app_data);
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
            fd_table: vec![
                Some(Arc::new(Stdin)),
                Some(Arc::new(Stdout)),
                Some(Arc::new(Stdout)),
            ],
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

    /// 为进程分配文件描述符
    pub fn alloc_fd(&mut self) -> usize {
        for (fd, file) in self.fd_table.iter().enumerate() {
            if file.is_none() {
                return fd;
            }
        }
        self.fd_table.push(None);
        self.fd_table.len() - 1
    }
}

impl SyncRefCell<ProcessControlBlock> {
    /// 创建子进程
    ///
    /// ## 返回值
    /// 返回子进程的引用
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
            fd_table: parent.fd_table.fd_clone(),
        }));
        parent.children.push(child.clone());

        let ctx = child.inner_borrow().get_trap_cx();
        ctx.kernel_virt_sp = kernel_stack_top;

        child
    }

    /// 在当前进程上执行应用程序
    ///
    /// ## 参数
    /// - `app_data`: 应用程序数据
    pub fn exec(self: &Arc<SyncRefCell<ProcessControlBlock>>, app_data: &[u8]) {
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
