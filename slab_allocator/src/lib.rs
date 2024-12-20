//! 基于 Slab Allocator 的堆内存分配器
//! 使用 Linked List Allocator 作为备选分配器，分配大于 4096 字节的内存
//!
//! # Example
//! ```no_run
//! use slab_allocator::LockedHeap;
//!
//! let mut heap = LockedHeap::empty();
//! unsafe {
//!    heap.lock().init(begin, size);
//! }
//! ```
//! Credits:
//! - 接口参考 [buddy_system_allocator - A buddy system allocator in pure Rust.](https://github.com/rcore-os/buddy_system_allocator) (MIT License)
//! - 实现参考 [slab_allocator - Slab allocator for no_std systems. ](https://github.com/weclaw1/slab_allocator/tree/master) (MIT License)

#![no_std]
#![feature(allocator_api)]

// 引入核心库中的分配器模块、分配错误和布局
use core::{
    alloc::{self, AllocError, Layout},
    ptr::NonNull,
};

// 引入链表分配器和 Slab 模块
use linked_list_allocator;
use slab::Slab;

// 如果启用了 "use_spin" 功能，则引入 spin::Mutex
#[cfg(feature = "use_spin")]
use spin::Mutex;

// 引入 slab 模块
mod slab;

/// 地址类型
pub type Address = usize;

pub const MIN_ALLOC_SIZE: usize = 64;
pub const MAX_SLAB_SIZE: usize = 4096;
/// 所有 Slab 和 Fallback 的数量
pub const ALLOCATORS_NUM: usize = 8;
pub const SLABS_NUM: usize = ALLOCATORS_NUM - 1;

/// 根据大小获取对应的 Slab 索引
///
/// ## 参数
/// - `size`: 要分配的内存大小
/// ## 返回
/// 返回对应的 Slab 索引，如果大小超过最大 Slab 大小，则返回 Fallback 分配器的索引
pub fn get_slab_index(mut size: usize) -> usize {
    if size <= MIN_ALLOC_SIZE {
        return 0;
    }
    if size > MAX_SLAB_SIZE {
        return ALLOCATORS_NUM - 1; // 回退到链表分配器
    }
    // 64 -> 0, 65-128 -> 1, 129-256 -> 2, ...
    size = size.next_power_of_two();
    size.trailing_zeros() as usize - 6
}

/// 堆内存分配器
pub struct Heap {
    /// Slab 分配器数组
    slabs: [slab::Slab; SLABS_NUM],
    /// 回退分配器 (链表分配器)
    fallback: linked_list_allocator::Heap,
    /// 用户请求的字节数
    user: usize,
    /// 实际分配的字节数
    allocated: usize,
    /// 堆中的总字节数
    total: usize,
}

/// 分配器类型
enum AllocType {
    /// Slab 分配器，包含索引
    Slab(usize),
    /// 回退分配器
    Fallback,
}

impl Heap {
    /// 创建一个空的堆内存分配器
    pub const fn new() -> Self {
        Heap {
            // 初始化 Slab 分配器数组，每个 Slab 分配器的块大小分别为 64, 128, 256, 512, 1024, 2048, 4096 字节
            slabs: [
                Slab::empty(64),
                Slab::empty(128),
                Slab::empty(256),
                Slab::empty(512),
                Slab::empty(1024),
                Slab::empty(2048),
                Slab::empty(4096),
            ],
            // 初始化回退分配器（链表分配器）
            fallback: linked_list_allocator::Heap::empty(),
            // 初始化用户请求的字节数为 0
            user: 0,
            // 初始化实际分配的字节数为 0
            allocated: 0,
            // 初始化堆中的总字节数为 0
            total: 0,
        }
    }

    /// 创建一个空的堆内存分配器
    pub fn empty() -> Self {
        Self::new()
    }

    /// 根据索引获取分配器类型
    ///
    /// ## 参数
    /// - `index`: 分配器索引
    /// ## 返回
    /// 返回对应的分配器类型，如果索引小于 ALLOCATORS_NUM - 1，则返回 Slab 分配器，
    /// 否则返回 Fallback 分配器
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
        // 避免在有些平台上出现未对齐的访问
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
        // 避免在有些平台上出现未对齐的访问
        start = (start + size_of::<usize>() - 1) & (!size_of::<usize>() + 1);
        unsafe {
            // 初始化链表分配器
            self.fallback.init(start as *mut u8, size);
        }
        // 更新堆中的总字节数
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

    /// 初始化堆内存分配器
    ///
    /// ## 参数
    /// - `start`: 内存起始地址
    /// - `size`: 内存大小
    /// ## 说明
    /// 将内存分配给 Slab 分配器和 Fallback 分配器
    pub unsafe fn init(&mut self, start: usize, size: usize) {
        // 计算每个分配器的分配大小
        let alloc_size = size / ALLOCATORS_NUM;
        // 计算所有 Slab 分配器的总大小
        let total_slab_size = alloc_size * SLABS_NUM;
        // 计算 Fallback 分配器的大小
        let fallback_size = alloc_size;
        unsafe {
            // 添加内存到 Slab 分配器
            self.add_to_heap(start, start + total_slab_size);
            // 初始化 Fallback 分配器
            self.init_fallback(start + total_slab_size, fallback_size);
        }
    }

    /// 根据内存布局选择合适的分配器
    ///
    /// ## 参数
    /// - `layout`: 内存布局
    /// ## 返回
    /// 返回对应的分配器类型，如果大小小于等于最大 Slab 大小，则返回 Slab 分配器，
    /// 否则返回 Fallback 分配器
    fn select_allocator(&self, layout: &Layout) -> AllocType {
        let size = layout.size();
        let index = get_slab_index(size);
        self.get_inner(index)
    }

    /// 从堆中分配一段满足 `layout` 要求的内存
    ///
    /// ## 参数
    /// - `layout`: 内存布局
    /// ## 返回
    /// 返回一个指向分配内存的指针，如果分配失败，则返回错误
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
    /// 用于格式化输出 Heap 结构体的信息
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Heap {{ user: {}, allocated: {}, total: {} }}",
            // 获取用户分配的内存
            self.stats_alloc_user(),
            // 获取实际分配的内存
            self.stats_alloc_actual(),
            // 获取总内存
            self.stats_total_bytes()
        )
    }
}

/// 一个锁定版本的 `Heap`
///
/// # 用法
/// 创建一个锁定的堆并添加一个内存区域:
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
///     // 或者添加到堆中
///     heap.lock().add_to_heap(begin, end);
/// }
/// ```
#[cfg(feature = "use_spin")]
/// 一个锁定版本的 `Heap`
pub struct LockedHeap(Mutex<Heap>);

#[cfg(feature = "use_spin")]
impl LockedHeap {
    /// 创建一个新的 `LockedHeap`
    pub fn new() -> Self {
        LockedHeap(Mutex::new(Heap::new()))
    }

    /// 锁定堆并获取内部 `Heap` 的可变引用
    pub fn lock(&self) -> spin::MutexGuard<Heap> {
        self.0.lock()
    }

    /// 创建一个空的堆
    pub const fn empty() -> Self {
        LockedHeap(Mutex::new(Heap::new()))
    }
}

#[cfg(feature = "use_spin")]
impl core::ops::Deref for LockedHeap {
    type Target = Mutex<Heap>;

    /// 实现 Deref trait，使 LockedHeap 可以像引用 Mutex<Heap> 一样使用
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(feature = "use_spin")]
/// # Safety
/// 这个实现是 `GlobalAlloc` trait 的一个不安全实现，
/// 需要确保在使用过程中不会违反 Rust 的内存安全规则。
///
/// # 注意事项
/// 由于这些方法都是不安全的（`unsafe`），调用者必须确保传入的参数是有效的，
/// 并且在调用这些方法时不会导致未定义行为。
unsafe impl alloc::GlobalAlloc for LockedHeap {
    /// # 分配器
    /// `alloc` 方法用于分配内存，根据传入的 `layout` 参数返回一个指向分配内存的指针。
    /// 如果分配失败，返回一个空指针。
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.0
            .lock()
            .alloc(layout)
            .ok()
            .map_or(core::ptr::null_mut(), |allocation| allocation.as_ptr())
    }

    /// # 释放器
    /// `dealloc` 方法用于释放内存，根据传入的指针 `ptr` 和 `layout` 参数释放对应的内存。
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.0
            .lock()
            .dealloc(unsafe { NonNull::new_unchecked(ptr) }, layout)
    }
}
