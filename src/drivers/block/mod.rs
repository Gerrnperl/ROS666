//! 块设备驱动程序
//!
//! 块设备驱动程序用于访问块设备。块设备驱动程序的实现需要实现 `BlockDevice` trait 以供文件系统模块使用。
use alloc::sync::Arc;
use lazy_static::lazy_static;
use ros_fs::block_dev::BlockDevice;

pub mod sdcard;
pub mod virtio_block;

#[cfg(feature = "qemu")]
type BlockDeviceImpl = virtio_block::VirtIOBlock;
#[cfg(feature = "qemu")]
pub use virtio_block::MMIO;

#[cfg(feature = "k210")]
type BlockDeviceImpl = sdcard::SDCardWrapper;
#[cfg(feature = "k210")]
pub use sdcard::MMIO;

lazy_static! {
    pub static ref BLOCK_DEVICE: Arc<dyn BlockDevice> = Arc::new(unsafe { BlockDeviceImpl::new() });
}
