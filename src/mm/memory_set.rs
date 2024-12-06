use core::ops::Range;

use crate::{
    app_loader::{AppData, USER_STACK_SIZE},
    extern_global, info,
    mm::{address::PAGE_SIZE_SV39, frame_allocator::MEMORY_END},
};

use super::{
    address::{PhysicalPageNumber, VPNRange, VirtualAddress, VirtualPageNumber},
    frame_allocator::{FrameTracker, StackFrameAllocator},
    page_table::{self, PageTable},
};
use alloc::{collections::btree_map::BTreeMap, vec::Vec};
use bitflags::bitflags;
use xmas_elf::ElfFile;

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
            let ppn = page_table.translate(vpn).unwrap();
            let dst = ppn.get_bytes_array();
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
                VPNRange::new(va_range.start.into(), va_range.end.into()),
                MapType::Framed,
                permission,
            ),
            None,
        );
    }
}
