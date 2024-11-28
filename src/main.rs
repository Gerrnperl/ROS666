#![no_std]
#![no_main]

mod language_item;
mod sbi;

use core::arch::global_asm;

global_asm!(include_str!("entry.asm"));

#[unsafe(no_mangle)]
pub extern "C" fn _kernel_entry() -> ! {
    clear_bss();
    // Say hello :)
    let hello = b"Hello, World!";
    for &c in hello {
        sbi::sbi_console_putchar(c as usize);
    }
    loop {}
}

/// Clear the .bss section
fn clear_bss() {
    unsafe extern "C" {
        static mut __bss_start: u64;
        static mut __bss_end: u64;
    }

    let bss_start = unsafe { __bss_start as usize };
    let bss_end = unsafe { __bss_end as usize };

    for i in bss_start..bss_end {
        unsafe {
            core::ptr::write_volatile(i as *mut u8, 0);
        }
    }
}
