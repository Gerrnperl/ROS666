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
/// 物理地址结构体，包含一个 `usize` 类型的地址值。
///
/// # 示例
/// ```
/// let addr = PhysicalAddress(0x1000);
/// ```
pub struct PhysicalAddress(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// 虚拟地址结构体，包含一个 `usize` 类型的地址值。
///
/// # 示例
/// ```
/// let addr = VirtualAddress(0x1000);
/// ```
pub struct VirtualAddress(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// 物理页号结构体，包含一个 `usize` 类型的页号值。
///
/// # 示例
/// ```
/// let ppn = PhysicalPageNumber(0x100);
/// ```
pub struct PhysicalPageNumber(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// 虚拟页号结构体，包含一个 `usize` 类型的页号值。
///
/// # 示例
/// ```
/// let vpn = VirtualPageNumber(0x100);
/// ```
pub struct VirtualPageNumber(pub usize);

impl PhysicalPageNumber {
    /// 获取一个可变引用
    pub fn get_mut<T>(&self) -> &'static mut T {
        let addr: PhysicalAddress = (*self).into();
        unsafe { &mut *(addr.0 as *mut T) }
    }

    /// 获取一个不可变引用
    pub fn get_ref<T>(&self) -> &'static T {
        let addr: PhysicalAddress = (*self).into();
        unsafe { &*(addr.0 as *const T) }
    }

    /// 获取页表项数组的可变引用
    pub fn get_pte_array(&self) -> &'static mut [PageTableEntry] {
        let addr: PhysicalAddress = (*self).into();
        unsafe { core::slice::from_raw_parts_mut(addr.0 as *mut PageTableEntry, 512) }
    }

    /// 获取字节数组的可变引用
    pub fn get_bytes_array(&self) -> &'static mut [u8] {
        let addr: PhysicalAddress = (*self).into();
        unsafe { core::slice::from_raw_parts_mut(addr.0 as *mut u8, 4096) }
    }
}

impl VirtualPageNumber {
    /// 取出三级页索引
    ///
    /// # 返回值
    /// 返回一个包含三级页索引的数组。
    /// 数组中的每个元素分别对应三级页索引中的一个。
    pub fn indexes(&self) -> [usize; 3] {
        let vpn = self.0;
        [vpn >> 18 & 0x1ff, vpn >> 9 & 0x1ff, vpn & 0x1ff]
    }
}

/// 将 `usize` 转换为 `PhysicalAddress` 类型。
///
/// # 参数
/// * `address` - 一个 `usize` 类型的值。
/// # 返回值
/// 返回 `address` 的 `PhysicalAddress` 表示。
impl From<usize> for PhysicalAddress {
    fn from(address: usize) -> Self {
        // 只保留低 56 位
        Self(address & ((1 << PHYSICAL_ADDRESS_WIDTH_SV39) - 1))
    }
}

/// 将 `usize` 转换为 `VirtualAddress` 类型。
///
/// # 参数
/// * `address` - 一个 `usize` 类型的值。
/// # 返回值
/// 返回 `address` 的 `VirtualAddress` 表示。
impl From<usize> for VirtualAddress {
    fn from(address: usize) -> Self {
        // 只保留低 39 位
        Self(address & ((1 << VIRTUAL_ADDRESS_WIDTH_SV39) - 1))
    }
}

/// 将 `usize` 转换为 `PhysicalPageNumber` 类型。
///
/// # 参数
/// * `address` - 一个 `usize` 类型的值。
/// # 返回值
/// 返回 `address` 的 `PhysicalPageNumber` 表示。
impl From<usize> for PhysicalPageNumber {
    fn from(address: usize) -> Self {
        // 只保留低 44 位
        Self(address & ((1 << PHYSICAL_PAGE_NUMBER_WIDTH_SV39) - 1))
    }
}

/// 将 `usize` 转换为 `VirtualPageNumber` 类型。
///
/// # 参数
/// * `address` - 一个 `usize` 类型的值。
/// # 返回值
/// 返回 `address` 的 `VirtualPageNumber` 表示。
impl From<usize> for VirtualPageNumber {
    fn from(address: usize) -> Self {
        // 只保留低 27 位
        Self(address & ((1 << VIRTUAL_PAGE_NUMBER_WIDTH_SV39) - 1))
    }
}

/// 将 `PhysicalAddress` 转换为 `usize` 类型。
///
/// # 参数
/// * `address` - 一个 `PhysicalAddress` 类型的值。
/// # 返回值
/// 返回 `address` 的内部 `usize` 表示。
impl From<PhysicalAddress> for usize {
    fn from(address: PhysicalAddress) -> Self {
        address.0
    }
}

/// 将 `VirtualAddress` 转换为 `usize` 类型。
///
/// # 参数
/// * `address` - 一个 `VirtualAddress` 类型的值。
/// # 返回值
/// 返回 `address` 的内部 `usize` 表示。
impl From<VirtualAddress> for usize {
    fn from(address: VirtualAddress) -> Self {
        address.0
    }
}

/// 将 `PhysicalPageNumber` 转换为 `usize` 类型。
///
/// # 参数
/// * `address` - 要转换的 `PhysicalPageNumber` 实例。
/// # 返回值
/// 返回 `PhysicalPageNumber` 的内部值，类型为 `usize`。
impl From<PhysicalPageNumber> for usize {
    fn from(address: PhysicalPageNumber) -> Self {
        address.0
    }
}

/// 将 `VirtualPageNumber` 转换为 `usize` 类型。
///
/// # 参数
/// * `address` - 要转换的 `VirtualPageNumber` 实例。
/// # 返回值
/// 返回 `VirtualPageNumber` 的内部值，类型为 `usize`。
impl From<VirtualPageNumber> for usize {
    fn from(address: VirtualPageNumber) -> Self {
        address.0
    }
}

/// 为 `VirtualPageNumber` 实现加法运算。
///
/// # 参数
/// * `rhs` - 右操作数，类型为 `VirtualPageNumber`。
/// # 返回值
/// 返回两个 `VirtualPageNumber` 相加的结果。
impl Add for VirtualPageNumber {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        VirtualPageNumber(self.0 + rhs.0)
    }
}

impl PhysicalAddress {
    /// 计算页内偏移量，通过对地址进行按位与操作
    pub fn page_offset(&self) -> usize {
        // PAGE_SIZE_SV39 - 1 用于获取页大小减一的值，这样可以确保只保留页内偏移部分
        self.0 & (PAGE_SIZE_SV39 - 1)
    }

    /// 计算对齐到页边界的物理地址，通过对地址进行按位与操作
    pub fn floor_page(&self) -> PhysicalAddress {
        // !(PAGE_SIZE_SV39 - 1) 用于获取页大小减一的补码，这样可以清除页内偏移部分
        PhysicalAddress(self.0 & !(PAGE_SIZE_SV39 - 1))
    }

    /// 计算向上对齐到页边界的物理地址
    pub fn ceil_page(&self) -> PhysicalAddress {
        // self.0 + PAGE_SIZE_SV39 - 1 用于确保地址向上对齐到下一个页边界
        // !(PAGE_SIZE_SV39 - 1) 用于获取页大小减一的补码，这样可以清除页内偏移部分
        PhysicalAddress((self.0 + PAGE_SIZE_SV39 - 1) & !(PAGE_SIZE_SV39 - 1))
    }

    /// 计算页号
    pub fn get_mut<T>(&self) -> &'static mut T {
        // 使用 unsafe 块将地址转换为指向类型 T 的可变指针，并返回可变引用
        unsafe { &mut *(self.0 as *mut T) }
    }
}

/// 将 `PhysicalAddress` 转换为 `PhysicalPageNumber` 类型。
///
/// # 参数
/// * `address` - 一个 `PhysicalAddress` 类型的值。
/// # 返回值
/// 返回 `address` 的 `PhysicalPageNumber` 表示。
impl From<PhysicalAddress> for PhysicalPageNumber {
    fn from(address: PhysicalAddress) -> Self {
        PhysicalPageNumber(address.0 >> PAGE_OFFSET_WIDTH_SV39)
    }
}

/// 将 `PhysicalPageNumber` 转换为 `PhysicalAddress` 类型。
///
/// # 参数
/// * `page_number` - 一个 `PhysicalPageNumber` 类型的值。
/// # 返回值
/// 返回 `page_number` 的 `PhysicalAddress` 表示。
impl From<PhysicalPageNumber> for PhysicalAddress {
    fn from(page_number: PhysicalPageNumber) -> Self {
        PhysicalAddress(page_number.0 << PAGE_OFFSET_WIDTH_SV39)
    }
}

impl VirtualAddress {
    /// 计算页内偏移量，通过对地址进行按位与操作
    pub fn page_offset(&self) -> usize {
        // PAGE_SIZE_SV39 - 1 用于获取页大小减一的值，这样可以确保只保留页内偏移部分
        self.0 & (PAGE_SIZE_SV39 - 1)
    }

    /// 计算对齐到页边界的虚拟地址，通过对地址进行按位与操作
    pub fn floor(&self) -> VirtualAddress {
        // !(PAGE_SIZE_SV39 - 1) 用于获取页大小减一的补码，这样可以清除页内偏移部分
        VirtualAddress(self.0 & !(PAGE_SIZE_SV39 - 1))
    }

    /// 计算向上对齐到页边界的虚拟地址
    pub fn ceil(&self) -> VirtualAddress {
        // self.0 + PAGE_SIZE_SV39 - 1 用于确保地址向上对齐到下一个页边界
        // !(PAGE_SIZE_SV39 - 1) 用于获取页大小减一的补码，这样可以清除页内偏移部分
        VirtualAddress((self.0 + PAGE_SIZE_SV39 - 1) & !(PAGE_SIZE_SV39 - 1))
    }

    /// 计算对齐到页边界的虚拟页号
    pub fn floor_page(&self) -> VirtualPageNumber {
        // 调用 floor() 方法获取对齐到页边界的虚拟地址
        // 然后将其转换为 VirtualPageNumber
        VirtualPageNumber::from(self.floor())
    }

    /// 计算向上对齐到页边界的虚拟页号
    pub fn ceil_page(&self) -> VirtualPageNumber {
        // 调用 ceil() 方法获取向上对齐到页边界的虚拟地址
        // 然后将其转换为 VirtualPageNumber
        VirtualPageNumber::from(self.ceil())
    }
}

/// 将 `VirtualAddress` 转换为 `VirtualPageNumber` 类型。
///
/// # 参数
/// * `address` - 一个 `VirtualAddress` 类型的值。
/// # 返回值
/// 返回 `address` 的 `VirtualPageNumber` 表示。
impl From<VirtualAddress> for VirtualPageNumber {
    fn from(address: VirtualAddress) -> Self {
        VirtualPageNumber(address.0 >> PAGE_OFFSET_WIDTH_SV39)
    }
}

/// 将 `VirtualPageNumber` 转换为 `VirtualAddress`。
///
/// # 参数
/// * `page_number` - 虚拟页号。
/// # 返回值
/// 返回一个 `VirtualAddress`，其值为 `page_number` 左移 `PAGE_OFFSET_WIDTH_SV39` 位。
impl From<VirtualPageNumber> for VirtualAddress {
    fn from(page_number: VirtualPageNumber) -> Self {
        VirtualAddress(page_number.0 << PAGE_OFFSET_WIDTH_SV39)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// 表示虚拟页号范围的结构体。
///
/// # 字段
/// * `start` - 范围的起始虚拟页号。
/// * `length` - 范围的长度，以页为单位。
/// * `current` - 当前虚拟页号，用于内部跟踪。
pub struct VPNRange {
    pub start: VirtualPageNumber,
    pub length: usize,
    current: VirtualPageNumber,
}

/// 实现 `Iterator` trait 用于 `VPNRange`。
///
/// `Item` 类型为 `VirtualPageNumber`。
/// # 方法
/// * `next` - 返回下一个 `VirtualPageNumber`，如果范围内没有更多的页码则返回 `None`。
/// # 示例
/// ```rust
/// let mut range = VPNRange { start: ..., length: ..., current: ... };
/// while let Some(vpn) = range.next() {
///     // 处理 `vpn`
/// }
/// ```
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
    /// 创建一个新的 VPNRange 实例
    ///
    /// # 参数
    /// * `start` - 起始虚拟页号
    /// * `end` - 结束虚拟页号
    pub fn new(start: VirtualPageNumber, end: VirtualPageNumber) -> Self {
        // 计算范围内的页数
        let length = end.0 - start.0;
        Self {
            start,
            length,
            current: start,
        }
    }

    /// 从虚拟地址范围创建一个新的 VPNRange 实例
    ///
    /// # 参数
    /// * `start` - 起始虚拟地址
    /// * `end` - 结束虚拟地址
    pub fn from_addr(start: VirtualAddress, end: VirtualAddress) -> Self {
        // 将虚拟地址转换为虚拟页号，并调用 new 方法
        Self::new(start.into(), end.into())
    }

    /// 获取范围的结束虚拟页号
    ///
    /// # 返回值
    /// 返回结束虚拟页号
    pub fn end(&self) -> VirtualPageNumber {
        // 计算结束虚拟页号
        VirtualPageNumber(self.start.0 + self.length)
    }
}
