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

pub struct MemInode {
    block_id: usize,
    block_offset: usize,
    fs: Arc<Mutex<FileSystem>>,
    dev: Arc<dyn BlockDevice>,
}

impl MemInode {
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

    fn read_disk_inode<R>(&self, adapter: impl FnOnce(&DiskInode) -> R) -> Option<R> {
        let cache = get_cache(self.block_id, Arc::clone(&self.dev)).expect("get cache failed");
        cache.lock().read_at(self.block_offset, adapter)
    }

    fn modify_disk_inode<R>(&self, adapter: impl FnOnce(&mut DiskInode) -> R) -> Option<R> {
        let cache = get_cache(self.block_id, Arc::clone(&self.dev)).expect("get cache failed");
        cache.lock().modify_at(self.block_offset, adapter)
    }

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

    pub fn read_at(&self, offset: usize, buf: &mut [u8]) -> usize {
        let _fs = self.fs.lock();
        let read_size =
            self.read_disk_inode(|disk_inode| disk_inode.read_at(offset, &self.dev, buf));
        read_size.unwrap()
    }

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

    pub fn get_size(&self) -> u32 {
        let size = self
            .read_disk_inode(|disk_inode| disk_inode.size)
            .expect("read disk inode failed");
        size
    }
}
