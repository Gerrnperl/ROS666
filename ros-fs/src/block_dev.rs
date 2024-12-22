//! 块设备接口
use core::any::Any;

/// 块设备接口
///
/// 块设备是一种随机访问设备，可以按块读写数据。需要提供 [BlockDevice::read_block] 和 [BlockDevice::write_block] 两个方法。
///
/// 实现 `BlockDevice` trait 来提供对特定块设备, 如 virtio block device 和 sd 卡等的访问。
///
/// 块设备的实现需要保证线程安全，块设备的读写操作是阻塞的
pub trait BlockDevice: Send + Sync + Any {
    /// 读取一个块
    ///
    /// ## 参数
    /// - `block_id`：块 ID
    /// - `buf`：目标缓冲区
    fn read_block(&self, block_id: usize, buf: &mut [u8]);
    /// 写入一个块
    ///
    /// ## 参数
    /// - `block_id`：块 ID
    /// - `buf`：源缓冲区
    fn write_block(&self, block_id: usize, buf: &[u8]);
}
