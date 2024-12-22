//! 内核堆初始化

use crate::trace;

use super::{KERNEL_SPACE, frame_allocator, heap_allocator};

/// 记录内核堆是否已初始化
///
/// 内核堆初始化后，内核地址空间将被激活。
///
/// 一些硬件相关的功能，如 sbi io 需要根据不同的地址空间执行不同的操作。
pub static mut HEAP_INITED: bool = false;

// 初始化内核内存管理
pub fn init() {
    trace!("[Kernel] Init heap allocator");
    heap_allocator::init_heap();
    trace!("[Kernel] Init frame allocator");
    frame_allocator::StackFrameAllocator::init_frame_allocator();
    trace!("[Kernel] Activate kernel space");
    KERNEL_SPACE.ref_cell.borrow_mut().activate();
    unsafe { HEAP_INITED = true };
}
