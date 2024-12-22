//! 物理文件系统
//!
//! 建立在物理磁盘块设备上的文件系统，提供了文件系统的底层基本操作，包括创建文件系统、打开文件系统、分配 inode、分配数据块等。
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

/// 文件系统
///
/// 一个文件系统包含了一个块设备，一个 inode 位图，一个数据块位图，inode 区域的起始块号，数据区域的起始块号
///
/// 文件系统物理结构：
/// - 超级块 [SuperBlock], 包含了文件系统的元信息
/// - inode 位示图 [Bitmap], 用于标记 inode 块的使用情况
/// - inode 区域, 用于存储 inode 结构
/// - 数据块位示图 [Bitmap], 用于标记数据块的使用情况
/// - 数据区域, 用于存储文件数据
pub struct FileSystem {
    pub dev: Arc<dyn BlockDevice>,
    pub inode_bitmap: Bitmap,
    pub data_bitmap: Bitmap,
    inode_area_start: u32,
    data_area_start: u32,
}

impl FileSystem {
    /// 创建一个新的文件系统
    ///
    /// 在块设备上创建一个新的文件系统，划分出超级块 [SuperBlock]、inode 位图、inode 区域、数据块位图、数据区域
    ///
    /// 并将第一个 inode 分配给根目录
    ///
    /// ## 参数
    /// - `dev`：块设备
    /// - `total_blocks`：总块数
    /// - `inode_bitmap_blocks`：inode 位图的块数
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
        crate::block_cache::block_cache_sync_all();
        Arc::new(Mutex::new(fs))
    }

    /// 获取 inode 的磁盘位置
    ///
    /// 根据 inode 编号计算 inode 所在的块号和偏移量，一个磁盘块可以存放多个 inode。
    ///
    /// ## 参数
    /// - `inode_id`：inode 编号
    ///
    /// ## 返回
    /// inode 所在的块号和偏移量
    pub fn get_disk_inode_pos(&self, inode_id: u32) -> (u32, usize) {
        let inode_size = core::mem::size_of::<DiskInode>();
        let inode_per_block = (BLOCK_SIZE / inode_size) as u32;
        let inode_bid = self.inode_area_start + inode_id / inode_per_block;
        let inode_offset = (inode_id % inode_per_block) as usize * inode_size;
        (inode_bid, inode_offset)
    }

    /// 获取数据块的编号
    ///
    /// 根据数据块的偏移量计算数据块的编号
    ///
    /// ## 参数
    /// - `offset`：数据块的偏移量
    ///
    /// ## 返回
    /// 数据块的编号
    pub fn get_data_block_id(&self, offset: u32) -> u32 {
        self.data_area_start + offset
    }

    /// 分配一个 inode
    ///
    /// 在 inode 位图中分配一个 inode
    ///
    /// ## 返回
    /// inode 编号
    pub fn alloc_inode(&self) -> u32 {
        self.inode_bitmap
            .alloc(&self.dev)
            .expect("alloc inode failed") as u32
    }

    /// 分配一个数据块
    ///
    /// 在数据块位图中分配一个数据块
    ///
    /// ## 返回
    /// 数据块编号
    pub fn alloc_data_block(&self) -> u32 {
        self.data_bitmap
            .alloc(&self.dev)
            .expect("alloc data block failed") as u32
            + self.data_area_start
    }

    /// 释放一个 inode
    ///
    /// 从 inode 位图中释放一个 inode
    ///
    /// ## 参数
    /// - `inode_id`：inode 编号
    pub fn free_inode(&self, inode_id: u32) {
        self.inode_bitmap.free(&self.dev, inode_id as usize);
    }

    /// 释放一个数据块
    ///
    /// 从数据块位图中释放一个数据块，并清空数据块内容
    ///
    /// ## 参数
    /// - `data_block_id`：数据块编号
    pub fn free_data_block(&self, data_block_id: u32) {
        let cache = get_cache(data_block_id as usize, self.dev.clone()).expect("get cache failed");
        cache.lock().modify_at(0, |data_block: &mut DataBlock| {
            data_block.iter_mut().for_each(|x| *x = 0);
        });
        self.data_bitmap
            .free(&self.dev, (data_block_id - self.data_area_start) as usize);
    }

    /// 打开一个已有的文件系统
    ///
    /// 从块设备中打开一个已有的文件系统，读取超级块，inode 位图，数据块位图等信息
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
    /// 获取根目录的 inode
    ///
    /// ## 返回
    /// 根目录的 inode
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
