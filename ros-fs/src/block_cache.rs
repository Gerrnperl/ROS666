use crate::block_dev::BlockDevice;
use alloc::{collections::vec_deque::VecDeque, sync::Arc};
use lazy_static::lazy_static;
use spin::Mutex;

pub const BLOCK_SIZE: usize = 512;
pub const BLOCK_CACHE_SIZE: usize = 16;

lazy_static! {
    pub static ref BLOCK_CACHE_MANAGER: Mutex<BlockCacheManager> =
        Mutex::new(BlockCacheManager::new());
}

pub struct BlockCacheManager {
    queue: VecDeque<(usize, Arc<Mutex<BlockCache>>)>,
}

pub struct BlockCache {
    cache: [u8; BLOCK_SIZE],
    id: usize,
    dev: Arc<dyn BlockDevice>,
    modified: bool,
}

impl BlockCacheManager {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    pub fn get_cache(
        &mut self,
        id: usize,
        dev: Arc<dyn BlockDevice>,
    ) -> Option<Arc<Mutex<BlockCache>>> {
        for (cache_id, cache) in self.queue.iter() {
            if *cache_id == id {
                return Some(cache.clone());
            }
        }
        if self.queue.len() >= BLOCK_CACHE_SIZE {
            let found = self
                .queue
                .iter()
                .enumerate()
                .find(|(_, (_, cache))| Arc::strong_count(cache) == 1);
            if let Some((idx, _)) = found {
                self.queue.remove(idx);
            } else {
                return None;
            }
        }
        let cache = Arc::new(Mutex::new(BlockCache::new(id, dev.clone())));
        self.queue.push_back((id, cache.clone()));
        Some(cache)
    }
}

pub fn get_cache(id: usize, dev: Arc<dyn BlockDevice>) -> Option<Arc<Mutex<BlockCache>>> {
    let mut manager = BLOCK_CACHE_MANAGER.lock();
    manager.get_cache(id, dev)
}

impl BlockCache {
    pub fn new(id: usize, dev: Arc<dyn BlockDevice>) -> Self {
        let mut cache = [0; BLOCK_SIZE];
        dev.read_block(id, &mut cache);
        Self {
            cache,
            id,
            dev,
            modified: false,
        }
    }

    fn get_addr_at(&self, offset: usize) -> usize {
        &self.cache[offset] as *const _ as usize
    }

    fn get_at<T>(&self, offset: usize) -> Option<usize>
    where
        T: Sized,
    {
        let t_size = core::mem::size_of::<T>();
        if offset + t_size > BLOCK_SIZE {
            return None;
        }
        Some(self.get_addr_at(offset))
    }

    pub fn get_ref_at<T>(&self, offset: usize) -> Option<&T>
    where
        T: Sized,
    {
        let addr = self.get_at::<T>(offset)?;
        Some(unsafe { &*(addr as *const T) })
    }

    pub fn get_mut_at<T>(&mut self, offset: usize) -> Option<&mut T>
    where
        T: Sized,
    {
        let addr = self.get_at::<T>(offset)?;
        self.modified = true;
        Some(unsafe { &mut *(addr as *mut T) })
    }

    pub fn read_at<T, R>(&self, offset: usize, adapter: impl FnOnce(&T) -> R) -> Option<R>
    where
        T: Sized,
    {
        let t = self.get_ref_at::<T>(offset)?;
        Some(adapter(t))
    }

    pub fn modify_at<T, R>(&mut self, offset: usize, adapter: impl FnOnce(&mut T) -> R) -> Option<R>
    where
        T: Sized,
    {
        let t = self.get_mut_at::<T>(offset)?;
        Some(adapter(t))
    }

    pub fn sync(&mut self) {
        if self.modified {
            self.dev.write_block(self.id, &self.cache);
            self.modified = false;
        }
    }
}

impl Drop for BlockCache {
    fn drop(&mut self) {
        self.sync();
    }
}
