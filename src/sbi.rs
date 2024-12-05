use core::ops::Add;

use rustsbi::{Console, Physical, Reset, SbiRet, Timer};

pub enum ConsolePutError {
    AddrInvalid,
    SBIFailed(SbiRet),
}

/// 输出字符到控制台
pub fn sbi_console_putchar(ch: usize) -> Result<(), ConsolePutError> {
    let sbi_ret = rustsbi::Forward {}.write_byte(ch as u8);
    if sbi_ret.is_ok() {
        Ok(())
    } else {
        Err(ConsolePutError::SBIFailed(sbi_ret))
    }
}

/// 输出字符串到控制台
pub fn sbi_console_put(s: &str) -> Result<(), ConsolePutError> {
    let num_bytes = s.len();
    if num_bytes == 0 {
        return Ok(());
    }
    let phys_addr_lo = s.as_ptr() as usize;
    if let Some(phys_addr_hi) = phys_addr_lo.checked_add(num_bytes - 1) {
        let sbi_ret =
            rustsbi::Forward {}.write(Physical::new(num_bytes, phys_addr_lo, phys_addr_hi));
        if sbi_ret.is_ok() {
            Ok(())
        } else {
            Err(ConsolePutError::SBIFailed(sbi_ret))
        }
    } else {
        Err(ConsolePutError::AddrInvalid)
    }
}

/// SRST 系统重置类型
///
/// https://github.com/riscv-non-isa/riscv-sbi-doc/blob/master/src/ext-sys-reset.adoc
enum ResetType {
    Shutdown = 0x00000000,
    ColdReboot = 0x00000001,
    WarmReboot = 0x00000002,
}

/// SRST 系统重置原因
///
/// https://github.com/riscv-non-isa/riscv-sbi-doc/blob/master/src/ext-sys-reset.adoc
enum ResetReason {
    NoReason = 0x00000000,
    SystemFailure = 0x00000001,
}

impl From<ResetType> for u32 {
    fn from(rt: ResetType) -> u32 {
        rt as u32
    }
}

impl From<ResetReason> for u32 {
    fn from(rr: ResetReason) -> u32 {
        rr as u32
    }
}

pub fn sbi_shutdown(failure: bool) -> ! {
    if failure {
        rustsbi::Forward {}.system_reset(
            ResetType::Shutdown.into(),
            ResetReason::SystemFailure.into(),
        );
    } else {
        rustsbi::Forward {}.system_reset(ResetType::Shutdown.into(), ResetReason::NoReason.into());
    }
    unreachable!("sbi_shutdown");
}

/// 设置下一个时钟中断触发时间 (mtimecmp)
pub fn sbi_set_timer(stime_value: u64) {
    rustsbi::Forward {}.set_timer(stime_value);
}
