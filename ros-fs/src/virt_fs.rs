//! 虚拟文件系统
//!
//! 建立在物理文件系统之上的虚拟文件系统，提供对物理文件系统的抽象。
use alloc::{
    string::{String, ToString},
    sync::Arc,
    vec::Vec,
};
use spin::{Mutex, MutexGuard};

use crate::{
    block_cache::get_cache,
    block_dev::BlockDevice,
    fs::FileSystem,
    layout::{
        dir_entry::{DIR_ENTRY_SIZE, DirEntry},
        disk_inode::{DiskInode, InodeType},
    },
};

/// 内存 inode
///
/// 用于表示虚拟文件系统中的一个 inode，提供了对 inode 的基本操作，存储在内存中，由物理文件系统提供支持。
pub struct MemInode {
    block_id: usize,
    block_offset: usize,
    fs: Arc<Mutex<FileSystem>>,
    dev: Arc<dyn BlockDevice>,
}

impl MemInode {
    /// 创建一个新的内存 inode
    ///
    /// ## 参数
    /// - `block_id`：块 ID
    /// - `block_offset`：块偏移
    /// - `fs`：物理文件系统
    /// - `dev`：块设备
    pub fn new(
        block_id: usize,
        block_offset: usize,
        fs: Arc<Mutex<FileSystem>>,
        dev: Arc<dyn BlockDevice>,
    ) -> Self {
        Self {
            block_id,
            block_offset,
            fs,
            dev,
        }
    }

    /// 读取 inode
    ///
    /// ## 参数
    /// - `adapter`：适配器, 用于在读取 inode 时进行处理
    ///
    /// ## 返回
    /// 返回 适配器 的返回值，如果读取失败则返回 None
    fn read_disk_inode<R>(&self, adapter: impl FnOnce(&DiskInode) -> R) -> Option<R> {
        let cache = get_cache(self.block_id, Arc::clone(&self.dev)).expect("get cache failed");
        cache.lock().read_at(self.block_offset, adapter)
    }

    /// 修改 inode
    ///
    /// ## 参数
    /// - `adapter`：适配器, 用于在修改 inode 时进行处理
    ///
    /// ## 返回
    /// 返回 适配器 的返回值，如果修改失败则返回 None
    fn modify_disk_inode<R>(&self, adapter: impl FnOnce(&mut DiskInode) -> R) -> Option<R> {
        let cache = get_cache(self.block_id, Arc::clone(&self.dev)).expect("get cache failed");
        cache.lock().modify_at(self.block_offset, adapter)
    }

    /// 根据文件名查找 inode
    ///
    /// 目前仅支持单级目录，查找时会遍历目录项，查找到对应的文件名则返回 inode
    ///
    /// ## 参数
    /// - `name`：文件名
    ///
    /// ## 返回
    /// 如果找到则返回 inode，否则返回 None
    pub fn find(&self, name: &str) -> Option<Arc<MemInode>> {
        let fs = self.fs.lock();
        let inode = self
            .read_disk_inode(|disk_inode| {
                self.find_inode_id(name, disk_inode).map(|inode_id| {
                    let (block_id, block_offset) = fs.get_disk_inode_pos(inode_id);
                    let inode = MemInode::new(
                        block_id as usize,
                        block_offset,
                        Arc::clone(&self.fs),
                        Arc::clone(&self.dev),
                    );
                    inode
                })
            })
            .expect("read disk inode failed");
        inode.map(|inode| Arc::new(inode))
    }

    /// 根据文件名在目录中查找 inode 编号
    ///
    /// ## 参数
    ///
    /// - `name`：文件名
    /// - `disk_inode`：目录的磁盘 inode
    ///
    /// ## 返回
    /// 如果找到则返回 inode 编号，否则返回 None
    pub fn find_inode_id(&self, name: &str, disk_inode: &DiskInode) -> Option<u32> {
        assert!(disk_inode.get_type() == InodeType::Dir);
        let sub_count = (disk_inode.size as usize) / DIR_ENTRY_SIZE;
        for sub_id in 0..sub_count {
            let sub_offset = sub_id * DIR_ENTRY_SIZE;
            let mut buf = DirEntry::default();
            disk_inode.read_at(sub_offset, &self.dev, buf.as_mut_bytes());
            if buf.name() == name {
                return Some(buf.inode());
            }
        }
        None
    }

    /// 列出目录下的所有文件
    pub fn ls(&self) -> Vec<String> {
        let mut res = Vec::new();
        let _fs = self.fs.lock();
        self.read_disk_inode(|disk_inode| {
            assert!(disk_inode.get_type() == InodeType::Dir);
            let sub_count = (disk_inode.size as usize) / DIR_ENTRY_SIZE;
            for sub_id in 0..sub_count {
                let sub_offset = sub_id * DIR_ENTRY_SIZE;
                let mut buf = DirEntry::default();
                disk_inode.read_at(sub_offset, &self.dev, buf.as_mut_bytes());
                res.push(buf.name().to_string());
            }
        })
        .expect("read disk inode failed");
        res
    }

    /// 创建一个新的 inode
    ///
    /// ## 参数
    /// - `name`：文件名
    /// - `inode_type`：inode 类型，目前仅支持文件
    pub fn create(&self, name: &str, inode_type: InodeType) -> Arc<MemInode> {
        let mut fs = self.fs.lock();
        let inode_id = fs.alloc_inode();
        let (block_id, block_offset) = fs.get_disk_inode_pos(inode_id);
        let inode = MemInode::new(
            block_id as usize,
            block_offset,
            Arc::clone(&self.fs),
            Arc::clone(&self.dev),
        );
        let _ = inode.modify_disk_inode(|disk_inode| {
            disk_inode.init(inode_type);
        });
        let _ = self.modify_disk_inode(|disk_inode| {
            let sub_count = (disk_inode.size as usize) / DIR_ENTRY_SIZE;
            let new_size = disk_inode.size + DIR_ENTRY_SIZE as u32;
            self.increase_size(new_size, disk_inode, &mut fs);
            disk_inode.write_at(
                sub_count * DIR_ENTRY_SIZE,
                &self.dev,
                DirEntry::new(inode_id, name).as_bytes(),
            );
        });
        crate::block_cache::block_cache_sync_all();
        Arc::new(inode)
    }

    /// 删除一个 inode，清除其数据块和 inode
    pub fn clear(&self) {
        let fs = self.fs.lock();
        self.modify_disk_inode(|disk_inode| {
            let _size = disk_inode.size;
            let dealloced = disk_inode.clear_size(&self.dev);
            dealloced.into_iter().for_each(|block_id| {
                fs.free_data_block(block_id);
            });
        })
        .expect("modify disk inode failed");
        crate::block_cache::block_cache_sync_all();
    }

    /// 增加 inode 的大小
    ///
    /// ## 参数
    /// - `new_size`：新的大小
    /// - `disk_inode`：磁盘 inode
    /// - `fs`：物理文件系统
    pub fn increase_size(
        &self,
        new_size: u32,
        disk_inode: &mut DiskInode,
        fs: &mut MutexGuard<FileSystem>,
    ) {
        if new_size <= disk_inode.size {
            return;
        }
        let required = disk_inode.required_delta_blocks_for(new_size);
        let new_blocks = (0..required)
            .map(|_| fs.alloc_data_block())
            .collect::<Vec<_>>();
        disk_inode.extend_size(new_size, new_blocks, &self.dev);
    }

    /// 读取 inode 数据
    ///
    /// ## 参数
    /// - `offset`：文件内偏移
    /// - `buf`：目标缓冲区
    ///
    /// ## 返回
    /// 返回读取的字节数
    pub fn read_at(&self, offset: usize, buf: &mut [u8]) -> usize {
        let _fs = self.fs.lock();
        let read_size =
            self.read_disk_inode(|disk_inode| disk_inode.read_at(offset, &self.dev, buf));
        read_size.unwrap()
    }

    /// 写入 inode 数据
    ///
    /// ## 参数
    /// - `offset`：文件内偏移
    /// - `buf`：源缓冲区
    ///
    /// ## 返回
    /// 返回写入的字节数
    pub fn write_at(&self, offset: usize, buf: &[u8]) -> usize {
        let mut fs = self.fs.lock();
        let write_size = self.modify_disk_inode(|disk_inode| {
            let new_size = offset as u32 + buf.len() as u32;
            self.increase_size(new_size, disk_inode, &mut fs);
            disk_inode.write_at(offset, &self.dev, buf)
        });
        crate::block_cache::block_cache_sync_all();
        write_size.unwrap()
    }

    /// 获取 inode 的大小
    pub fn get_size(&self) -> u32 {
        let size = self
            .read_disk_inode(|disk_inode| disk_inode.size)
            .expect("read disk inode failed");
        size
    }

    pub fn get_type(&self) -> InodeType {
        let inode_type = self
            .read_disk_inode(|disk_inode| disk_inode.get_type())
            .expect("read disk inode failed");
        inode_type
    }
}
