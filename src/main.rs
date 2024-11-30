#![no_std]
#![no_main]

mod io;
mod language_item;
mod sbi;

use core::arch::global_asm;

global_asm!(include_str!("entry.asm"));

/// 内核入口函数
#[unsafe(no_mangle)]
pub extern "C" fn _kernel_entry() -> ! {
    clear_bss();
    startup_log();
    printkln!("Hello, {}!", "World");

    // sbi::sbi_shutdown(false);
    loop {}
}

fn startup_log() {
    unsafe extern "C" {
        fn __text_start();
        fn __text_end();
        fn __rodata_start();
        fn __rodata_end();
        fn __data_start();
        fn __data_end();
        fn __bss_start();
        fn __bss_end();
    }

    info!("[Kernel] Secion:");
    info!(
        "[Kernel]  text   : [{:#x}, {:#x})",
        __text_start as usize, __text_end as usize
    );
    info!(
        "[Kernel]  rodata : [{:#x}, {:#x})",
        __rodata_start as usize, __rodata_end as usize
    );
    info!(
        "[Kernel]  data   : [{:#x}, {:#x})",
        __data_start as usize, __data_end as usize
    );
    info!(
        "[Kernel]  bss    : [{:#x}, {:#x})",
        __bss_start as usize, __bss_end as usize
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
