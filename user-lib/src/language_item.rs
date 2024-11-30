use core::panic::PanicInfo;

use crate::{exit, print, println};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let message = info.message();
    print!("Panicked at '{:?}", message);
    if let Some(location) = info.location() {
        let file = location.file();
        let line = location.line();
        let column = location.column();
        println!(", {}:{}:{}", file, line, column);
    }
    exit(1);
}
