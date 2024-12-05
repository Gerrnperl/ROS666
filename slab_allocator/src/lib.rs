//! 基于 Slab Allocator 的堆内存分配器
//! 使用 Linked List Allocator 作为备选分配器，分配大于 4096 字节的内存
//!
//! # Example
//!
//! ```no_run
//! use slab_allocator::LockedHeap;
//!
//! let mut heap = LockedHeap::empty();
//! unsafe {
//!    heap.lock().init(begin, size);
//! }
//! ```
//!
//! Credits:
//! - 接口参考 [buddy_system_allocator - A buddy system allocator in pure Rust.](https://github.com/rcore-os/buddy_system_allocator) (MIT License)
//! - 实现参考 [slab_allocator - Slab allocator for no_std systems. ](https://github.com/weclaw1/slab_allocator/tree/master) (MIT License)

#![no_std]
#![feature(allocator_api)]
use core::{
    alloc::{self, AllocError, Layout},
    ptr::NonNull,
};
use linked_list_allocator;
use slab::Slab;

#[cfg(feature = "use_spin")]
use spin::Mutex;
mod slab;

pub type Address = usize;

pub const MIN_ALLOC_SIZE: usize = 64;
pub const MAX_SLAB_SIZE: usize = 4096;
/// 所有 Slab 和 Fallback 的数量
pub const ALLOCATORS_NUM: usize = 8;
pub const SLABS_NUM: usize = ALLOCATORS_NUM - 1;

pub fn get_slab_index(mut size: usize) -> usize {
    if size <= MIN_ALLOC_SIZE {
        return 0;
    }
    if size > MAX_SLAB_SIZE {
        return ALLOCATORS_NUM - 1; // fallback to the linked list allocator
    }
    // 64 -> 0, 65-128 -> 1, 129-256 -> 2, ...
    size = size.next_power_of_two();
    size.trailing_zeros() as usize - 6
}

pub struct Heap {
    slabs: [slab::Slab; SLABS_NUM],
    fallback: linked_list_allocator::Heap,
    user: usize,
    allocated: usize,
    total: usize,
}

enum AllocType {
    Slab(usize),
    Fallback,
}

impl Heap {
    /// Create an empty heap
    pub const fn new() -> Self {
        Heap {
            slabs: [
                Slab::empty(64),
                Slab::empty(128),
                Slab::empty(256),
                Slab::empty(512),
                Slab::empty(1024),
                Slab::empty(2048),
                Slab::empty(4096),
            ],
            fallback: linked_list_allocator::Heap::empty(),
            user: 0,
            allocated: 0,
            total: 0,
        }
    }

    /// Create an empty heap
    pub fn empty() -> Self {
        Self::new()
    }

    fn get_inner(&self, index: usize) -> AllocType {
        if index < ALLOCATORS_NUM - 1 {
            AllocType::Slab(index)
        } else {
            AllocType::Fallback
        }
    }

    /// 添加一段内存[start, end)到堆中
    ///
    /// 由于 Linked List Allocator 需要一个连续的内存块，
    /// 所以不会在此被扩展
    pub unsafe fn add_to_heap(&mut self, mut start: usize, mut end: usize) {
        // avoid unaligned access on some platforms
        start = (start + size_of::<usize>() - 1) & (!size_of::<usize>() + 1);
        end &= !size_of::<usize>() + 1;
        assert!(start <= end);
        let new_heap_size = end - start;
        self.total += new_heap_size;

        let slab_size = new_heap_size / SLABS_NUM;
        for slab_i in 0..SLABS_NUM {
            let slab_start = start + slab_i * slab_size;
            match self.get_inner(slab_i) {
                AllocType::Slab(i) => {
                    self.slabs[i].grow(slab_start, slab_size);
                }
                AllocType::Fallback => {
                    unreachable!("Fallback allocator should not be initialized here");
                }
            }
        }
    }

    /// 初始化 Fallback Allocator (Linked List Allocator)
    ///
    /// 只能被调用一次
    unsafe fn init_fallback(&mut self, mut start: usize, size: usize) {
        start = (start + size_of::<usize>() - 1) & (!size_of::<usize>() + 1);
        unsafe {
            self.fallback.init(start as *mut u8, size);
        }
        self.total += size;
    }

    /// 扩展 Fallback Allocator (Linked List Allocator) 的内存
    ///
    /// 仅扩展 Linked List Allocator 之后的内存，需要确保 Linked List Allocator 已经初始化
    /// 并且内存空间 [start + original_size, start + original_size + size) 未被使用
    pub unsafe fn extend_fallback(&mut self, size: usize) {
        unsafe {
            self.fallback.extend(size);
        }
        self.total += size;
    }

    /// 添加一段内存[start, end)到堆中
    pub unsafe fn init(&mut self, start: usize, size: usize) {
        // self.add_to_heap(start, start + size);
        let alloc_size = size / ALLOCATORS_NUM;
        let total_slab_size = alloc_size * SLABS_NUM;
        let fallback_size = alloc_size;
        unsafe {
            self.add_to_heap(start, start + total_slab_size);
            self.init_fallback(start + total_slab_size, fallback_size);
        }
    }

    fn select_allocator(&self, layout: &Layout) -> AllocType {
        let size = layout.size();
        let index = get_slab_index(size);
        self.get_inner(index)
    }

    /// 从堆中分配一段满足 `layout` 要求的内存
    pub fn alloc(&mut self, layout: Layout) -> Result<NonNull<u8>, AllocError> {
        let size = layout.size();
        match self.select_allocator(&layout) {
            AllocType::Slab(i) => {
                let ret = self.slabs[i].alloc();
                if ret.is_ok() {
                    self.user += size;
                    self.allocated += self.slabs[i].block_size;
                }
                ret
            }
            AllocType::Fallback => {
                let ret = self.fallback.allocate_first_fit(layout);
                if ret.is_ok() {
                    self.user += size;
                    self.allocated += size;
                    Ok(ret.unwrap())
                } else {
                    Err(AllocError)
                }
            }
        }
    }

    /// 从堆中释放一段内存
    pub fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        let size = layout.size();
        match self.select_allocator(&layout) {
            AllocType::Slab(i) => {
                self.slabs[i].dealloc(ptr);
                self.user -= size;
                self.allocated -= self.slabs[i].block_size;
            }
            AllocType::Fallback => {
                unsafe { self.fallback.deallocate(ptr, layout) };
                self.user -= size;
                self.allocated -= size;
            }
        }
    }

    /// 返回用户请求的字节数
    pub fn stats_alloc_user(&self) -> usize {
        self.user
    }

    /// 返回实际分配的字节数
    pub fn stats_alloc_actual(&self) -> usize {
        self.allocated
    }

    /// 返回堆中的总字节数
    pub fn stats_total_bytes(&self) -> usize {
        self.total
    }
}

impl core::fmt::Debug for Heap {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Heap {{ user: {}, allocated: {}, total: {} }}",
            self.stats_alloc_user(),
            self.stats_alloc_actual(),
            self.stats_total_bytes()
        )
    }
}

/// A locked version of `Heap`
///
/// # Usage
///
/// Create a locked heap and add a memory region to it:
/// ```no_run
/// use buddy_system_allocator::*;
/// # use core::mem::size_of;
/// let mut heap = LockedHeap::new();
/// # let space: [usize; 100] = [0; 100];
/// # let begin: usize = space.as_ptr() as usize;
/// # let end: usize = begin + 100 * size_of::<usize>();
/// # let size: usize = 100 * size_of::<usize>();
/// unsafe {
///     heap.lock().init(begin, size);
///     // or
///     heap.lock().add_to_heap(begin, end);
/// }
/// ```
#[cfg(feature = "use_spin")]
pub struct LockedHeap(Mutex<Heap>);

#[cfg(feature = "use_spin")]
impl LockedHeap {
    /// Create a new `LockedHeap`
    pub fn new() -> Self {
        LockedHeap(Mutex::new(Heap::new()))
    }

    /// Lock the heap and get a mutable reference to the inner `Heap`
    pub fn lock(&self) -> spin::MutexGuard<Heap> {
        self.0.lock()
    }

    /// Creates an empty heap
    pub const fn empty() -> Self {
        LockedHeap(Mutex::new(Heap::new()))
    }
}

#[cfg(feature = "use_spin")]
impl core::ops::Deref for LockedHeap {
    type Target = Mutex<Heap>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(feature = "use_spin")]
unsafe impl alloc::GlobalAlloc for LockedHeap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.0
            .lock()
            .alloc(layout)
            .ok()
            .map_or(core::ptr::null_mut(), |allocation| allocation.as_ptr())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.0
            .lock()
            .dealloc(unsafe { NonNull::new_unchecked(ptr) }, layout)
    }
}
