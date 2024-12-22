//! 进程 ID（PID）分配器

use core::{cell::RefCell, ops::Range};

use alloc::vec::Vec;
use lazy_static::lazy_static;

use crate::utils::safety::SyncRefCell;

lazy_static! {
    /// 全局 PID 分配器
    pub static ref PID_ALLOCATOR: SyncRefCell<PidAllocator> = SyncRefCell {
        ref_cell: RefCell::new(PidAllocator::new())
    };
}

/// PID 处理器结构体
pub struct PidHandler(pub usize);

/// PID 分配器结构体
pub struct PidAllocator {
    unused: Range<usize>, // 未使用的 PID 范围
    recycled: Vec<usize>, // 回收的 PID 列表
}

impl PidAllocator {
    /// 创建一个新的 PID 分配器
    fn new() -> Self {
        Self {
            unused: 0..(usize::MAX - 256), // 保留 256 个 PID
            recycled: Vec::new(),
        }
    }

    /// 分配一个新的 PID
    fn alloc(&mut self) -> Option<PidHandler> {
        // 优先使用回收的 PID
        if let Some(pid) = self.recycled.pop() {
            return Some(PidHandler(pid));
        }
        // 如果没有回收的 PID，则使用未使用的 PID
        if self.unused.start < self.unused.end {
            let pid = self.unused.start;
            self.unused.start += 1;
            return Some(PidHandler(pid));
        }
        // 如果没有可用的 PID，则返回 None
        None
    }

    /// 释放一个 PID
    fn dealloc(&mut self, pid: usize) {
        // 检查 PID 是否在未使用的范围内
        if pid >= self.unused.start {
            panic!("尝试释放未分配的 PID: {:?}", pid);
        }
        // 检查 PID 是否已经在回收列表中
        if self.recycled.iter().any(|&f| f == pid) {
            panic!("尝试释放已释放的 PID: {:?}", pid);
        }
        // 将 PID 添加到回收列表中
        self.recycled.push(pid);
    }

    /// 分配一个新的 PID 并返回处理器
    pub fn alloc_pid() -> PidHandler {
        PID_ALLOCATOR.ref_cell.borrow_mut().alloc().unwrap()
    }
}

impl Drop for PidHandler {
    /// 当 PID 处理器被丢弃时，自动释放 PID
    fn drop(&mut self) {
        PID_ALLOCATOR.ref_cell.borrow_mut().dealloc(self.0);
    }
}
