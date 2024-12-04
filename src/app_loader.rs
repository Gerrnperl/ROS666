use core::arch::asm;

use crate::{APP_MANAGER, batch::AppManager};

pub fn load_apps() {
    let app_manager = APP_MANAGER.ref_cell.borrow();
    for i in 0..app_manager.app_count {
        load_app(&app_manager, i);
    }
    unsafe {
        asm!("fence.i");
    }
}

fn load_app(app_manager: &AppManager, app_id: usize) {
    let app_start = app_manager.app_table[app_id];
    let app_end = if app_id + 1 < app_manager.app_count {
        app_manager.app_table[app_id + 1]
    } else {
        app_manager.apps_end
    };
    let app_size = app_end - app_start;
    let app_base = AppManager::get_app_base_addr(app_id);
    unsafe { core::slice::from_raw_parts_mut(app_base as *mut u8, 0x20000).fill(0) };
    let app_src = unsafe { core::slice::from_raw_parts(app_start as *const u8, app_size) };
    let app_dst = unsafe { core::slice::from_raw_parts_mut(app_base as *mut u8, app_size) };
    app_dst.copy_from_slice(app_src);
}
