use core::{arch::asm, cell::RefCell};

use alloc::{collections::btree_map::BTreeMap, vec::Vec};
use lazy_static::lazy_static;

use crate::{
    extern_global, info,
    mm::{address::PAGE_SIZE_SV39, memory_set::TRAMPOLINE},
    task::manager::TaskManager,
    trap::context::TrapCtx,
    utils::safety::SyncRefCell,
};

lazy_static! {
    /// All app names
    pub static ref APP_NAMES: BTreeMap<&'static str, usize> = {
        let mut app_names = BTreeMap::new();
        let names = get_all_app_names();
        for (i, name) in names.iter().enumerate() {
            app_names.insert(*name, i);
        }
        app_names
    };
}

pub struct AppData {
    pub app_id: usize,
    pub app_name: &'static str,
    pub data: &'static [u8],
}

pub fn get_app_count() -> usize {
    let app_count = extern_global!(__app_count) as *const usize;
    let app_count = unsafe { app_count.read_volatile() };
    app_count
}

pub fn get_all_app_names() -> Vec<&'static str> {
    let app_count = get_app_count();
    let app_name_table = extern_global!(__app_name_table) as *const usize;
    let mut app_names = Vec::new();
    for i in 0..app_count {
        let app_name_ptr = unsafe {
            core::slice::from_raw_parts(app_name_table.add(i), 1)
                .get(0)
                .copied()
                .unwrap()
        };
        let app_name = get_app_name(app_name_ptr as *const i8);
        app_names.push(app_name);
    }
    app_names
}

pub fn load_app_data_by_name(app_name: &str) -> Option<AppData> {
    let app_id = APP_NAMES.get(app_name)?;
    Some(load_app_data(*app_id))
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
