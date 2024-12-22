//! 超级块
//!
//! 用于存储文件系统的基本信息，一个文件系统只有一个超级块
//!
//! 一个超级块包含了 inode 和 数据块 的位示图及其内容区域的大小
//!
//! 在超级块的起始位置，存储了一个魔数，用于标识超级块

/// 超级块
///
/// 用于存储文件系统的基本信息，
/// 一个文件系统只有一个超级块
///
/// ## 布局
///
/// | 字段 | 类型 | 描述 |
/// | --- | --- | --- |
/// | magic | u32 | 魔数，用于标识超级块 |
/// | total_blocks | u32 | 总块数 |
/// | inode_bitmap_blocks | u32 | inode 位示图块数 |
/// | inode_area_blocks | u32 | inode 区块数 |
/// | data_bitmap_blocks | u32 | 数据位示图块数 |
#[repr(C)]
pub struct SuperBlock {
    magic: u32,
    pub total_blocks: u32,
    pub inode_bitmap_blocks: u32,
    pub inode_area_blocks: u32,
    pub data_bitmap_blocks: u32,
    pub data_area_blocks: u32,
}

const SUPER_BLOCK_MAGIC: u32 = 0x78887C6A; // 2022210666

impl SuperBlock {
    /// 初始化超级块
    pub fn init_with(
        &mut self,
        total_blocks: u32,
        inode_bitmap_blocks: u32,
        inode_area_blocks: u32,
        data_bitmap_blocks: u32,
        data_area_blocks: u32,
    ) {
        *self = Self {
            magic: SUPER_BLOCK_MAGIC,
            total_blocks,
            inode_bitmap_blocks,
            inode_area_blocks,
            data_bitmap_blocks,
            data_area_blocks,
        }
    }

    /// 检查超级块是否有效
    pub fn check(&self) -> bool {
        self.magic == SUPER_BLOCK_MAGIC
    }
}
