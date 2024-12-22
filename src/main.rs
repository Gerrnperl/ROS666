#![no_std]
#![no_main]
#![feature(inline_const_pat)]
#![feature(alloc_error_handler)]

extern crate alloc;

mod drivers;
mod fs;
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

use fs::inode::ROOT_INODE;

global_asm!(include_str!("entry.asm"));

/// 内核入口函数
#[unsafe(no_mangle)]
pub extern "C" fn _kernel_entry() -> ! {
    print_logo();
    trace!("[Kernel] Clear BSS");
    clear_bss();
    startup_log();
    trace!("[Kernel] Init trap");
    trap::init();
    trace!("[Kernel] Init memory");
    mm::init::init();
    mm::memory_set::remap_test();

    trace!("[Kernel] Load file system");
    for app in ROOT_INODE.ls() {
        info!("App: {}", app);
    }

    trace!("[Kernel] Init process manager");
    task::init();

    trace!("[Kernel] Enable timer interrupt");
    trap::init::enable_timer_interrupt();
    timer::set_next_timeout(trap::handler::TIMER_INTERVAL_USEC);

    task::processor::Processor::run_tasks();

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

fn print_logo() {
    printkln!(
        r#"
+--------------------------------------------------------------------+
|      _/_/_/      _/_/      _/_/_/    _/_/_/    _/_/_/    _/_/_/    |
|     _/    _/  _/    _/  _/        _/        _/        _/           |
|    _/_/_/    _/    _/    _/_/    _/_/_/    _/_/_/    _/_/_/        |
|   _/    _/  _/    _/        _/  _/    _/  _/    _/  _/    _/       |
|  _/    _/    _/_/    _/_/_/      _/_/      _/_/      _/_/          |
+--------------------------------------------------------------------+
"#
    );
}
