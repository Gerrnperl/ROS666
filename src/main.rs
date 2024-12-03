#![no_std]
#![no_main]

mod batch;
mod io;
mod language_item;
mod sbi;
mod utils;

use core::{arch::global_asm, cell::RefCell};

use lazy_static::lazy_static;
use utils::safety::SyncRefCell;

global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("app_loader.asm"));

lazy_static! {
    static ref APP_MANAGER: SyncRefCell<batch::AppManager> = {
        let app_count = extern_global!(__app_count) as *const usize;
        let app_count = unsafe { app_count.read_volatile() };
        let app_table = extern_global!(__app_table) as *const usize;
        let app_name_table = extern_global!(__app_name_table) as *const usize;

        SyncRefCell {
            ref_cell: RefCell::new(batch::AppManager::new(app_count, app_table, app_name_table)),
        }
    };
}

/// 内核入口函数
#[unsafe(no_mangle)]
pub extern "C" fn _kernel_entry() -> ! {
    clear_bss();
    startup_log();
    APP_MANAGER.ref_cell.borrow().print_apps_info();
    APP_MANAGER.ref_cell.borrow_mut().load_app(0);
    printkln!("Hello, {}!", "World");

    // sbi::sbi_shutdown(false);
    loop {}
}

fn startup_log() {
    info!("[Kernel] Secion:");
    info!(
        "[Kernel]  text   : [{:#x}, {:#x})",
        extern_global!(__text_start) as usize,
        extern_global!(__text_end) as usize
    );
    info!(
        "[Kernel]  rodata : [{:#x}, {:#x})",
        extern_global!(__rodata_start) as usize,
        extern_global!(__rodata_end) as usize
    );
    info!(
        "[Kernel]  data   : [{:#x}, {:#x})",
        extern_global!(__data_start) as usize,
        extern_global!(__data_end) as usize
    );
    info!(
        "[Kernel]  bss    : [{:#x}, {:#x})",
        extern_global!(__bss_start) as usize,
        extern_global!(__bss_end) as usize
    );
}

/// 清空 BSS 段
fn clear_bss() {
    unsafe extern "C" {
        fn __bss_start();
        fn __bss_end();
    }

    for i in __bss_start as usize..__bss_end as usize {
        unsafe {
            core::ptr::write_volatile(i as *mut u8, 0);
        }
    }
}
