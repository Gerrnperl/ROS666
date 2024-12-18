use core::{cell::RefCell, ops::Range};

use alloc::vec::Vec;
use lazy_static::lazy_static;

use crate::utils::safety::{self, SyncRefCell};

lazy_static! {
    /// 全局 PID 分配器
    pub static ref PID_ALLOCATOR: SyncRefCell<PidAllocator> = SyncRefCell {
        ref_cell: RefCell::new(PidAllocator::new())
    };
}

pub struct PidHandler(pub usize);

pub struct PidAllocator {
    unused: Range<usize>,
    recycled: Vec<usize>,
}

impl PidAllocator {
    fn new() -> Self {
        Self {
            unused: 0..(usize::MAX - 256), // reserve 256
            recycled: Vec::new(),
        }
    }

    fn alloc(&mut self) -> Option<PidHandler> {
        if let Some(pid) = self.recycled.pop() {
            return Some(PidHandler(pid));
        }
        if self.unused.start < self.unused.end {
            let pid = self.unused.start;
            self.unused.start += 1;
            return Some(PidHandler(pid));
        }
        None
    }

    fn dealloc(&mut self, pid: usize) {
        if pid >= self.unused.start {
            panic!("trying to dealloc dealloced pid: {:?}", pid);
        }
        if self.recycled.iter().any(|&f| f == pid) {
            panic!("trying to dealloc dealloced pid: {:?}", pid);
        }
        self.recycled.push(pid);
    }

    pub fn alloc_pid() -> PidHandler {
        PID_ALLOCATOR.ref_cell.borrow_mut().alloc().unwrap()
    }
}

impl Drop for PidHandler {
    fn drop(&mut self) {
        PID_ALLOCATOR.ref_cell.borrow_mut().dealloc(self.0);
    }
}
