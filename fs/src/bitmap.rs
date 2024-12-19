use alloc::sync::Arc;

use crate::{
    block_cache::{BLOCK_SIZE, get_cache},
    block_dev::BlockDevice,
};

type BitmapBlock = [u64; 64];

const BLOCK_BITS: usize = BLOCK_SIZE * 8;

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

    fn extract_bit_position(mut bit: usize) -> (usize, usize, usize) {
        let block = bit / BLOCK_BITS;
        bit %= BLOCK_BITS;
        let pos = bit / 64;
        let bit = bit % 64;
        (block, pos, bit)
    }

    pub fn dealloc(&self, dev: &Arc<dyn BlockDevice>, bit: usize) {
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
}
