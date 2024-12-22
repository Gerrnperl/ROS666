/// 地址模块
pub mod address;
/// 帧分配器模块
pub mod frame_allocator;
/// 堆分配器模块
pub mod heap_allocator;
/// 初始化模块
pub mod init;
/// 内存设置模块
pub mod memory_set;
/// 页表模块
pub mod page_table;
/// 将 `KERNEL_SPACE` 从 `memory_set` 模块公开使用。
///
/// `KERNEL_SPACE` 表示内核空间的内存设置。
pub use memory_set::KERNEL_SPACE;
