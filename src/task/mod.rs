//! # 任务模块

// 引入所需的库和模块
use alloc::sync::Arc;
use common::syscall::OpenFlags;
use lazy_static::lazy_static;
use manager::TaskManager;
use task::ProcessControlBlock;

use crate::{fs::inode::open_file, utils::safety::SyncRefCell};

// 声明子模块
pub mod context;
pub mod manager;
pub mod pid;
pub mod processor;
pub mod stack;
pub mod switch;
pub mod task;

// 使用 lazy_static 宏定义一个静态的 INIT_PROC 变量
lazy_static! {
    // INIT_PROC 是一个 Arc 包装的 SyncRefCell，内部存储了一个 ProcessControlBlock 实例
    pub static ref INIT_PROC: Arc<SyncRefCell<ProcessControlBlock>> = {
        Arc::new(SyncRefCell::new({
            let inode = open_file("initproc", OpenFlags::READONLY).unwrap();
            let data = inode.read_all();
            ProcessControlBlock::new(data.as_slice())
        }))
    };
}

/// 初始化任务管理器函数
pub fn init() {
    // 将 INIT_PROC 添加到任务管理器中
    TaskManager::put_task(INIT_PROC.clone());
}
