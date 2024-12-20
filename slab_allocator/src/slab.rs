//! `slab` 模块包含 `Slab` 结构体，该结构体表示一个内存块分配器。

use core::{alloc::AllocError, ptr::NonNull};

use crate::Address;

/// `Slab` 结构体表示一个内存块分配器。
///
/// ## 字段
/// - `block_size`：每个内存块的大小
/// - `block_num`：内存块的数量
/// - `free_list`：空闲内存块列表
pub struct Slab {
    pub block_size: usize,
    pub block_num: usize,
    free_list: FreeList,
}

impl Slab {
    /// 创建一个新的空的 `Slab` 实例。
    ///
    /// ## 参数
    /// - `block_size`：每个内存块的大小
    ///
    /// ## 返回
    ///
    /// 具有指定块大小的新 `Slab` 空实例，其中没有内存块。
    pub const fn empty(block_size: usize) -> Self {
        Slab {
            block_size,
            block_num: 0,
            free_list: FreeList::empty(),
        }
    }

    /// 创建一个新的 `Slab` 实例。
    ///
    /// ## 参数
    /// - `start`：内存块的起始地址
    /// - `block_size`：每个内存块的大小
    /// - `block_num`：内存块的数量
    ///
    /// ## 返回
    ///
    /// 具有指定参数的新 `Slab` 实例。
    pub fn new(start: Address, block_size: usize, block_num: usize) -> Self {
        Slab {
            block_size,
            block_num,
            free_list: FreeList::new(start, block_size, block_num),
        }
    }

    /// 扩展 `Slab` 实例的内存块。
    ///
    /// ## 参数
    /// - `start`：新内存块的起始地址
    /// - `slab_size`：新内存块的总大小
    pub fn grow(&mut self, start: Address, slab_size: usize) {
        let block_num = slab_size / self.block_size;
        self.block_num += block_num;
        // 添加到 self.free_list
        for i in 0..block_num {
            let block = (start + i * self.block_size) as *mut FreeBlock;
            let block = unsafe { &mut *block };
            self.free_list.push(block);
        }
    }

    /// 从 `Slab` 实例中分配一个内存块。
    ///
    /// ## 返回
    ///
    /// 如果成功分配内存块，则返回内存块的地址；否则返回 `None`。
    pub fn alloc(&mut self) -> Result<NonNull<u8>, AllocError> {
        self.free_list
            .pop()
            .map(|block| unsafe { NonNull::new_unchecked(block as *mut FreeBlock as *mut u8) })
            .ok_or(AllocError)
    }

    /// 将一个内存块释放回 `Slab` 实例。
    ///
    /// ## 参数
    /// - `addr`：要释放的内存块的地址
    pub fn dealloc(&mut self, addr: NonNull<u8>) {
        let block = addr.as_ptr() as *mut FreeBlock;
        let block = unsafe { &mut *block };
        self.free_list.push(block);
    }
}

/// 内存块空闲列表
struct FreeList {
    /// 空闲列表的长度
    len: usize,
    /// 空闲列表的头部节点
    head: Option<&'static mut FreeBlock>,
}

/// 表示一个空闲的内存块
struct FreeBlock {
    /// 指向下一个空闲块的可选引用
    next: Option<&'static mut FreeBlock>,
}

/// 为 `FreeList` 实现 `Drop` 特性，以便在 `FreeList` 被销毁时释放所有节点。
impl Drop for FreeList {
    fn drop(&mut self) {
        let mut current = self.head.take();
        while let Some(node) = current {
            current = node.next.take();
        }
    }
}

impl FreeList {
    /// 建立一个新的空闲列表
    pub const fn empty() -> Self {
        FreeList { len: 0, head: None }
    }

    /// 建立一个新的空闲列表，其中包含指定数量的空闲块。
    ///
    /// ## 参数
    /// - `start`：内存块的起始地址
    /// - `block_size`：每个内存块的大小
    /// - `block_num`：内存块的数量
    ///
    /// ## 返回
    ///
    /// 具有指定参数的新 `FreeList` 实例。
    pub fn new(start: Address, block_size: usize, block_num: usize) -> Self {
        let mut list = FreeList { len: 0, head: None };
        for i in (0..block_num).rev() {
            let block = (start + i * block_size) as *mut FreeBlock;
            let block = unsafe { &mut *block };
            list.push(block);
        }
        list
    }

    /// 将一个空闲块推入空闲列表。
    ///
    /// ## 参数
    /// - `free_block`：要添加到列表中的空闲块的可变引用。
    pub fn push(&mut self, free_block: &'static mut FreeBlock) {
        free_block.next = self.head.take();
        self.head = Some(free_block);
        self.len += 1;
    }

    /// 从空闲列表中弹出一个空闲块。
    ///
    /// ## 返回
    ///
    /// 如果列表不为空，则包含对空闲块的可变引用的 `Option`，如果列表为空，则返回 `None`。
    pub fn pop(&mut self) -> Option<&'static mut FreeBlock> {
        self.head.take().map(|free_block| {
            self.head = free_block.next.take();
            self.len -= 1;
            free_block
        })
    }
}
