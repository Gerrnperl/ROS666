use core::{arch::asm, cell::RefCell};

use lazy_static::lazy_static;

use crate::{
    extern_global, info,
    mm::{address::PAGE_SIZE_SV39, memory_set::TRAMPOLINE},
    task::manager::TaskManager,
    trap::context::TrapCtx,
    utils::safety::SyncRefCell,
};

pub const KERNEL_STACK_SIZE: usize = 4096 * 2;
pub const USER_STACK_SIZE: usize = 4096 * 2;

pub fn kernel_stack_position(app_id: usize) -> (usize, usize) {
    let top = TRAMPOLINE - app_id * (KERNEL_STACK_SIZE + PAGE_SIZE_SV39);
    let bottom = top - KERNEL_STACK_SIZE;
    (bottom, top)
}

pub fn get_app_count() -> usize {
    let app_count = extern_global!(__app_count) as *const usize;
    let app_count = unsafe { app_count.read_volatile() };
    app_count
}

pub struct AppData {
    pub app_id: usize,
    pub app_name: &'static str,
    pub data: &'static [u8],
}

pub fn load_app_data(app_id: usize) -> AppData {
    let app_count = get_app_count();
    assert!(app_id < app_count);
    let app_table = extern_global!(__app_table) as *const usize;
    let app_name_table = extern_global!(__app_name_table) as *const usize;
    let app_start = unsafe {
        core::slice::from_raw_parts(app_table.add(app_id), 1)
            .get(0)
            .copied()
            .unwrap()
    };
    let app_end: usize = if app_id + 1 < app_count {
        unsafe {
            core::slice::from_raw_parts(app_table.add(app_id + 1), 1)
                .get(0)
                .copied()
                .unwrap()
        }
    } else {
        unsafe {
            core::slice::from_raw_parts(app_table.add(app_count), 1)
                .get(0)
                .copied()
                .unwrap()
        }
    };
    let app_name_ptr = unsafe {
        core::slice::from_raw_parts(app_name_table.add(app_id), 1)
            .get(0)
            .copied()
            .unwrap()
    };
    let app_name = get_app_name(app_name_ptr as *const i8);
    let app_data: &[u8] =
        unsafe { core::slice::from_raw_parts(app_start as *const u8, app_end - app_start) };
    AppData {
        app_id,
        app_name,
        data: app_data,
    }
}

pub fn get_app_name(app_name_ptr: *const i8) -> &'static str {
    let app_name = unsafe { core::ffi::CStr::from_ptr(app_name_ptr) };
    let app_name = app_name.to_str().unwrap();
    app_name
}

impl AppData {
    pub fn print_info(&self) {
        info!(
            "App {} - Name: {}, Size: {:#x}",
            self.app_id,
            self.app_name,
            self.data.len(),
        );
    }
}
