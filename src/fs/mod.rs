use crate::mm::page_table::UserBuffer;

pub mod inode;
pub mod stdio;

/// 文件操作 trait
///
/// 用于实现文件的读写操作
///
/// 一个文件可以被多个进程共享，因此需要实现 Send 和 Sync trait
pub trait File: Send + Sync {
    /// 从文件中读取数据到用户空间
    ///
    /// ## 参数
    /// - `buf`: 用户空间的缓冲区
    ///
    /// ## 返回
    /// 读取的字节数
    fn read(&self, buf: UserBuffer) -> usize;
    /// 将用户空间的数据写入文件
    ///
    /// ## 参数
    /// - `buf`: 用户空间的缓冲区
    ///
    /// ## 返回
    /// 写入的字节数
    fn write(&self, buf: UserBuffer) -> usize;
    /// 根据打开文件的模式判断是否可读
    fn readable(&self) -> bool;
    /// 根据打开文件的模式判断是否可写
    fn writable(&self) -> bool;
}
