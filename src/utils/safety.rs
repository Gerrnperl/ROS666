use core::cell::{RefCell, RefMut};

pub struct SyncRefCell<T> {
    pub ref_cell: RefCell<T>,
}

unsafe impl<T> Sync for SyncRefCell<T> {}

impl<T> SyncRefCell<T> {
    pub fn new(t: T) -> Self {
        Self {
            ref_cell: RefCell::new(t),
        }
    }
    pub fn inner_borrow_mut(&self) -> RefMut<'_, T> {
        self.ref_cell.borrow_mut()
    }
    pub fn inner_borrow(&self) -> core::cell::Ref<'_, T> {
        self.ref_cell.borrow()
    }
}
