use crate::mm::{
    KERNEL_SPACE,
    address::{PAGE_SIZE_SV39, VirtualAddress, VirtualPageNumber},
    memory_set::{MapPermission, TRAMPOLINE},
};

use super::pid::PidHandler;

pub const KERNEL_STACK_SIZE: usize = 4096 * 2;
pub const USER_STACK_SIZE: usize = 4096 * 2;

pub fn kernel_stack_position(pid: usize) -> (usize, usize) {
    let top = TRAMPOLINE - pid * (KERNEL_STACK_SIZE + PAGE_SIZE_SV39);
    let bottom = top - KERNEL_STACK_SIZE;
    (bottom, top)
}

pub struct KernelStack {
    pid: usize,
}

impl KernelStack {
    pub fn new(pid: &PidHandler) -> Self {
        let (bottom, top) = kernel_stack_position(pid.0);
        KERNEL_SPACE.ref_cell.borrow_mut().insert(
            VirtualAddress::from(bottom)..VirtualAddress::from(top),
            MapPermission::Read | MapPermission::Write,
        );
        Self { pid: pid.0 }
    }

    pub fn top(&self) -> usize {
        let (_bottom, top) = kernel_stack_position(self.pid);
        top
    }

    /// 压入一个 `Sized` 值
    ///
    /// 返回栈中值的可变引用
    pub fn push<T>(&self, data: T) -> *mut T
    where
        T: Sized,
    {
        let top = self.top();
        let size = core::mem::size_of::<T>();
        // let align = core::mem::align_of::<T>();
        let new_top = top - size;
        // let new_top = new_top & !(align - 1);
        let ptr = new_top as *mut T;
        unsafe {
            *ptr = data;
        }
        ptr
    }
}

impl Drop for KernelStack {
    fn drop(&mut self) {
        let (bottom, top) = kernel_stack_position(self.pid);
        KERNEL_SPACE
            .ref_cell
            .borrow_mut()
            .remove_area(VirtualPageNumber::from(VirtualAddress::from(bottom)));
    }
}
