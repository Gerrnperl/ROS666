use alloc::sync::Arc;
use spin::Mutex;

use crate::{
    bitmap::Bitmap,
    block_cache::{BLOCK_SIZE, get_cache},
    block_dev::BlockDevice,
    layout::{
        disk_inode::{DataBlock, DiskInode, InodeType},
        super_block::SuperBlock,
    },
    virt_fs::MemInode,
};

pub struct FileSystem {
    pub dev: Arc<dyn BlockDevice>,
    pub inode_bitmap: Bitmap,
    pub data_bitmap: Bitmap,
    inode_area_start: u32,
    data_area_start: u32,
}

impl FileSystem {
    pub fn new(
        dev: Arc<dyn BlockDevice>,
        total_blocks: u32,
        inode_bitmap_blocks: u32,
    ) -> Arc<Mutex<Self>> {
        let inode_bitmap = Bitmap::new(1, inode_bitmap_blocks as usize);
        let inode_count = inode_bitmap.maximum();
        // 所有 inode 所需要的块数
        let inode_area_blocks =
            (inode_count * core::mem::size_of::<DiskInode>()).div_ceil(BLOCK_SIZE) as u32;
        let inode_total_blocks = inode_bitmap_blocks + inode_area_blocks;
        let remaining_blocks = total_blocks - inode_total_blocks - 1;
        let data_bitmap_blocks = remaining_blocks.div_ceil(4097);
        let data_area_blocks = remaining_blocks - data_bitmap_blocks;
        let data_bitmap = Bitmap::new(
            (1 + inode_bitmap_blocks + inode_area_blocks) as usize,
            data_bitmap_blocks as usize,
        );
        let fs = Self {
            dev: dev.clone(),
            inode_bitmap,
            data_bitmap,
            inode_area_start: 1 + inode_bitmap_blocks,
            data_area_start: 1 + inode_total_blocks + data_bitmap_blocks,
        };
        // write zero
        for i in 0..total_blocks {
            let cache = get_cache(i as usize, dev.clone()).expect("get cache failed");
            cache.lock().modify_at(0, |data_block: &mut DataBlock| {
                data_block.iter_mut().for_each(|x| *x = 0);
            });
        }
        // super block
        let cache = get_cache(0, dev.clone()).expect("get cache failed");
        cache.lock().modify_at(0, |super_block: &mut SuperBlock| {
            super_block.init_with(
                total_blocks,
                inode_bitmap_blocks,
                inode_area_blocks,
                data_bitmap_blocks,
                data_area_blocks,
            );
        });
        assert_eq!(fs.alloc_inode(), 0);
        let (root_inode_bid, root_inode_offset) = fs.get_disk_inode_pos(0);
        let cache = get_cache(root_inode_bid as usize, dev.clone()).expect("get cache failed");
        cache
            .lock()
            .modify_at(root_inode_offset as usize, |disk_inode: &mut DiskInode| {
                disk_inode.init(InodeType::Dir);
            });
        Arc::new(Mutex::new(fs))
    }

    pub fn get_disk_inode_pos(&self, inode_id: u32) -> (u32, usize) {
        let inode_size = core::mem::size_of::<DiskInode>();
        let inode_per_block = (BLOCK_SIZE / inode_size) as u32;
        let inode_bid = self.inode_area_start + inode_id / inode_per_block;
        let inode_offset = (inode_id % inode_per_block) as usize * inode_size;
        (inode_bid, inode_offset)
    }

    pub fn get_data_block_id(&self, offset: u32) -> u32 {
        self.data_area_start + offset
    }

    pub fn alloc_inode(&self) -> u32 {
        self.inode_bitmap
            .alloc(&self.dev)
            .expect("alloc inode failed") as u32
    }

    pub fn alloc_data_block(&self) -> u32 {
        self.data_bitmap
            .alloc(&self.dev)
            .expect("alloc data block failed") as u32
            + self.data_area_start
    }

    pub fn free_inode(&self, inode_id: u32) {
        self.inode_bitmap.free(&self.dev, inode_id as usize);
    }

    pub fn free_data_block(&self, data_block_id: u32) {
        let cache = get_cache(data_block_id as usize, self.dev.clone()).expect("get cache failed");
        cache.lock().modify_at(0, |data_block: &mut DataBlock| {
            data_block.iter_mut().for_each(|x| *x = 0);
        });
        self.data_bitmap
            .free(&self.dev, (data_block_id - self.data_area_start) as usize);
    }

    pub fn open(dev: Arc<dyn BlockDevice>) -> Arc<Mutex<Self>> {
        let cache = get_cache(0, dev.clone()).expect("get cache failed");
        let fs = cache
            .lock()
            .read_at(0, |super_block: &SuperBlock| {
                assert!(super_block.check(), "super block magic number error");

                let inode_total_blocks =
                    super_block.inode_bitmap_blocks + super_block.inode_area_blocks;

                let inode_bitmap = Bitmap::new(1, super_block.inode_bitmap_blocks as usize);
                let data_bitmap = Bitmap::new(
                    (1 + inode_total_blocks) as usize,
                    super_block.data_bitmap_blocks as usize,
                );

                Self {
                    dev,
                    inode_bitmap,
                    data_bitmap,
                    inode_area_start: 1 + super_block.inode_bitmap_blocks,
                    data_area_start: 1 + inode_total_blocks + super_block.data_bitmap_blocks,
                }
            })
            .expect("read super block failed");
        Arc::new(Mutex::new(fs))
    }
}

pub trait FileSystemRootInode {
    fn root_inode(&self) -> MemInode;
}

impl FileSystemRootInode for Arc<Mutex<FileSystem>> {
    fn root_inode(&self) -> MemInode {
        let fs = self.lock();
        let (root_inode_bid, root_inode_offset) = fs.get_disk_inode_pos(0);
        MemInode::new(
            root_inode_bid as usize,
            root_inode_offset as usize,
            self.clone(),
            fs.dev.clone(),
        )
    }
}
