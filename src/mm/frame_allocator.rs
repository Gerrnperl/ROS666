use super::address::{PhysicalAddress, PhysicalPageNumber};
use crate::{extern_global, printkln, utils::safety::SyncRefCell};
use alloc::vec::Vec;
use core::{cell::RefCell, ops::Range};
use lazy_static::lazy_static;

// 在调试模式下，内存结束地址为 0x84800000
// 在发布模式下，内存结束地址为 0x80800000
// pub const MEMORY_END: usize = 0x80800000;
#[cfg(debug_assertions)]
pub const MEMORY_END: usize = 0x84800000;
#[cfg(not(debug_assertions))]
pub const MEMORY_END: usize = 0x80800000;

/// 帧分配器
///
/// 帧分配器需要实现 `FrameAllocator` trait，提供分配和释放物理页帧的功能。
trait FrameAllocator {
    /// 创建一个新的帧分配器实例。
    ///
    /// ## 返回值
    /// 返回一个新的 `FrameAllocator` 实例。
    fn new() -> Self;

    /// 分配一个物理页帧。
    ///
    /// ## 返回值
    /// 如果成功，返回一个 `PhysicalPageNumber`，否则返回 `None`。
    fn alloc(&mut self) -> Option<PhysicalPageNumber>;

    /// 释放一个物理页帧。
    ///
    /// ## 参数
    /// * `frame` - 要释放的物理页帧。
    fn dealloc(&mut self, frame: PhysicalPageNumber);
}

lazy_static! {
    /// 全局帧分配器
    pub static ref FRAME_ALLOCATOR: SyncRefCell<StackFrameAllocator> = {
        SyncRefCell {
            ref_cell: RefCell::new(StackFrameAllocator::new()),
        }
    };
}

/// 栈帧分配器
///
/// 基于栈的帧分配器，用于分配和释放物理页帧。
///
/// 在栈帧分配器中，未使用的物理页帧范围是一个左闭右开的区间，表示从未使用的物理页帧的起始页号到结束页号的范围。
///
/// 释放帧栈维护了一个已释放的物理页帧列表，当需要分配物理页帧时，
/// 首先从已释放的物理页帧列表中弹出一个物理页帧，如果列表为空，则从未使用的物理页帧范围中分配一个物理页帧。
///
/// 包含未使用的物理页帧范围和已释放的物理页帧列表。
pub struct StackFrameAllocator {
    /// 未使用的物理页帧范围。
    unused: Range<PhysicalPageNumber>,
    /// 已释放的物理页帧列表。
    freed: Vec<PhysicalPageNumber>,
}

impl FrameAllocator for StackFrameAllocator {
    fn new() -> Self {
        Self {
            unused: PhysicalPageNumber(0)..PhysicalPageNumber(0),
            freed: Vec::new(),
        }
    }

    fn alloc(&mut self) -> Option<PhysicalPageNumber> {
        // 如果有已释放的物理页帧，弹出并返回
        if let Some(frame) = self.freed.pop() {
            return Some(frame);
        }
        // 如果还有未使用的物理页帧，分配并返回
        if self.unused.start < self.unused.end {
            let frame = self.unused.start;
            self.unused.start.0 += 1;
            return Some(frame);
        }
        None
    }

    fn dealloc(&mut self, frame: PhysicalPageNumber) {
        // 检查释放的物理页帧是否在未使用的物理页帧范围内以及是否已经释放
        if frame.0 >= self.unused.start.0 {
            panic!("trying to free free page: {:?}", frame);
        }
        if self.freed.iter().any(|&f| f == frame) {
            panic!("trying to free free page: {:?}", frame);
        }
        self.freed.push(frame);
    }
}

impl StackFrameAllocator {
    /// 初始化栈帧分配器
    ///
    /// 将未使用的物理页号范围设置为传入的范围
    pub fn init(&mut self, range: Range<PhysicalPageNumber>) {
        self.unused = range;
    }

    /// 初始化全局栈帧分配器
    ///
    /// 将未使用的物理页号范围设置为从内核结束地址到内存结束地址的范围
    pub fn init_frame_allocator() {
        // 获取内核结束地址，并向上取整到页边界
        let start = PhysicalAddress::from(extern_global!(__kernel_end) as usize)
            .ceil_page()
            .into();
        // 获取内存结束地址，并向下取整到页边界
        let end = PhysicalAddress::from(MEMORY_END).floor_page().into();

        FRAME_ALLOCATOR.ref_cell.borrow_mut().init(start..end);
    }

    /// 从全局栈帧分配器分配一个物理页帧，并返回一个 `FrameTracker` 实例。
    ///
    /// ## 返回
    /// 如果成功，返回一个 `FrameTracker`，否则返回 `None`。
    pub fn alloc_frame() -> Option<FrameTracker> {
        FRAME_ALLOCATOR
            .ref_cell
            .borrow_mut()
            .alloc()
            .map(|frame| FrameTracker::new(frame))
    }

    /// 从全局栈帧分配器释放一个物理页帧。
    ///
    /// ## 参数
    /// * `frame` - 要释放的物理页帧。
    pub fn dealloc_frame(frame: PhysicalPageNumber) {
        FRAME_ALLOCATOR.ref_cell.borrow_mut().dealloc(frame);
    }
}

/// 帧追踪器
///
/// 用于追踪物理页帧的分配和释放，每个帧追踪器实例都会追踪（分配）一个物理页帧
/// 当帧追踪器实例被销毁时，自动释放物理页帧
#[derive(Debug)]
pub struct FrameTracker {
    pub frame: PhysicalPageNumber,
}

impl FrameTracker {
    /// 创建一个新的 FrameTracker 实例
    ///
    /// ## 参数
    /// * `frame` - 一个 PhysicalPageNumber 类型的帧
    /// ## 返回值
    /// 返回一个新的 FrameTracker 实例
    /// ## 示例
    /// ```no_run
    /// let frame = PhysicalPageNumber::new(0);
    /// let tracker = FrameTracker::new(frame);
    /// ```
    /// ## 注意
    /// 在创建新的 FrameTracker 实例时，会将帧中的所有字节清零
    pub fn new(frame: PhysicalPageNumber) -> Self {
        // 清零帧中的所有字节
        frame.get_bytes_array().iter_mut().for_each(|x| *x = 0);
        Self { frame }
    }
}

impl Drop for FrameTracker {
    /// 当 FrameTracker 实例被销毁时，自动释放物理页帧
    fn drop(&mut self) {
        StackFrameAllocator::dealloc_frame(self.frame);
    }
}

/// 测试帧分配器的功能
#[allow(unused)]
pub fn frame_allocator_test() {
    // 创建一个空的 FrameTracker 向量
    let mut v: Vec<FrameTracker> = Vec::new();
    // 分配 5 个物理页帧，并将其添加到向量中
    for i in 0..5 {
        let frame = StackFrameAllocator::alloc_frame().unwrap();
        printkln!("{:?}", frame);
        v.push(frame);
    }
    // 清空向量，释放所有帧
    v.clear();
    // 再次分配 5 个物理页帧，并将其添加到向量中
    for i in 0..5 {
        let frame = StackFrameAllocator::alloc_frame().unwrap();
        printkln!("{:?}", frame);
        v.push(frame);
    }
    // 显式销毁向量，释放所有帧
    drop(v);
    // 打印测试通过信息
    printkln!("frame_allocator_test passed!");
}
