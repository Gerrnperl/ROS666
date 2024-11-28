pub fn sbi_console_putchar(ch: usize) {
    #[allow(deprecated)]
    sbi_rt::legacy::console_putchar(ch);
}

pub fn sbi_shutdown(failure: bool) -> ! {
    if failure {
        sbi_rt::system_reset(sbi_rt::Shutdown, sbi_rt::SystemFailure);
    } else {
        sbi_rt::system_reset(sbi_rt::Shutdown, sbi_rt::NoReason);
    }
    unreachable!("sbi_shutdown");
}
