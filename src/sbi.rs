pub fn sbi_console_putchar(ch: usize) {
    #[allow(deprecated)]
    sbi_rt::legacy::console_putchar(ch);
}
