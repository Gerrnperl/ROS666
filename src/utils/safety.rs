use core::cell::{RefCell, RefMut};

pub struct SyncRefCell<T> {
    pub ref_cell: RefCell<T>,
}

unsafe impl<T> Sync for SyncRefCell<T> {}

impl<T> SyncRefCell<T> {
    // fn borrow_mut(&self) -> RefMut<'_, T> {
    //     self.ref_cell.borrow_mut()
    // }
}
