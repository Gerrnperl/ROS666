//! 安全性相关

use core::cell::{RefCell, RefMut};

/// 一个线程安全的 `RefCell` 包装器。
///
/// "I Promise I'm Safe RefCell :)"
pub struct SyncRefCell<T> {
    pub ref_cell: RefCell<T>,
}

unsafe impl<T> Sync for SyncRefCell<T> {}

impl<T> SyncRefCell<T> {
    /// 创建一个新的 `SyncRefCell` 实例。
    ///
    /// ## 参数
    /// * `t` - 要存储在 `SyncRefCell` 中的值。
    /// ## 返回值
    /// 返回一个包含给定值的 `SyncRefCell` 实例。
    pub fn new(t: T) -> Self {
        Self {
            ref_cell: RefCell::new(t),
        }
    }

    /// 获取对内部值的可变引用。
    ///
    /// ## 返回
    /// 返回一个可变引用，允许修改内部值。
    /// ## 安全性
    /// 如果已经有不可变引用存在，则此方法会导致运行时 `Panic`。
    pub fn inner_borrow_mut(&self) -> RefMut<'_, T> {
        self.ref_cell.borrow_mut()
    }

    /// 获取对内部值的不可变引用。
    ///
    /// ## 返回
    /// 返回一个不可变引用，允许读取内部值。
    /// ## 安全性
    /// 如果已经有可变引用存在，则此方法会导致运行时 `Panic`。
    pub fn inner_borrow(&self) -> core::cell::Ref<'_, T> {
        self.ref_cell.borrow()
    }
}
