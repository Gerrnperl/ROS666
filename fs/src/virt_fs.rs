use alloc::sync::Arc;
use spin::Mutex;

use crate::{
    block_cache::get_cache, block_dev::BlockDevice, fs::FileSystem, layout::disk_inode::DiskInode,
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
}
