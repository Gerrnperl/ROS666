//! 堆内存分配器

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

/// Rust 语言项 - 分配错误处理函数
///
/// 当分配失败，例如 Out Of Memory 时会调用该函数
#[alloc_error_handler]
fn alloc_error_handler(layout: core::alloc::Layout) -> ! {
    panic!("Heap allocation error: {:?}", layout)
}
