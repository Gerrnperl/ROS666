use alloc::sync::Arc;
use spin::Mutex;

use crate::{
    block_cache::get_cache,
    block_dev::BlockDevice,
    fs::FileSystem,
    layout::{
        dir_entry::{DIR_ENTRY_SIZE, DirEntry},
        disk_inode::{self, DiskInode, InodeType},
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
        todo!()
    }
}
