//! 任务模块
//!
//! 任务模块实现了进程控制块、任务管理调度、任务切换等功能。

use alloc::sync::Arc;
use common::syscall::OpenFlags;
use lazy_static::lazy_static;
use manager::TaskManager;
use task::ProcessControlBlock;

use crate::{fs::inode::open_file, utils::safety::SyncRefCell};

pub mod context;
pub mod manager;
pub mod pid;
pub mod processor;
pub mod stack;
pub mod switch;
pub mod task;

lazy_static! {
    /// INIT_PROC
    ///
    /// ininproc 是系统的第一个进程，它的代码和数据来自 initproc 文件。
    ///
    /// 其主要功能是启动其他用户进程（sh)
    pub static ref INIT_PROC: Arc<SyncRefCell<ProcessControlBlock>> = {
        Arc::new(SyncRefCell::new({
            let inode = open_file("initproc", OpenFlags::READONLY).unwrap();
            let data = inode.read_all();
            ProcessControlBlock::new(data.as_slice())
        }))
    };
}

/// 初始化任务管理器
///
/// 初始化任务管理器，并将 INIT_PROC 放入任务管理器的就绪队列中
pub fn init() {
    TaskManager::put_task(INIT_PROC.clone());
}
