use super::{KERNEL_SPACE, frame_allocator, heap_allocator};

pub static mut HEAP_INITED: bool = false;

pub fn init() {
    heap_allocator::init_heap();
    frame_allocator::StackFrameAllocator::init_frame_allocator();
    KERNEL_SPACE.ref_cell.borrow_mut().activate();
    unsafe { HEAP_INITED = true };
}
