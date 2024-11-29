use core::ops::Add;

use sbi_rt::{Physical, SbiRet};

pub enum ConsolePutError {
    AddrInvalid,
    SBIFailed(SbiRet),
}

/// 输出字符到控制台
pub fn sbi_console_putchar(ch: usize) -> Result<(), ConsolePutError> {
    let sbi_ret = sbi_rt::console_write_byte(ch as u8);
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
        let phys_addr_hi = phys_addr_hi >> 32;
        let sbi_ret = sbi_rt::console_write(Physical::new(num_bytes, phys_addr_lo, phys_addr_hi));
        if sbi_ret.is_ok() {
            Ok(())
        } else {
            Err(ConsolePutError::SBIFailed(sbi_ret))
        }
    } else {
        Err(ConsolePutError::AddrInvalid)
    }
}

pub fn sbi_shutdown(failure: bool) -> ! {
    if failure {
        sbi_rt::system_reset(sbi_rt::Shutdown, sbi_rt::SystemFailure);
    } else {
        sbi_rt::system_reset(sbi_rt::Shutdown, sbi_rt::NoReason);
    }
    unreachable!("sbi_shutdown");
}
