//! 进程管理模块

use core::borrow::Borrow;

use alloc::{
    sync::{Arc, Weak},
    vec::Vec,
};

/// 导入所需的模块和类型:
///
/// - `crate::app_loader::AppData`: 应用加载器相关的数据类型。
/// - `crate::mm::{KERNEL_SPACE, address::{PAGE_SIZE_SV39, PhysicalPageNumber, VirtualAddress, VirtualPageNumber}, memory_set::{self, MapPermission, MemorySet, TRAMPOLINE}}`: 内存管理相关的模块和类型。
/// - `crate::trap::{self, context::TrapCtx, handler::trap_handler}`: 中断和陷阱处理相关的模块和类型。
/// - `crate::utils::safety::SyncRefCell`: 一个线程安全的 RefCell 类型。
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

/// 定义陷阱上下文的地址常量
pub const TRAP_CONTEXT: usize = TRAMPOLINE - PAGE_SIZE_SV39;

/// 进程状态枚举
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

/// 进程控制块结构体
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
}

/// 进程控制块（Process Control Block）实现
impl ProcessControlBlock {
    /// 获取陷阱上下文的可变引用
    ///
    /// ## 返回值
    /// 返回陷阱上下文的可变引用
    pub fn get_trap_cx(&self) -> &'static mut TrapCtx {
        self.trap_ctx_ppn.get_mut()
    }

    /// 获取用户态的令牌
    ///
    /// ## 返回值
    /// 返回用户态的令牌
    pub fn get_user_token(&mut self) -> usize {
        let t = self.memory_set.token();
        t
    }

    /// 获取进程状态
    ///
    /// ## 返回值
    /// 返回进程状态
    pub fn get_status(&self) -> ProcessStatus {
        self.status
    }

    /// 获取进程ID
    ///
    /// ## 返回值
    /// 返回进程ID
    pub fn get_pid(&self) -> usize {
        self.pid.0
    }

    /// 创建新的进程控制块
    ///
    /// ## 参数
    /// - `app_data`: 应用程序数据
    /// ## 返回值
    /// 返回一个新的进程控制块实例
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
        }));
        parent.children.push(child.clone());

        let ctx = child.inner_borrow().get_trap_cx();
        ctx.kernel_virt_sp = kernel_stack_top;

        child
    }

    /// 执行新程序
    ///
    /// ## 参数
    /// - `app_data`: 应用程序数据
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
