//! 磁盘 Inode 结构
//!
//! 磁盘 Inode 是实际存储在磁盘上的 Inode 结构，包含了文件的元数据信息和数据块索引。
//!
//! 一个 Inode 包含了文件的元数据信息，包括文件类型、大小、数据块索引等。
//!
//! 此实现使用了二级混合索引结构，包含了直接索引、一级索引和二级索引。
//!
//! 直接索引包含了 28 个直接索引块，一级索引包含了 128 个间接索引块，二级索引包含了 128 个一级索引块。
use alloc::{sync::Arc, vec::Vec};

use crate::{
    block_cache::{BLOCK_SIZE, get_cache},
    block_dev::BlockDevice,
};

const INODE_DIRECT_BLOCKS: usize = 28;
const INODE_INDIRECT_BLOCKS: usize = BLOCK_SIZE / 4;
const INODE_DOUBLE_INDIRECT_START: usize = INODE_DIRECT_BLOCKS + INODE_INDIRECT_BLOCKS;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InodeType {
    File,
    Dir,
}

pub type IndirectBlock = [u32; INODE_INDIRECT_BLOCKS];
pub type DataBlock = [u8; BLOCK_SIZE];

/// 磁盘 Inode 结构
///
/// 一个 Inode 包含了文件的元数据信息，包括文件类型、大小、数据块索引等。
///
/// Inode 结构的大小为 64 字节，包含了文件大小、直接索引、一级索引、二级索引等信息。
///
/// - 文件大小：文件的大小，单位为字节。
/// - 直接索引：直接索引表，包含了 28 个直接索引块。
/// - 一级索引：一级索引表，包含了 128 个间接索引块。
/// - 二级索引：二级索引表，包含了 128 个一级索引块。
#[repr(C)]
pub struct DiskInode {
    r#type: InodeType,
    pub size: u32,
    /// 直接索引
    pub direct: [u32; INODE_DIRECT_BLOCKS],
    /// 一级索引
    pub indirect: u32,
    /// 二级索引
    pub double_indirect: u32,
}

impl DiskInode {
    /// 初始化 Inode 结构
    pub fn init(&mut self, r#type: InodeType) {
        self.size = 0;
        self.r#type = r#type;
        self.direct.iter_mut().for_each(|x| *x = 0);
        self.indirect = 0;
        self.double_indirect = 0;
    }

    /// 获取 Inode 类型
    pub fn get_type(&self) -> InodeType {
        self.r#type
    }

    /// 获取直接索引块
    ///
    /// 不检查索引是否越界
    fn get_direct_block(&self, offset: usize) -> u32 {
        self.direct[offset]
    }

    /// 获取一级索引块
    ///
    /// 不检查索引是否越界
    fn get_indirect_block(&self, offset: usize, dev: &Arc<dyn BlockDevice>) -> u32 {
        get_cache(self.indirect as usize, dev.clone())
            .expect("cannot get cache")
            .lock()
            .read_at(0, |indirect: &IndirectBlock| {
                indirect[offset as usize - INODE_DIRECT_BLOCKS]
            })
            .expect("cannot read cache")
    }

    /// 计算偏移在二级索引中的位置
    ///
    /// ## 参数
    /// - `offset`：偏移, 需要大于等于 [INODE_DOUBLE_INDIRECT_START]
    ///
    /// ## 返回
    /// 在二级索引中的位置二元组，分别偏移在一级索引表的偏移和二级索引表的偏移，
    /// 如果偏移不在二级索引范围内，返回 None
    fn extract_double_indirect_block(offset: usize) -> Option<(usize, usize)> {
        let start = INODE_DOUBLE_INDIRECT_START;
        if offset < start {
            return None;
        }
        let offset = offset - start;
        let level1_offset = offset / INODE_INDIRECT_BLOCKS;
        let level2_offset = offset % INODE_INDIRECT_BLOCKS;
        Some((level1_offset, level2_offset))
    }

    /// 获取二级索引块
    ///
    /// 不检查索引是否越界
    fn get_double_indirect_block(&self, offset: usize, dev: &Arc<dyn BlockDevice>) -> u32 {
        let (level1_offset, level2_offset) =
            Self::extract_double_indirect_block(offset).expect("invalid offset");
        let cache =
            get_cache(self.double_indirect as usize, dev.clone()).expect("cannot get cache");
        let indirect_block = cache
            .lock()
            .read_at(0, |indirect: &IndirectBlock| indirect[level1_offset])
            .expect("cannot read cache");
        get_cache(indirect_block as usize, dev.clone())
            .expect("cannot get cache")
            .lock()
            .read_at(0, |indirect: &IndirectBlock| indirect[level2_offset])
            .expect("cannot read cache")
    }

    /// 将文件内偏移转换为其对应的数据块索引
    ///
    /// 通过文件内偏移计算数据块索引，如果偏移在直接索引范围内，返回直接索引；
    /// 如果偏移在一级索引范围内，返回一级索引；否则返回二级索引。
    ///
    /// ## 参数
    /// - `offset`：文件内偏移
    /// - `dev`：块设备
    ///
    /// ## 返回
    /// 数据块索引
    pub fn translate(&self, offset: u32, dev: &Arc<dyn BlockDevice>) -> u32 {
        let offset = offset as usize;
        if offset < INODE_DIRECT_BLOCKS {
            self.get_direct_block(offset)
        } else if offset < INODE_DOUBLE_INDIRECT_START {
            self.get_indirect_block(offset, dev)
        } else {
            self.get_double_indirect_block(offset, dev)
        }
    }

    /// 计算文件大小所需的数据块数
    ///
    /// 通过文件大小计算所需的数据块数，不包括索引块。
    ///
    /// 由于一个数据块只能存储一个文件块，因此文件大小除以数据块大小向上取整即可得到所需的数据块数。
    fn calc_required_block_for(size: u32) -> u32 {
        size.div_ceil(BLOCK_SIZE as u32)
    }

    /// 计算文件大小所需的数据块数
    pub fn required_data_blocks_for(&self, size: u32) -> u32 {
        Self::calc_required_block_for(size)
    }

    /// 计算文件大小所需的总块数
    ///
    /// 通过文件大小计算所需的总块数，包括数据块和索引块。
    ///
    /// 在计算出数据块数后，需要考虑索引块的情况：
    /// - 如果数据块数小于等于直接索引块数，不需要额外的索引块；
    /// - 如果数据块数大于直接索引块数，需要一级索引块；
    /// - 如果数据块数大于一级索引块数，需要二级索引块，二级索引块中的每个索引指向一个一级索引块，共需要 (数据块数 - 一级索引块数) / 一级索引块数 个索引块。
    pub fn required_blocks_for(&self, size: u32) -> u32 {
        let data_blocks = Self::calc_required_block_for(size) as usize;
        let mut index_blocks = 0;
        if data_blocks > INODE_DIRECT_BLOCKS {
            index_blocks += 1;
        }
        if data_blocks > INODE_DOUBLE_INDIRECT_START {
            index_blocks += 1;
            index_blocks +=
                (data_blocks - INODE_DOUBLE_INDIRECT_START).div_ceil(INODE_INDIRECT_BLOCKS);
        }
        data_blocks as u32 + index_blocks as u32
    }

    /// 计算增加文件大小还需的总块数
    pub fn required_delta_blocks_for(&self, new_size: u32) -> u32 {
        assert!(new_size >= self.size);
        self.required_blocks_for(new_size) - self.required_blocks_for(self.size)
    }

    /// 扩展文件大小
    ///
    /// 通过新的文件大小和新分配的数据块列表扩展文件大小。
    ///
    /// 调用该方法前需要调用 [Self::required_delta_blocks_for] 方法计算所需的总块数，
    /// 并申请足够的数据块。
    ///
    /// 在扩展文件大小时，会将新分配的数据块依次填充到直接索引、一级索引和二级索引中。
    pub fn extend_size(&mut self, new_size: u32, new_blocks: Vec<u32>, dev: &Arc<dyn BlockDevice>) {
        // 从 offset 开始分配新块
        let mut offset = self.required_blocks_for(self.size) as usize;
        self.size = new_size;

        // 二级索引的索引表写入偏移
        // 当前分配偏移未达到二级索引时，从 0 开始
        let (mut level1_offset, mut level2_offset) =
            Self::extract_double_indirect_block(offset).unwrap_or((0, 0));

        let mut new_blocks = new_blocks.into_iter().peekable();

        // 从直接索引开始分配
        while offset < INODE_DIRECT_BLOCKS {
            if let Some(block) = new_blocks.next() {
                self.direct[offset] = block;
                offset += 1;
            } else {
                return;
            }
        }

        if offset == INODE_DIRECT_BLOCKS {
            // 分配一级索引表块
            self.indirect = new_blocks.next().unwrap();
        }

        let indirect_table_cache =
            get_cache(self.indirect as usize, dev.clone()).expect("cannot get cache");

        while offset < INODE_DOUBLE_INDIRECT_START {
            if let Some(block) = new_blocks.next() {
                indirect_table_cache
                    .lock()
                    .modify_at(0, |indirect: &mut IndirectBlock| {
                        indirect[offset - INODE_DIRECT_BLOCKS] = block;
                    })
                    .expect("cannot modify cache");
                offset += 1;
            } else {
                return;
            }
        }

        if offset == INODE_DOUBLE_INDIRECT_START {
            // 分配二级索引表的一级索引块
            self.double_indirect = new_blocks.next().unwrap();
        }

        let double_indirect_level1_cache =
            get_cache(self.double_indirect as usize, dev.clone()).expect("cannot get cache");

        // 二级索引块缓存
        let mut double_indirect_level2_cache = None;

        while new_blocks.peek().is_some() {
            if level2_offset == 0 && double_indirect_level2_cache.is_none() {
                // 分配二级索引表中的二级索引块
                // 写入二级索引表的一级索引块
                let block = new_blocks.next().unwrap();
                double_indirect_level1_cache
                    .lock()
                    .modify_at(0, |indirect: &mut IndirectBlock| {
                        indirect[level1_offset] = block;
                    })
                    .expect("cannot modify cache");
                double_indirect_level2_cache =
                    Some(get_cache(block as usize, dev.clone()).expect("cannot get cache"));

                // 更新索引偏移
                level1_offset += 1;
            } else {
                // 分配二级索引表中的数据块
                // 写入二级索引表的二级索引块
                let block = new_blocks.next().unwrap();
                double_indirect_level2_cache
                    .as_ref()
                    .unwrap()
                    .lock()
                    .modify_at(0, |indirect: &mut IndirectBlock| {
                        indirect[level2_offset] = block;
                    })
                    .expect("cannot modify cache");

                // 更新索引偏移
                if level2_offset == INODE_INDIRECT_BLOCKS - 1 {
                    level2_offset = 0;
                    double_indirect_level2_cache = None;
                } else {
                    level2_offset += 1;
                }
            }
        }
    }

    /// 清空 Inode 的数据块
    ///
    /// 清空 Inode 的数据块，返回被回收的数据块列表。
    pub fn clear_size(&mut self, dev: &Arc<dyn BlockDevice>) -> Vec<u32> {
        let mut recycled = Vec::new();
        self.size = 0;
        for i in 0..INODE_DIRECT_BLOCKS {
            if self.direct[i] != 0 {
                recycled.push(self.direct[i]);
                self.direct[i] = 0;
            }
        }
        if self.indirect != 0 {
            let cache = get_cache(self.indirect as usize, dev.clone()).expect("cannot get cache");
            cache
                .lock()
                .read_at(0, |indirect: &IndirectBlock| {
                    for &block in indirect.iter().filter(|&&b| b != 0) {
                        recycled.push(block);
                    }
                })
                .expect("cannot read cache");
            recycled.push(self.indirect);
            self.indirect = 0;
        }
        if self.double_indirect != 0 {
            let cache =
                get_cache(self.double_indirect as usize, dev.clone()).expect("cannot get cache");
            cache
                .lock()
                .read_at(0, |indirect: &IndirectBlock| {
                    for &block in indirect.iter().filter(|&&b| b != 0) {
                        let cache =
                            get_cache(block as usize, dev.clone()).expect("cannot get cache");
                        cache
                            .lock()
                            .read_at(0, |indirect: &IndirectBlock| {
                                for &block in indirect.iter().filter(|&&b| b != 0) {
                                    recycled.push(block);
                                }
                            })
                            .expect("cannot read cache");
                        recycled.push(block);
                    }
                })
                .expect("cannot read cache");
            recycled.push(self.double_indirect);
            self.double_indirect = 0;
        }

        recycled
    }

    /// 读取 Inode 的数据
    ///
    /// 从 Inode 中读取数据到缓冲区，返回实际读取的字节数。
    ///
    /// ## 参数
    /// - `offset`：读取偏移
    /// - `dev`：块设备
    /// - `buf`：缓冲区
    pub fn read_at(&self, mut offset: usize, dev: &Arc<dyn BlockDevice>, buf: &mut [u8]) -> usize {
        let mut read = 0;
        let end = (offset + buf.len()).min(self.size as usize);
        if offset >= end {
            return 0;
        }
        let mut offset_block = offset / BLOCK_SIZE;
        loop {
            let current_block_end = ((offset / BLOCK_SIZE + 1) * BLOCK_SIZE).min(end);
            let size_to_read = current_block_end - offset;
            let dst = &mut buf[read..read + size_to_read];
            let cache = get_cache(
                self.translate(offset_block as u32, dev) as usize,
                dev.clone(),
            )
            .expect("cannot get cache");
            cache.lock().read_at(0, |block: &DataBlock| {
                let start = offset % BLOCK_SIZE;
                dst.copy_from_slice(&block[start..start + size_to_read]);
            });
            read += size_to_read;
            offset_block += 1;
            offset = current_block_end;
            if current_block_end == end
            /*|| (read == buf.len()) */
            {
                break;
            }
        }
        read
    }

    /// 写入 Inode 的数据
    ///
    /// 将缓冲区中的数据写入 Inode，返回实际写入的字节数。
    ///
    /// ## 参数
    /// - `offset`：写入偏移
    /// - `dev`：块设备
    /// - `buf`：缓冲区
    ///
    /// ## 返回
    /// 实际写入的字节数
    pub fn write_at(&mut self, mut offset: usize, dev: &Arc<dyn BlockDevice>, buf: &[u8]) -> usize {
        let mut written = 0;
        let end = (offset + buf.len()).min(self.size as usize);
        if offset >= end {
            return 0;
        }
        // if end as u32 > self.size {
        //     let new_size = end as u32;
        //     let new_blocks = todo!();
        //     self.extend_size(new_size, new_blocks, dev);
        // }
        let mut offset_block = offset / BLOCK_SIZE;
        loop {
            let current_block_end = ((offset / BLOCK_SIZE + 1) * BLOCK_SIZE).min(end);
            let size_to_write = current_block_end - offset;
            let src = &buf[written..written + size_to_write];
            let cache = get_cache(
                self.translate(offset_block as u32, dev) as usize,
                dev.clone(),
            )
            .expect("cannot get cache");
            cache.lock().modify_at(0, |block: &mut DataBlock| {
                let start = offset % BLOCK_SIZE;
                block[start..start + size_to_write].copy_from_slice(src);
            });
            written += size_to_write;
            offset_block += 1;
            offset = current_block_end;
            if current_block_end == end
            /*|| (written == buf.len()) */
            {
                break;
            }
        }
        self.size = self.size.max(end as u32);
        written
    }
}
