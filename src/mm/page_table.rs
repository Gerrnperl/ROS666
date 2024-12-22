//! 页表模块

use alloc::{string::String, vec::Vec};

use super::{
    address::{
        PHYSICAL_PAGE_NUMBER_WIDTH_SV39, PhysicalAddress, PhysicalPageNumber, VirtualAddress,
        VirtualPageNumber,
    },
    frame_allocator::{FrameTracker, StackFrameAllocator},
};

/// 页表
///
/// 包含根页表的物理页号和帧跟踪器的向量
pub struct PageTable {
    /// 根页表的物理页号
    pub root: PhysicalPageNumber,
    /// 帧跟踪器的向量
    pub frames: Vec<FrameTracker>,
}

impl PageTable {
    /// 创建一个新的页表
    ///
    /// 分配一个根页表帧，并初始化页表结构体
    pub fn new() -> Self {
        let root = StackFrameAllocator::alloc_frame().unwrap();
        Self {
            root: root.frame,
            frames: alloc::vec![root],
        }
    }

    /// 获取页表的 token
    ///
    /// token 即 satp (satp: Supervisor Address Translation and Protection) 寄存器的值
    ///
    /// 在启用 Sv39 页表时，satp 即作为三级页表的根节点。其第 60-63 位为模式位，为 0b100 时启用 Sv39 页表
    pub fn token(&self) -> usize {
        self.root.0 | (8 << 60)
    }

    /// 映射虚拟页号到物理页号
    ///
    /// 根据虚拟页号查找页表项，并将其映射到给定的物理页号和标志位
    pub fn map(&mut self, vpn: VirtualPageNumber, ppn: PhysicalPageNumber, flags: PTEFlags) {
        let entry = self.find_pte_mut(vpn).unwrap();
        if (*entry).valid() {
            panic!("map: {:?} has been mapped", vpn);
        }
        *entry = PageTableEntry::new(ppn, flags | PTEFlags::Valid);
    }

    /// 取消映射虚拟页号
    ///
    /// 根据虚拟页号查找页表项，并将其设置为默认值，取消映射
    pub fn unmap(&mut self, vpn: VirtualPageNumber) {
        let entry = self.find_pte(vpn).unwrap();
        if !(*entry).valid() {
            panic!("unmap: {:?} has not been mapped", vpn);
        }
        *entry = PageTableEntry::default();
    }

    /// 查找并返回可变的页表项
    ///
    /// 根据虚拟页号查找页表项，如果不存在则分配新的页表项
    pub fn find_pte_mut(&mut self, vpn: VirtualPageNumber) -> Option<&mut PageTableEntry> {
        let indexes = vpn.indexes();
        let mut ppn = self.root;
        for i in 0..3 {
            let entry = &mut ppn.get_pte_array()[indexes[i]];
            if i == 2 {
                return Some(entry);
            }
            if !entry.valid() {
                let frame = StackFrameAllocator::alloc_frame().unwrap();
                *entry = PageTableEntry::new(frame.frame, PTEFlags::Valid);
                self.frames.push(frame);
            }
            ppn = PhysicalPageNumber::from(entry);
        }
        None
    }

    /// 查找并返回不可变的页表项
    ///
    /// 根据虚拟页号查找页表项，如果不存在则返回 None
    pub fn find_pte(&self, vpn: VirtualPageNumber) -> Option<&mut PageTableEntry> {
        let indexes = vpn.indexes();
        let mut ppn = self.root;
        for i in 0..3 {
            let entry = &mut ppn.get_pte_array()[indexes[i]];
            if i == 2 {
                return Some(entry);
            }
            if !entry.valid() {
                return None;
            }
            ppn = PhysicalPageNumber::from(entry);
        }
        None
    }

    /// 翻译虚拟页号到页表项
    ///
    /// 根据虚拟页号查找页表项，并返回其副本
    pub fn translate(&self, vpn: VirtualPageNumber) -> Option<PageTableEntry> {
        self.find_pte(vpn).map(|entry| *entry)
    }

    /// 翻译虚拟地址到物理地址
    ///
    /// 根据虚拟地址查找页表项，并计算物理地址
    pub fn translate_addr(&self, va: VirtualAddress) -> Option<PhysicalAddress> {
        self.find_pte(VirtualPageNumber::from(va)).map(|entry| {
            let floor = PhysicalAddress::from(PhysicalPageNumber::from(entry));
            let offset = va.page_offset();
            PhysicalAddress::from(usize::from(floor) + offset)
        })
    }

    /// 从 SATP 寄存器创建页表
    ///
    /// 根据 SATP 寄存器中的值创建页表结构体
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
        /// 有效位
        const Valid = 1 << 0;
        /// 可读位
        const Read = 1 << 1;
        /// 可写位
        const Write = 1 << 2;
        /// 可执行位
        const Execute = 1 << 3;
        /// 用户位
        const User = 1 << 4;
        /// 全局位
        const Global = 1 << 5;
        /// 访问位
        const Accessed = 1 << 6;
        /// 脏位
        const Dirty = 1 << 7;
    }
}

/// 页表项 PTE
///
/// 64 位 RISC-V 地址空间下，页表项大小为 64 位
///
/// ## 页表项格式
/// | 63-54 | 53-10 | 9-8 | 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |
/// |-------|-------|-----|---|---|---|---|---|---|---|---|
/// | 保留位 | 物理页号 | RSW | D | A | G | U | X | W | R | V |
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PageTableEntry {
    pub bits: usize, // 页表项的位字段
}

impl PageTableEntry {
    /// 创建一个新的页表项
    ///
    /// 根据给定的物理页号和标志位创建页表项
    pub fn new(phy_page: PhysicalPageNumber, flags: PTEFlags) -> Self {
        Self {
            bits: (phy_page.0 << 10) | flags.bits() as usize,
        }
    }

    /// 判断页表项是否有效
    ///
    /// 检查页表项的有效位是否被设置
    pub fn valid(&self) -> bool {
        (PTEFlags::Valid & self.into()) != PTEFlags::empty()
    }

    /// 判断页表项是否可读
    ///
    /// 检查页表项的可读位是否被设置
    pub fn readable(&self) -> bool {
        (PTEFlags::Read & self.into()) != PTEFlags::empty()
    }

    /// 判断页表项是否可写
    ///
    /// 检查页表项的可写位是否被设置
    pub fn writable(&self) -> bool {
        (PTEFlags::Write & self.into()) != PTEFlags::empty()
    }

    /// 判断页表项是否可执行
    ///
    /// 检查页表项的可执行位是否被设置
    pub fn executable(&self) -> bool {
        (PTEFlags::Execute & self.into()) != PTEFlags::empty()
    }

    /// 判断页表项是否为用户态
    ///
    /// 检查页表项的用户位是否被设置
    pub fn user(&self) -> bool {
        (PTEFlags::User & self.into()) != PTEFlags::empty()
    }

    /// 判断页表项是否被访问
    ///
    /// 检查页表项的访问位是否被设置
    pub fn accessed(&self) -> bool {
        (PTEFlags::Accessed & self.into()) != PTEFlags::empty()
    }

    /// 判断页表项是否脏
    ///
    /// 检查页表项的脏位是否被设置
    pub fn dirty(&self) -> bool {
        (PTEFlags::Dirty & self.into()) != PTEFlags::empty()
    }
}

impl Default for PageTableEntry {
    /// 创建一个默认的页表项
    ///
    /// 默认页表项的位字段为 0
    fn default() -> Self {
        Self { bits: 0 }
    }
}

impl From<&PageTableEntry> for PTEFlags {
    /// 从页表项创建标志位
    ///
    /// 根据页表项的位字段创建标志位
    fn from(pte: &PageTableEntry) -> Self {
        Self::from_bits_truncate(pte.bits as u8)
    }
}

impl From<&PageTableEntry> for PhysicalPageNumber {
    /// 从页表项创建物理页号
    ///
    /// 根据页表项的位字段创建物理页号
    fn from(pte: &PageTableEntry) -> Self {
        PhysicalPageNumber::from(pte.bits >> 10 & ((1 << PHYSICAL_PAGE_NUMBER_WIDTH_SV39) - 1))
    }
}

impl From<&mut PageTableEntry> for PhysicalPageNumber {
    /// 从可变页表项创建物理页号
    ///
    /// 根据页表项的位字段创建物理页号
    fn from(pte: &mut PageTableEntry) -> Self {
        PhysicalPageNumber::from(pte.bits >> 10 & ((1 << PHYSICAL_PAGE_NUMBER_WIDTH_SV39) - 1))
    }
}

pub struct UserBuffer {
    pub buffers: Vec<&'static mut [u8]>,
}

impl UserBuffer {
    pub fn new(buffers: Vec<&'static mut [u8]>) -> Self {
        Self { buffers }
    }

    pub fn len(&self) -> usize {
        let mut total: usize = 0;
        for buf in self.buffers.iter() {
            total += buf.len();
        }
        total
    }
}

/// 根据给定的页表 token、指针和长度，获取翻译后的切片。
///
/// 用于将一个指针指向的用户地址空间区域翻译为内核地址空间的切片。内核可以通过切片引用到用户地址空间的数据。
///
/// 该函数会将用户地址空间区域根据其虚拟页号划分为多个页，然后逐页翻译，翻译时进行页表查找，获取物理页号。
/// 再获取物理页号的字节数组，根据起始偏移和结束偏移获取切片。
/// 最后将翻译后的切片收集到一个向量中并返回。
///
/// ## 参数
/// - `token`: 页表的 token，用于从 SATP 寄存器中创建页表。
/// - `ptr`: 指向内存区域的指针。
/// - `len`: 内存区域的长度。
/// - `get_slice`: 一个闭包函数，用于根据物理页号、起始偏移和结束偏移获取切片。
///
/// ## 泛型参数
/// - `T`: 切片的类型。例如 `&'static [u8]` 或 `&'static mut [u8]`。
///
/// ## 返回值
/// 返回一个包含翻译后切片的向量数组。
fn get_translated_slices<T>(
    token: usize,
    ptr: *const u8,
    len: usize,
    get_slice: impl Fn(&PhysicalPageNumber, usize, usize) -> T,
) -> Vec<T> {
    let page_table = PageTable::from_satp(token);
    let mut start = ptr as usize;
    let end = start + len;
    let mut slices = Vec::new();
    while start < end {
        let start_va = VirtualAddress::from(start);
        let vpn = start_va.floor_page();
        let ppn = PhysicalPageNumber::from(&page_table.translate(vpn).unwrap());
        let end_va = VirtualAddress::from(vpn + VirtualPageNumber(1));
        let end_va = end_va.min(VirtualAddress::from(end));
        let slice = get_slice(&ppn, start_va.page_offset(), end_va.page_offset());
        slices.push(slice);
        start = usize::from(end_va);
    }
    slices
}

/// 根据给定的页表 token、指针和长度，获取翻译后的字节切片。
///
/// 用于将一个指针指向的用户地址空间区域翻译为内核地址空间的字节切片。内核可以通过切片引用到用户地址空间的数据。
///
/// ## 参数
/// - `token`: 页表的 token，用于从 SATP 寄存器中创建页表。
/// - `ptr`: 指向内存区域的指针。
/// - `len`: 内存区域的长度。
///
/// ## 返回值
/// 返回一个包含翻译后字节切片的向量数组。
pub fn get_translated_byte_slices(token: usize, ptr: *const u8, len: usize) -> Vec<&'static [u8]> {
    get_translated_slices(token, ptr, len, |ppn, start_offset, end_offset| {
        if end_offset == 0 {
            &ppn.get_bytes_array()[start_offset..]
        } else {
            &ppn.get_bytes_array()[start_offset..end_offset]
        }
    })
}

/// 根据给定的页表 token、指针和长度，获取翻译后的可变字节切片。
///
/// 用于将一个指针指向的用户地址空间区域翻译为内核地址空间的可变字节切片。内核可以通过切片引用到用户地址空间的数据。
///
/// ## 参数
/// - `token`: 页表的 token，用于从 SATP 寄存器中创建页表。
/// - `ptr`: 指向内存区域的指针。
/// - `len`: 内存区域的长度。
///
/// ## 返回值
/// 返回一个包含翻译后可变字节切片的向量数组。
pub fn get_mut_translated_byte_slices(
    token: usize,
    ptr: *const u8,
    len: usize,
) -> Vec<&'static mut [u8]> {
    get_translated_slices(token, ptr, len, |ppn, start_offset, end_offset| {
        if end_offset == 0 {
            &mut ppn.get_bytes_array()[start_offset..]
        } else {
            &mut ppn.get_bytes_array()[start_offset..end_offset]
        }
    })
}

/// 根据给定的页表 token 和指针，获取翻译后的字符串。
///
/// 用于将一个指针指向的用户地址空间区域翻译为内核地址空间的字符串。内核可以通过字符串引用到用户地址空间的数据。
/// ## 参数
/// - `token`: 页表的 token，用于从 SATP 寄存器中创建页表。
/// - `ptr`: 指向内存区域的指针。
/// ## 返回值
/// 返回一个包含翻译后字符串的 String 对象。
pub fn get_translated_string(token: usize, ptr: *const u8) -> String {
    let page_table = PageTable::from_satp(token);
    let mut string = String::new();
    let mut start = ptr as usize;
    loop {
        let ch: u8 = *(page_table
            .translate_addr(VirtualAddress::from(start))
            .unwrap())
        .get_mut();
        if ch == 0 {
            return string;
        } else {
            string.push(ch as char);
            start += 1;
        }
    }
}

/// 根据给定的页表 token 和指针，获取翻译后的可变引用。
///
/// 用于将一个指针指向的用户地址空间区域翻译为内核地址空间的可变引用。内核可以通过可变引用访问用户地址空间的数据。
/// ## 参数
/// - `token`: 页表的 token，用于从 SATP 寄存器中创建页表。
/// - `ptr`: 指向内存区域的指针。
/// ## 返回值
/// 返回一个包含翻译后可变引用的静态生命周期引用。
pub fn get_translated_refmut<T>(token: usize, ptr: *mut T) -> &'static mut T {
    let page_table = PageTable::from_satp(token);
    page_table
        .translate_addr(VirtualAddress::from(ptr as usize))
        .unwrap()
        .get_mut()
}
