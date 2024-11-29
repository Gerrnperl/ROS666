use core::panic::PanicInfo;

use crate::{printkln, sbi::sbi_shutdown};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let message = info.message();
    printkln!("Kernel panic: {:?}", message);
    if let Some(location) = info.location() {
        let file = location.file();
        let line = location.line();
        let column = location.column();
        printkln!("\tat {}:{}:{}", file, line, column);
    }

    sbi_shutdown(true);
}
