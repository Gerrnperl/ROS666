use super::address::{PHYSICAL_PAGE_NUMBER_WIDTH_SV39, PhysicalPageNumber};

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
