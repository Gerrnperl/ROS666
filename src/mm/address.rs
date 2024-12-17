use core::ops::Add;

use super::page_table::PageTableEntry;

/// 页内偏移位数
pub const PAGE_OFFSET_WIDTH_SV39: usize = 12;
/// 页大小
pub const PAGE_SIZE_SV39: usize = 1 << PAGE_OFFSET_WIDTH_SV39;

/// 物理地址位数
pub const PHYSICAL_ADDRESS_WIDTH_SV39: usize = 56;
/// 物理页号位数
pub const PHYSICAL_PAGE_NUMBER_WIDTH_SV39: usize =
    PHYSICAL_ADDRESS_WIDTH_SV39 - PAGE_OFFSET_WIDTH_SV39;

/// 虚拟地址位数
pub const VIRTUAL_ADDRESS_WIDTH_SV39: usize = 39;
/// 虚拟页号位数
pub const VIRTUAL_PAGE_NUMBER_WIDTH_SV39: usize =
    VIRTUAL_ADDRESS_WIDTH_SV39 - PAGE_OFFSET_WIDTH_SV39;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysicalAddress(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtualAddress(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysicalPageNumber(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtualPageNumber(pub usize);

impl PhysicalPageNumber {
    pub fn get_mut<T>(&self) -> &'static mut T {
        let addr: PhysicalAddress = (*self).into();
        unsafe { &mut *(addr.0 as *mut T) }
    }

    pub fn get_ref<T>(&self) -> &'static T {
        let addr: PhysicalAddress = (*self).into();
        unsafe { &*(addr.0 as *const T) }
    }

    pub fn get_pte_array(&self) -> &'static mut [PageTableEntry] {
        let addr: PhysicalAddress = (*self).into();
        unsafe { core::slice::from_raw_parts_mut(addr.0 as *mut PageTableEntry, 512) }
    }

    pub fn get_bytes_array(&self) -> &'static mut [u8] {
        let addr: PhysicalAddress = (*self).into();
        unsafe { core::slice::from_raw_parts_mut(addr.0 as *mut u8, 4096) }
    }
}

impl VirtualPageNumber {
    /// 取出三级页索引
    pub fn indexes(&self) -> [usize; 3] {
        let vpn = self.0;
        [vpn >> 18 & 0x1ff, vpn >> 9 & 0x1ff, vpn & 0x1ff]
    }
}

impl From<usize> for PhysicalAddress {
    fn from(address: usize) -> Self {
        // 只保留低 56 位
        Self(address & ((1 << PHYSICAL_ADDRESS_WIDTH_SV39) - 1))
    }
}

impl From<usize> for VirtualAddress {
    fn from(address: usize) -> Self {
        // 只保留低 39 位
        Self(address & ((1 << VIRTUAL_ADDRESS_WIDTH_SV39) - 1))
    }
}

impl From<usize> for PhysicalPageNumber {
    fn from(address: usize) -> Self {
        // 只保留低 44 位
        Self(address & ((1 << PHYSICAL_PAGE_NUMBER_WIDTH_SV39) - 1))
    }
}

impl From<usize> for VirtualPageNumber {
    fn from(address: usize) -> Self {
        // 只保留低 27 位
        Self(address & ((1 << VIRTUAL_PAGE_NUMBER_WIDTH_SV39) - 1))
    }
}

impl From<PhysicalAddress> for usize {
    fn from(address: PhysicalAddress) -> Self {
        address.0
    }
}

impl From<VirtualAddress> for usize {
    fn from(address: VirtualAddress) -> Self {
        address.0
    }
}

impl From<PhysicalPageNumber> for usize {
    fn from(address: PhysicalPageNumber) -> Self {
        address.0
    }
}

impl From<VirtualPageNumber> for usize {
    fn from(address: VirtualPageNumber) -> Self {
        address.0
    }
}

impl Add for VirtualPageNumber {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        VirtualPageNumber(self.0 + rhs.0)
    }
}

impl PhysicalAddress {
    pub fn page_offset(&self) -> usize {
        self.0 & (PAGE_SIZE_SV39 - 1)
    }

    pub fn floor_page(&self) -> PhysicalAddress {
        PhysicalAddress(self.0 & !(PAGE_SIZE_SV39 - 1))
    }

    pub fn ceil_page(&self) -> PhysicalAddress {
        PhysicalAddress(self.0 + PAGE_SIZE_SV39 - 1 & !(PAGE_SIZE_SV39 - 1))
    }
}

impl From<PhysicalAddress> for PhysicalPageNumber {
    fn from(address: PhysicalAddress) -> Self {
        PhysicalPageNumber(address.0 >> PAGE_OFFSET_WIDTH_SV39)
    }
}

impl From<PhysicalPageNumber> for PhysicalAddress {
    fn from(page_number: PhysicalPageNumber) -> Self {
        PhysicalAddress(page_number.0 << PAGE_OFFSET_WIDTH_SV39)
    }
}

impl VirtualAddress {
    pub fn page_offset(&self) -> usize {
        self.0 & (PAGE_SIZE_SV39 - 1)
    }

    pub fn floor(&self) -> VirtualAddress {
        VirtualAddress(self.0 & !(PAGE_SIZE_SV39 - 1))
    }

    pub fn ceil(&self) -> VirtualAddress {
        VirtualAddress(self.0 + PAGE_SIZE_SV39 - 1 & !(PAGE_SIZE_SV39 - 1))
    }

    pub fn floor_page(&self) -> VirtualPageNumber {
        VirtualPageNumber::from(self.floor())
    }

    pub fn ceil_page(&self) -> VirtualPageNumber {
        VirtualPageNumber::from(self.ceil())
    }
}

impl From<VirtualAddress> for VirtualPageNumber {
    fn from(address: VirtualAddress) -> Self {
        VirtualPageNumber(address.0 >> PAGE_OFFSET_WIDTH_SV39)
    }
}

impl From<VirtualPageNumber> for VirtualAddress {
    fn from(page_number: VirtualPageNumber) -> Self {
        VirtualAddress(page_number.0 << PAGE_OFFSET_WIDTH_SV39)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VPNRange {
    pub start: VirtualPageNumber,
    pub length: usize,
    current: VirtualPageNumber,
}

impl Iterator for VPNRange {
    type Item = VirtualPageNumber;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.0 < self.start.0 + self.length {
            let current = self.current;
            self.current.0 += 1;
            Some(current)
        } else {
            None
        }
    }
}

impl VPNRange {
    pub fn new(start: VirtualPageNumber, end: VirtualPageNumber) -> Self {
        let length = end.0 - start.0;
        Self {
            start,
            length,
            current: start,
        }
    }
    pub fn from_addr(start: VirtualAddress, end: VirtualAddress) -> Self {
        Self::new(start.into(), end.into())
    }
    pub fn end(&self) -> VirtualPageNumber {
        VirtualPageNumber(self.start.0 + self.length)
    }
}
