use alloc::vec::Vec;

use super::{
    address::{PHYSICAL_PAGE_NUMBER_WIDTH_SV39, PhysicalPageNumber, VirtualPageNumber},
    frame_allocator::{FrameTracker, StackFrameAllocator},
};

pub struct PageTable {
    pub root: PhysicalPageNumber,
    pub frames: Vec<FrameTracker>,
}

impl PageTable {
    pub fn new() -> Self {
        let root = StackFrameAllocator::alloc_frame().unwrap();
        Self {
            root: root.frame,
            frames: Vec::new(),
        }
    }

    pub fn map(&mut self, vpn: VirtualPageNumber, ppn: PhysicalPageNumber, flags: PTEFlags) {
        let entry = self.find_pte(vpn, true).unwrap();
        if (*entry).valid() {
            panic!("map: {:?} has been mapped", vpn);
        }
        *entry = PageTableEntry::new(ppn, flags);
    }

    pub fn unmap(&mut self, vpn: VirtualPageNumber) {
        let entry = self.find_pte(vpn, false).unwrap();
        if !(*entry).valid() {
            panic!("unmap: {:?} has not been mapped", vpn);
        }
        *entry = PageTableEntry::default();
    }

    pub fn find_pte(
        &mut self,
        vpn: VirtualPageNumber,
        create: bool,
    ) -> Option<&mut PageTableEntry> {
        let indexes = vpn.indexes();
        let mut page_table = self.root.get_pte_array().get_mut(0).unwrap() as *mut PageTableEntry;
        for i in 0..3 {
            let entry = unsafe { &mut *page_table.add(indexes[i]) };
            if i == 2 {
                return Some(entry);
            }
            if !entry.valid() {
                if !create {
                    return None;
                }
                let frame = StackFrameAllocator::alloc_frame().unwrap();
                *entry = PageTableEntry::new(frame.frame, PTEFlags::Valid);
                self.frames.push(frame);
            }
            let next_ppn: PhysicalPageNumber = (&*entry).into();
            page_table = next_ppn.get_pte_array().get_mut(0).unwrap() as *mut PageTableEntry;
        }
        None
    }

    pub fn translate(&mut self, vpn: VirtualPageNumber) -> Option<PhysicalPageNumber> {
        self.find_pte(vpn, false).map(|entry| (&*entry).into())
    }

    pub fn from_satp(satp: usize) -> Self {
        let root_ppn = PhysicalPageNumber::from(satp);
        Self {
            root: root_ppn,
            frames: Vec::new(),
        }
    }
}

bitflags::bitflags! {
    #[derive(PartialEq)]
    pub struct PTEFlags: u8 {
        const Valid = 1 << 0;
        const Read = 1 << 1;
        const Write = 1 << 2;
        const Execute = 1 << 3;
        const User = 1 << 4;
        const Global = 1 << 5;
        const Accessed = 1 << 6;
        const Dirty = 1 << 7;
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
/// 页表项 PTE
///
/// 64 位 RISC-V 地址空间下，页表项大小为 64 位
///
/// ## 页表项格式
/// | 63-54 | 53-10 | 9-8 | 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |
/// |-------|-------|-----|---|---|---|---|---|---|---|---|
/// | 保留位 | 物理页号 | RSW | D | A | G | U | X | W | R | V |
pub struct PageTableEntry {
    pub bits: usize,
}

impl PageTableEntry {
    pub fn new(phy_page: PhysicalPageNumber, flags: PTEFlags) -> Self {
        Self {
            bits: (phy_page.0 << 10) | flags.bits() as usize,
        }
    }

    pub fn valid(&self) -> bool {
        (PTEFlags::Valid & self.into()) != PTEFlags::empty()
    }

    pub fn readable(&self) -> bool {
        (PTEFlags::Read & self.into()) != PTEFlags::empty()
    }

    pub fn writable(&self) -> bool {
        (PTEFlags::Write & self.into()) != PTEFlags::empty()
    }

    pub fn executable(&self) -> bool {
        (PTEFlags::Execute & self.into()) != PTEFlags::empty()
    }

    pub fn user(&self) -> bool {
        (PTEFlags::User & self.into()) != PTEFlags::empty()
    }

    pub fn accessed(&self) -> bool {
        (PTEFlags::Accessed & self.into()) != PTEFlags::empty()
    }

    pub fn dirty(&self) -> bool {
        (PTEFlags::Dirty & self.into()) != PTEFlags::empty()
    }
}

impl Default for PageTableEntry {
    fn default() -> Self {
        Self { bits: 0 }
    }
}

impl From<&PageTableEntry> for PTEFlags {
    fn from(pte: &PageTableEntry) -> Self {
        Self::from_bits_truncate(pte.bits as u8)
    }
}

impl From<&PageTableEntry> for PhysicalPageNumber {
    fn from(pte: &PageTableEntry) -> Self {
        PhysicalPageNumber(pte.bits >> 10 & ((1 << PHYSICAL_PAGE_NUMBER_WIDTH_SV39) - 1))
    }
}
