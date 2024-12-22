//! https://github.com/rcore-os/rCore-Tutorial-v3/blob/ch6/os/src/drivers/block/virtio_blk.rs
//! https://github.com/rcore-os/virtio-drivers/blob/master/examples/riscv/src/virtio_impl.rs
use core::ptr::NonNull;

use alloc::vec::Vec;
use lazy_static::lazy_static;
use ros_fs::block_dev::BlockDevice;
use spin::Mutex;
use virtio_drivers::{
    BufferDirection, Hal, PhysAddr,
    device::blk::VirtIOBlk,
    transport::mmio::{MmioTransport, VirtIOHeader},
};

use crate::mm::{
    KERNEL_SPACE,
    address::{PhysicalAddress, PhysicalPageNumber, VirtualAddress},
    frame_allocator::{FrameTracker, StackFrameAllocator},
    page_table::PageTable,
};

#[cfg(feature = "qemu")]
pub const MMIO: &[(usize, usize)] = &[(0x10001000, 0x1000)];

const VIRTIO_0: usize = 0x10001000;
pub struct VirtIOBlock(Mutex<VirtIOBlk<HalImpl, MmioTransport>>);

impl VirtIOBlock {
    pub unsafe fn new() -> Self {
        let vaddr = MMIO[0].0;
        let header = NonNull::new(vaddr as *mut VirtIOHeader).unwrap();
        let transport = unsafe { MmioTransport::new(header) }.unwrap();
        let virtio_blk = VirtIOBlk::new(transport).expect("failed to create VirtIOBlk");
        VirtIOBlock(Mutex::new(virtio_blk))
    }
}

unsafe impl Sync for VirtIOBlock {}
unsafe impl Send for VirtIOBlock {}

impl BlockDevice for VirtIOBlock {
    fn read_block(&self, block_id: usize, buf: &mut [u8]) {
        self.0
            .lock()
            .read_blocks(block_id, buf)
            .expect("Error when reading VirtIOBlk");
    }
    fn write_block(&self, block_id: usize, buf: &[u8]) {
        self.0
            .lock()
            .write_blocks(block_id, buf)
            .expect("Error when writing VirtIOBlk");
    }
}

lazy_static! {
    static ref QUEUE_FRAMES: Mutex<Vec<FrameTracker>> = Mutex::new(Vec::new());
}

pub struct HalImpl;

unsafe impl Hal for HalImpl {
    fn dma_alloc(pages: usize, _direction: BufferDirection) -> (PhysAddr, NonNull<u8>) {
        let mut ppn_base = PhysicalPageNumber(0);
        for i in 0..pages {
            let frame = StackFrameAllocator::alloc_frame().unwrap();
            if i == 0 {
                ppn_base = frame.frame;
            }
            assert_eq!(frame.frame.0, ppn_base.0 + i);
            QUEUE_FRAMES.lock().push(frame);
        }
        // ppn_base.into()
        let paddr = PhysicalAddress::from(ppn_base).0;

        let vaddr = NonNull::new(paddr as _).unwrap();
        (paddr, vaddr)
    }

    unsafe fn dma_dealloc(paddr: PhysAddr, _vaddr: NonNull<u8>, pages: usize) -> i32 {
        let pa = PhysicalAddress::from(paddr);
        let mut ppn_base = PhysicalPageNumber::from(pa);
        for _ in 0..pages {
            StackFrameAllocator::dealloc_frame(ppn_base);
            ppn_base.0 += 1;
        }
        0
    }

    unsafe fn mmio_phys_to_virt(paddr: PhysAddr, _size: usize) -> NonNull<u8> {
        let token = KERNEL_SPACE.inner_borrow_mut().token();
        let va = PageTable::from_satp(token)
            .translate_addr(VirtualAddress::from(paddr))
            .unwrap()
            .0;
        NonNull::new(va as *mut u8).unwrap()
    }

    unsafe fn share(buffer: NonNull<[u8]>, _direction: BufferDirection) -> PhysAddr {
        let token = KERNEL_SPACE.inner_borrow_mut().token();
        let va = PageTable::from_satp(token)
            .translate_addr(VirtualAddress::from(buffer.as_ptr() as *mut u8 as usize))
            .unwrap()
            .0;
        va
    }

    unsafe fn unshare(_paddr: PhysAddr, _buffer: NonNull<[u8]>, _direction: BufferDirection) {
        // Nothing to do, as the host already has access to all memory and we didn't copy the buffer
        // anywhere else.
    }
}
