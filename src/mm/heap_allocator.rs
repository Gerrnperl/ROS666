//! 堆内存分配器
use slab_allocator::LockedHeap;

// 引入外部全局变量和打印宏
use crate::{extern_global, printkln, trace};

// 定义内核堆大小
pub const KERNEL_HEAP_SIZE: usize = 0x30_0000;

// 定义全局分配器
#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

// NOTE: 如果不使用 `static mut`，`HEAP` 会被放到 `.rodata` 段
// 使用 `static mut` 可以将 `HEAP` 放到 `.bss` 段
// .bss: uninitialized global variables
// .rodata: const global variables
static mut HEAP: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];

#[allow(static_mut_refs)]
/// 初始化堆内存分配器
pub fn init_heap() {
    unsafe {
        trace!(
            "HEAP BASE: {:#x}, SIZE: {:#x}",
            HEAP.as_ptr() as usize,
            KERNEL_HEAP_SIZE
        );
        HEAP_ALLOCATOR
            .lock()
            .init(HEAP.as_ptr() as usize, KERNEL_HEAP_SIZE);
    }
}

/// 分配错误处理函数
#[alloc_error_handler]
fn alloc_error_handler(layout: core::alloc::Layout) -> ! {
    panic!("Heap allocation error: {:?}", layout)
}

#[allow(unused)]
/// 堆内存分配测试函数
pub fn heap_test() {
    use alloc::boxed::Box;
    use alloc::vec::Vec;

    // 获取 .bss 段的范围
    let bss_range = extern_global!(__bss_start) as usize..extern_global!(__bss_end) as usize;

    // 测试 Box 分配
    let a = Box::new(5);
    assert_eq!(*a, 5);
    assert!(bss_range.contains(&(a.as_ref() as *const _ as usize)));
    drop(a);

    // 测试 Vec 分配
    let mut v: Vec<usize> = Vec::new();
    for i in 0..500 {
        v.push(i);
    }
    for i in 0..500 {
        assert_eq!(v[i], i);
    }
    assert!(bss_range.contains(&(v.as_ptr() as usize)));
    drop(v);

    // 打印测试通过信息
    printkln!("heap_test passed!");
}
