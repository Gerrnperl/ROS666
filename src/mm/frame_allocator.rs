use super::address::{PhysicalAddress, PhysicalPageNumber};
use crate::{extern_global, printkln, utils::safety::SyncRefCell};
use alloc::vec::Vec;
use core::{cell::RefCell, ops::Range};
use lazy_static::lazy_static;

// debug: 0x84800000;
// release: 0x80800000;
// pub const MEMORY_END: usize = 0x80800000;
#[cfg(debug_assertions)]
pub const MEMORY_END: usize = 0x84800000;
#[cfg(not(debug_assertions))]
pub const MEMORY_END: usize = 0x80800000;

trait FrameAllocator {
    fn new() -> Self;
    fn alloc(&mut self) -> Option<PhysicalPageNumber>;
    fn dealloc(&mut self, frame: PhysicalPageNumber);
}

lazy_static! {
    pub static ref FRAME_ALLOCATOR: SyncRefCell<StackFrameAllocator> = {
        SyncRefCell {
            ref_cell: RefCell::new(StackFrameAllocator::new()),
        }
    };
}

pub struct StackFrameAllocator {
    unused: Range<PhysicalPageNumber>,
    freed: Vec<PhysicalPageNumber>,
}

impl FrameAllocator for StackFrameAllocator {
    fn new() -> Self {
        Self {
            unused: PhysicalPageNumber(0)..PhysicalPageNumber(0),
            freed: Vec::new(),
        }
    }

    fn alloc(&mut self) -> Option<PhysicalPageNumber> {
        if let Some(frame) = self.freed.pop() {
            return Some(frame);
        }
        if self.unused.start < self.unused.end {
            let frame = self.unused.start;
            self.unused.start.0 += 1;
            return Some(frame);
        }
        None
    }

    fn dealloc(&mut self, frame: PhysicalPageNumber) {
        if frame.0 >= self.unused.start.0 {
            panic!("trying to free free page: {:?}", frame);
        }
        if self.freed.iter().any(|&f| f == frame) {
            panic!("trying to free free page: {:?}", frame);
        }
        self.freed.push(frame);
    }
}

impl StackFrameAllocator {
    pub fn init(&mut self, range: Range<PhysicalPageNumber>) {
        self.unused = range;
    }

    pub fn init_frame_allocator() {
        let start = PhysicalAddress::from(extern_global!(__kernel_end) as usize)
            .ceil_page()
            .into();
        let end = PhysicalAddress::from(MEMORY_END).floor_page().into();
        FRAME_ALLOCATOR.ref_cell.borrow_mut().init(start..end);
    }

    pub fn alloc_frame() -> Option<FrameTracker> {
        FRAME_ALLOCATOR
            .ref_cell
            .borrow_mut()
            .alloc()
            .map(|frame| FrameTracker { frame })
    }

    pub fn dealloc_frame(frame: PhysicalPageNumber) {
        FRAME_ALLOCATOR.ref_cell.borrow_mut().dealloc(frame);
    }
}

#[derive(Debug)]
pub struct FrameTracker {
    pub frame: PhysicalPageNumber,
}

impl FrameTracker {
    pub fn new(frame: PhysicalPageNumber) -> Self {
        // clean
        frame.get_bytes_array().iter_mut().for_each(|x| *x = 0);
        Self { frame }
    }
}

impl Drop for FrameTracker {
    fn drop(&mut self) {
        StackFrameAllocator::dealloc_frame(self.frame);
    }
}

#[allow(unused)]
pub fn frame_allocator_test() {
    let mut v: Vec<FrameTracker> = Vec::new();
    for i in 0..5 {
        let frame = StackFrameAllocator::alloc_frame().unwrap();
        printkln!("{:?}", frame);
        v.push(frame);
    }
    v.clear();
    for i in 0..5 {
        let frame = StackFrameAllocator::alloc_frame().unwrap();
        printkln!("{:?}", frame);
        v.push(frame);
    }
    drop(v);
    printkln!("frame_allocator_test passed!");
}
