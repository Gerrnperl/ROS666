//! 位示图
//!
//! 用于管理块设备某一范围内的空闲块
//!
//! 位示图由多个块组成，每个块包含 64 个 u64 类型的位
//!
//! 位示图的每一位对应一个物理磁盘块，0 表示空闲，1 表示已分配
use alloc::sync::Arc;

use crate::{
    block_cache::{BLOCK_SIZE, get_cache},
    block_dev::BlockDevice,
};

type BitmapBlock = [u64; 64];

const BLOCK_BITS: usize = BLOCK_SIZE * 8;

/// 位示图
///
/// 用于管理块设备某一范围内的空闲块
///
/// 位示图的每一位对应一个块，0 表示空闲，1 表示已分配
pub struct Bitmap {
    start_block: usize,
    blocks: usize,
}

impl Bitmap {
    pub fn new(start_block: usize, blocks: usize) -> Self {
        Self {
            start_block,
            blocks,
        }
    }

    /// 分配一个空闲块
    ///
    /// 从位示图中找到第一个空闲的块，并返回其在位示图中的偏移
    ///
    /// ## 参数
    /// - `dev`：块设备
    ///
    /// ## 返回
    /// 返回空闲块在位示图中的偏移，如果没有空闲块则返回 None
    pub fn alloc(&self, dev: &Arc<dyn BlockDevice>) -> Option<usize> {
        for id in 0..self.blocks {
            let cache = get_cache((id + self.start_block) as usize, Arc::clone(dev))
                .expect("cannot get cache");
            let pos = cache
                .lock()
                .modify_at(0, |bitmap_block: &mut BitmapBlock| {
                    for (pos, bits64) in bitmap_block.iter_mut().enumerate() {
                        // 找到第一个有空闲的组的第一个为 0（空闲）的位
                        if *bits64 != u64::MAX {
                            let bits64 = bits64.trailing_ones() as usize;
                            bitmap_block[pos] |= 1 << bits64;
                            return Some(id * BLOCK_BITS + pos * 64 + bits64);
                        }
                    }
                    None
                })
                .expect("cannot modify cache");
            if pos.is_some() {
                return pos;
            }
        }
        None
    }

    /// 从位示图中提取块的位置
    fn extract_bit_position(mut bit: usize) -> (usize, usize, usize) {
        let block = bit / BLOCK_BITS;
        bit %= BLOCK_BITS;
        let pos = bit / 64;
        let bit = bit % 64;
        (block, pos, bit)
    }

    /// 释放一个块
    ///
    /// 将指定块标记为空闲
    ///
    /// ## 参数
    /// - `dev`：块设备
    /// - `bit`：块在位示图内的偏移
    pub fn free(&self, dev: &Arc<dyn BlockDevice>, bit: usize) {
        let (block, pos, bit) = Self::extract_bit_position(bit);
        let cache = get_cache((block + self.start_block) as usize, Arc::clone(dev))
            .expect("cannot get cache");
        cache
            .lock()
            .modify_at(0, |bitmap_block: &mut BitmapBlock| {
                assert!(bitmap_block[pos] & (1 << bit) != 0);
                bitmap_block[pos] &= !(1 << bit);
            })
            .expect("cannot modify cache");
    }

    /// 获取位示图的最大块数
    pub fn maximum(&self) -> usize {
        self.blocks * BLOCK_BITS
    }
}
