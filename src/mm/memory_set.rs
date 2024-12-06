use core::{arch::asm, borrow::BorrowMut, cell::RefCell, ops::Range};

use crate::{
    app_loader::{AppData, USER_STACK_SIZE},
    extern_global, info,
    mm::{address::PAGE_SIZE_SV39, frame_allocator::MEMORY_END},
    printkln,
    utils::safety::SyncRefCell,
};

use super::{
    address::{PhysicalPageNumber, VPNRange, VirtualAddress, VirtualPageNumber},
    frame_allocator::{FrameTracker, StackFrameAllocator},
    page_table::{self, PageTable},
};
use alloc::{collections::btree_map::BTreeMap, sync::Arc, vec::Vec};
use bitflags::bitflags;
use riscv::register::satp;
use xmas_elf::ElfFile;

use lazy_static::lazy_static;

lazy_static! {
    pub static ref KERNEL_SPACE: Arc<SyncRefCell<MemorySet>> = {
        Arc::new(SyncRefCell {
            ref_cell: RefCell::new(MemorySet::new_kernel()),
        })
    };
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum MapType {
    Linear,
    Framed,
}

bitflags! {
    #[derive(Copy, Clone, Debug)]
    pub struct MapPermission: u8 {
        const Read = 1 << 1;
        const Write = 1 << 2;
        const Execute = 1 << 3;
        const User = 1 << 4;
    }
}

/// 一段虚拟内存 (逻辑段)
pub struct MapArea {
    vpn_range: VPNRange,
    data: BTreeMap<VirtualPageNumber, FrameTracker>,
    map_type: MapType,
    permission: MapPermission,
}

/// 一组虚拟内存 (地址空间)
pub struct MemorySet {
    page_table: PageTable,
    areas: Vec<MapArea>,
}

impl MapArea {
    pub fn new(vpn_range: VPNRange, map_type: MapType, permission: MapPermission) -> Self {
        Self {
            vpn_range,
            data: BTreeMap::new(),
            map_type,
            permission,
        }
    }

    pub fn map(&mut self, page_table: &mut PageTable) {
        for vpn in self.vpn_range {
            self.map_one(vpn, page_table);
        }
    }

    pub fn unmap(&mut self, page_table: &mut PageTable) {
        for vpn in self.vpn_range {
            self.unmap_one(vpn, page_table);
        }
    }

    pub fn get_frame(&self, vpn: VirtualPageNumber) -> Option<&FrameTracker> {
        self.data.get(&vpn)
    }

    pub fn get_permission(&self) -> MapPermission {
        self.permission
    }

    pub fn map_one(&mut self, vpn: VirtualPageNumber, page_table: &mut PageTable) {
        let ppn = match self.map_type {
            MapType::Linear => PhysicalPageNumber(vpn.0),
            MapType::Framed => {
                let frame = StackFrameAllocator::alloc_frame().unwrap();
                let ppn = frame.frame;
                self.data.insert(vpn, frame);
                ppn
            }
        };
        let flags = page_table::PTEFlags::from_bits(self.permission.bits()).unwrap();
        page_table.map(vpn, ppn, flags);
    }

    pub fn unmap_one(&mut self, vpn: VirtualPageNumber, page_table: &mut PageTable) {
        match self.map_type {
            MapType::Linear => {}
            MapType::Framed => {
                self.data.remove(&vpn);
                // StackFrameAllocator::dealloc_frame(ppn);
            }
        }
        page_table.unmap(vpn);
    }

    pub fn copy_from(&mut self, page_table: &mut PageTable, src: &[u8]) {
        if self.map_type != MapType::Framed {
            panic!("copy_from: only support framed mapping");
        }
        let len = src.len();
        let mut offset = 0;
        let mut current_vpn = self.vpn_range.start;
        while offset < len {
            let vpn = current_vpn;
            let pte = page_table.translate(vpn).unwrap();
            let dst = PhysicalPageNumber::from(&pte).get_bytes_array();
            let src = &src[offset..];
            let copy_len = dst.len().min(src.len());
            dst[..copy_len].copy_from_slice(&src[..copy_len]);
            offset += copy_len;
            current_vpn.0 += 1;
        }
    }
}

impl MemorySet {
    pub fn empty() -> Self {
        Self {
            page_table: PageTable::new(),
            areas: Vec::new(),
        }
    }

    pub fn activate(&self) {
        let satp = self.page_table.token();
        unsafe {
            satp::write(satp);
            asm!("sfence.vma"); // 刷新 TLB
        }
    }

    pub fn push(&mut self, mut area: MapArea, data: Option<&[u8]>) {
        area.map(&mut self.page_table);
        if let Some(data) = data {
            area.copy_from(&mut self.page_table, data);
        }
        self.areas.push(area);
    }

    pub fn insert(&mut self, va_range: Range<VirtualAddress>, permission: MapPermission) {
        self.push(
            MapArea::new(
                VPNRange::from_addr(va_range.start, va_range.end),
                MapType::Framed,
                permission,
            ),
            None,
        );
    }

    pub fn map_trampoline(&mut self) {
        unimplemented!("map_trampoline");
        unsafe extern "C" {
            fn _start();
            fn trampoline();
        }
        // let mut trampoline = MapArea::new(
        //     VPNRange::new(
        //         VirtualAddress::from(_start).into(),
        //         VirtualAddress::from(trampoline).into(),
        //     ),
        //     MapType::Linear,
        //     MapPermission::Read | MapPermission::Execute,
        // );
        // trampoline.map(&mut self.page_table)6;
        // self.areas.push(trampoline);
    }
    pub fn new_kernel() -> Self {
        let kernel_start = extern_global!(__kernel_start) as usize;
        let kernel_end = extern_global!(__kernel_end) as usize;
        let text_start = extern_global!(__text_start) as usize;
        let text_end = extern_global!(__text_end) as usize;
        let rodata_start = extern_global!(__rodata_start) as usize;
        let rodata_end = extern_global!(__rodata_end) as usize;
        let data_start = extern_global!(__data_start) as usize;
        let data_end = extern_global!(__data_end) as usize;
        let bss_start = extern_global!(__bss_start_stack) as usize;
        let bss_end = extern_global!(__bss_end) as usize;

        let mut memory_set = MemorySet::empty();
        // memory_set.map_trampoline();
        info!("mapping .text: [{:#x}, {:#x})", text_start, text_end);
        memory_set.push(
            MapArea::new(
                VPNRange::from_addr(
                    VirtualAddress::from(text_start),
                    VirtualAddress::from(text_end),
                ),
                MapType::Linear,
                MapPermission::Read | MapPermission::Execute,
            ),
            None,
        );
        info!("mapping .rodata: [{:#x}, {:#x})", rodata_start, rodata_end);
        memory_set.push(
            MapArea::new(
                VPNRange::from_addr(
                    VirtualAddress::from(rodata_start),
                    VirtualAddress::from(rodata_end),
                ),
                MapType::Linear,
                MapPermission::Read,
            ),
            None,
        );
        info!("mapping .data: [{:#x}, {:#x})", data_start, data_end);
        memory_set.push(
            MapArea::new(
                VPNRange::from_addr(
                    VirtualAddress::from(data_start),
                    VirtualAddress::from(data_end),
                ),
                MapType::Linear,
                MapPermission::Read | MapPermission::Write,
            ),
            None,
        );
        info!("mapping .bss: [{:#x}, {:#x})", bss_start, bss_end);
        memory_set.push(
            MapArea::new(
                VPNRange::from_addr(
                    VirtualAddress::from(bss_start),
                    VirtualAddress::from(bss_end),
                ),
                MapType::Linear,
                MapPermission::Read | MapPermission::Write,
            ),
            None,
        );
        info!("mapping heap: [{:#x}, {:#x})", kernel_end, MEMORY_END);
        memory_set.push(
            MapArea::new(
                VPNRange::from_addr(
                    VirtualAddress::from(kernel_end),
                    VirtualAddress::from(MEMORY_END),
                ),
                MapType::Linear,
                MapPermission::Read | MapPermission::Write,
            ),
            None,
        );
        memory_set
    }
    pub fn from_elf_app(app: AppData) -> (Self, usize, usize) {
        let elf = app.data;
        let elf = ElfFile::new(elf).unwrap();
        let elf_header = elf.header;
        let elf_magic = elf_header.pt1.magic;
        assert_eq!(elf_magic, [0x7f, 0x45, 0x4c, 0x46], "invalid elf file");
        let ph_count = elf_header.pt2.ph_count();
        let mut max_end = VirtualPageNumber(0);
        let mut memory_set = MemorySet::empty();
        memory_set.map_trampoline();
        for i in 0..ph_count {
            let ph = elf.program_header(i).unwrap();
            if ph.get_type().unwrap() != xmas_elf::program::Type::Load {
                continue;
            }
            let vpn_start = VirtualAddress::from(ph.virtual_addr() as usize).floor_page();
            let vpn_end = VirtualAddress::from(ph.virtual_addr() as usize + ph.mem_size() as usize)
                .ceil_page();
            let vpn_range = VPNRange::new(vpn_start, vpn_end);
            let map_type = MapType::Framed;
            let permission = {
                let mut flags = MapPermission::empty();
                if ph.flags().is_read() {
                    flags |= MapPermission::Read;
                }
                if ph.flags().is_write() {
                    flags |= MapPermission::Write;
                }
                if ph.flags().is_execute() {
                    flags |= MapPermission::Execute;
                }
                flags
            };
            let map_area = MapArea::new(vpn_range, map_type, permission);
            max_end = map_area.vpn_range.end();
            let data = Some(&elf.input[ph.offset() as usize..][..ph.file_size() as usize]);
            memory_set.push(map_area, data);
        }
        let max_end_va = VirtualAddress::from(max_end);
        let mut user_stack_bottom: usize = max_end_va.into();
        user_stack_bottom += PAGE_SIZE_SV39;
        let user_stack_top = user_stack_bottom + USER_STACK_SIZE;
        memory_set.push(
            MapArea::new(
                VPNRange::from_addr(
                    VirtualAddress::from(user_stack_bottom),
                    VirtualAddress::from(user_stack_top),
                ),
                MapType::Framed,
                MapPermission::Read | MapPermission::Write | MapPermission::User,
            ),
            None,
        );

        // memory_set.push(
        //     MapArea::new(
        //         VPNRange::new(
        //             VirtualAddress::from(TRAP_CONTEXT).into(),
        //             VirtualAddress::from(TRAMPOLINE).into(),
        //         ),
        //         MapType::Framed,
        //         MapPermission::Read | MapPermission::Write,
        //     ),
        //     None,
        // );
        (
            memory_set,
            user_stack_top,
            elf.header.pt2.entry_point() as usize,
        )
    }
}

pub fn remap_test() {
    let mut kernel_space = KERNEL_SPACE.ref_cell.borrow_mut();
    let mid_text: VirtualAddress =
        ((extern_global!(__text_start) as usize + extern_global!(__text_end) as usize) / 2).into();
    let mid_rodata: VirtualAddress =
        ((extern_global!(__rodata_start) as usize + extern_global!(__rodata_end) as usize) / 2)
            .into();
    let mid_data: VirtualAddress =
        ((extern_global!(__data_start) as usize + extern_global!(__data_end) as usize) / 2).into();
    assert_eq!(
        kernel_space
            .page_table
            .translate(mid_text.floor_page().into())
            .unwrap()
            .writable(),
        false
    );
    assert_eq!(
        kernel_space
            .page_table
            .translate(mid_rodata.floor_page().into())
            .unwrap()
            .writable(),
        false,
    );
    assert_eq!(
        kernel_space
            .page_table
            .translate(mid_data.floor_page().into())
            .unwrap()
            .executable(),
        false,
    );
    printkln!("remap_test passed!");
}
