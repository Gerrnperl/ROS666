use super::{KERNEL_SPACE, frame_allocator, heap_allocator};

pub fn init() {
    heap_allocator::init_heap();
    frame_allocator::StackFrameAllocator::init_frame_allocator();
    KERNEL_SPACE.ref_cell.borrow_mut().activate();
}
