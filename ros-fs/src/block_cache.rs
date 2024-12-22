use crate::block_dev::BlockDevice;
use alloc::{collections::vec_deque::VecDeque, sync::Arc};
use lazy_static::lazy_static;
use spin::Mutex;

pub const BLOCK_SIZE: usize = 512;
pub const BLOCK_CACHE_SIZE: usize = 16;

lazy_static! {
    /// 块缓存管理器
    pub static ref BLOCK_CACHE_MANAGER: Mutex<BlockCacheManager> =
        Mutex::new(BlockCacheManager::new());
}

/// 块缓存管理器
///
/// 用于管理块缓存，以先进先出的方式管理块缓存
pub struct BlockCacheManager {
    queue: VecDeque<(usize, Arc<Mutex<BlockCache>>)>,
}

/// 块缓存
///
/// 用于缓存块设备中的块，减少对块设备的读写次数。
///
/// 当块缓存被销毁时，会将缓存中的数据写回到块设备中。
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

    /// 获取块缓存
    ///
    /// - 如果缓存中存在指定 ID 的块缓存，则返回该块缓存的引用。
    /// - 如果缓存中不存在指定 ID 的块缓存，则创建一个新的块缓存，并将其加入缓存队列。
    /// - 如果缓存队列已满，则会将引用计数为 1 的块缓存移除。
    ///
    /// ## 参数
    /// - `id`：块 ID
    /// - `dev`：块设备
    ///
    /// ## 返回
    /// 返回块缓存的引用，如果创建失败则返回 None
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

/// 获取块缓存
///
/// ## 参数
/// - `id`：块 ID
/// - `dev`：块设备
///
/// ## 返回
/// 返回块缓存的引用，如果创建失败则返回 None
pub fn get_cache(id: usize, dev: Arc<dyn BlockDevice>) -> Option<Arc<Mutex<BlockCache>>> {
    let mut manager = BLOCK_CACHE_MANAGER.lock();
    manager.get_cache(id, dev)
}

impl BlockCache {
    /// 创建一个新的块缓存
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

    /// 获取指定偏移处的数据
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

    /// 获取指定偏移处的数据的引用
    pub fn get_ref_at<T>(&self, offset: usize) -> Option<&T>
    where
        T: Sized,
    {
        let addr = self.get_at::<T>(offset)?;
        Some(unsafe { &*(addr as *const T) })
    }

    /// 获取指定偏移处的数据的可变引用
    pub fn get_mut_at<T>(&mut self, offset: usize) -> Option<&mut T>
    where
        T: Sized,
    {
        let addr = self.get_at::<T>(offset)?;
        self.modified = true;
        Some(unsafe { &mut *(addr as *mut T) })
    }

    /// 读取指定偏移处的数据
    ///
    /// 在读取数据时，可以通过适配器对数据进行处理
    ///
    /// ## 参数
    /// - `offset`：偏移
    /// - `adapter`：适配器, 用于在读取数据时进行处理
    ///
    /// ## 返回
    /// 返回 适配器 的返回值，如果读取失败则返回 None
    pub fn read_at<T, R>(&self, offset: usize, adapter: impl FnOnce(&T) -> R) -> Option<R>
    where
        T: Sized,
    {
        let t = self.get_ref_at::<T>(offset)?;
        Some(adapter(t))
    }

    /// 修改指定偏移处的数据
    ///
    /// 在修改数据时，可以通过适配器对数据进行处理
    ///
    /// ## 参数
    /// - `offset`：偏移
    /// - `adapter`：适配器, 用于在修改数据时进行处理
    ///
    /// ## 返回
    /// 返回 适配器 的返回值，如果修改失败则返回 None
    pub fn modify_at<T, R>(&mut self, offset: usize, adapter: impl FnOnce(&mut T) -> R) -> Option<R>
    where
        T: Sized,
    {
        let t = self.get_mut_at::<T>(offset)?;
        Some(adapter(t))
    }

    /// 将缓存中的数据写回到块设备中
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

/// 同步所有块缓存
pub fn block_cache_sync_all() {
    let manager = BLOCK_CACHE_MANAGER.lock();
    for (_, cache) in manager.queue.iter() {
        cache.lock().sync();
    }
}
