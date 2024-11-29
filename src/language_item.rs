use core::panic::PanicInfo;

use crate::{error, sbi::sbi_shutdown};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let message = info.message();
    error!("Kernel panic: {:?}", message);
    if let Some(location) = info.location() {
        let file = location.file();
        let line = location.line();
        let column = location.column();
        error!("\tat {}:{}:{}", file, line, column);
    }

    sbi_shutdown(true);
}
