//! # 初始化模块

use crate::trace;

use super::{KERNEL_SPACE, frame_allocator, heap_allocator};

// 全局静态变量，表示堆是否已初始化
pub static mut HEAP_INITED: bool = false;

// 初始化函数
pub fn init() {
    // 初始化堆分配器
    trace!("[Kernel] Init heap allocator");
    heap_allocator::init_heap();
    // 初始化帧分配器
    trace!("[Kernel] Init frame allocator");
    frame_allocator::StackFrameAllocator::init_frame_allocator();
    // 激活内核空间
    trace!("[Kernel] Activate kernel space");
    KERNEL_SPACE.ref_cell.borrow_mut().activate();
    // 设置堆已初始化标志
    unsafe { HEAP_INITED = true };
}
