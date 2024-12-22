//! 这个模块包含堆分配器的初始化和错误处理。
//!
//! 主要功能包括：
//! - 初始化堆分配器。
//! - 处理堆分配错误。

use slab_allocator::LockedHeap;

pub const USER_HEAP_SIZE: usize = 0x4000;

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

// NOTE: 如果不使用 `static mut`，`HEAP` 会被放到 `.rodata` 段
// 使用 `static mut` 可以将 `HEAP` 放到 `.bss` 段
// .bss: uninitialized global variables
// .rodata: const global variables
static mut HEAP: [u8; USER_HEAP_SIZE] = [0; USER_HEAP_SIZE];

/// 初始化堆分配器
#[allow(static_mut_refs)]
pub fn init_heap() {
    unsafe {
        HEAP_ALLOCATOR
            .lock()
            .init(HEAP.as_ptr() as usize, USER_HEAP_SIZE);
    }
}

/// 堆分配错误处理函数
///
/// ## 参数
/// - `layout`: 分配错误的内存布局
/// ## 返回值
/// 该函数不会返回，直接 panic
#[alloc_error_handler]
fn alloc_error_handler(layout: core::alloc::Layout) -> ! {
    panic!("Heap allocation error: {:?}", layout)
}
