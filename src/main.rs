#![no_std]
#![no_main]
#![feature(inline_const_pat)]
#![feature(alloc_error_handler)]

extern crate alloc;

mod app_loader;
mod io;
mod language_item;
mod mm;
mod sbi;
mod syscall;
mod task;
mod timer;
mod trap;
mod utils;

use core::{arch::global_asm, cell::RefCell};

use app_loader::APP_NAMES;

global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("app_loader.asm"));

/// 内核入口函数
#[unsafe(no_mangle)]
pub extern "C" fn _kernel_entry() -> ! {
    clear_bss();
    startup_log();
    trap::init();

    mm::init::init();
    mm::memory_set::remap_test();

    info!("Hi there, it's {}.", "ROS666");

    APP_NAMES.iter().for_each(|(name, id)| {
        info!("App: {} id: {}", name, id);
    });

    task::init();

    trap::init::enable_timer_interrupt();
    timer::set_next_timeout(trap::handler::TIMER_INTERVAL_USEC);
    // APP_LOADER.ref_cell.borrow().print_apps_info();
    task::processor::Processor::run_tasks();

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
