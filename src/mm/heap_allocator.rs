use slab_allocator::LockedHeap;

use crate::{extern_global, printkln, trace};

pub const KERNEL_HEAP_SIZE: usize = 0x30_0000;

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

// NOTE: 如果不使用 `static mut`，`HEAP` 会被放到 `.rodata` 段
// 使用 `static mut` 可以将 `HEAP` 放到 `.bss` 段
// .bss: uninitialized global variables
// .rodata: const global variables
static mut HEAP: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];

#[allow(static_mut_refs)]
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

#[alloc_error_handler]
fn alloc_error_handler(layout: core::alloc::Layout) -> ! {
    panic!("Heap allocation error: {:?}", layout)
}

#[allow(unused)]
pub fn heap_test() {
    use alloc::boxed::Box;
    use alloc::vec::Vec;
    let bss_range = extern_global!(__bss_start) as usize..extern_global!(__bss_end) as usize;
    let a = Box::new(5);
    assert_eq!(*a, 5);
    assert!(bss_range.contains(&(a.as_ref() as *const _ as usize)));
    drop(a);
    let mut v: Vec<usize> = Vec::new();
    for i in 0..500 {
        v.push(i);
    }
    for i in 0..500 {
        assert_eq!(v[i], i);
    }
    assert!(bss_range.contains(&(v.as_ptr() as usize)));
    drop(v);
    printkln!("heap_test passed!");
}
