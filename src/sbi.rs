use core::ops::Add;

use alloc::vec;
use rustsbi::{Console, Physical, Reset, SbiRet, Timer};

use crate::{
    mm::{self},
    task::manager::TaskManager,
};

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
    if s.is_empty() {
        return Ok(());
    }

    let num_bytes = s.len();

    if unsafe { mm::init::HEAP_INITED } {
        // 复制字符串到内核堆上
        // NOTE:
        // 在用户程序进行输出系统调用时，尽管已经调用了 `get_translated_byte_slices`
        // 将字符串所在的用户地址空间映射到内核地址空间，
        // 但直接将字符串地址传递给 sbi 会出现错误. 测试时可以观察到纯字符串可以输出，
        // 但是所有 format 参数无法输出并且会导致 panic.
        //
        // 为了解决这个问题，这里将字符串复制到内核堆上，然后将内核堆的地址传递给 sbi.
        //
        // 原因目前不明，可能是由于地址空间或者权限的问题.
        let buf = alloc::vec::Vec::from(s);
        let buf_ptr = buf.as_ptr();
        let physical = Physical::new(num_bytes, buf_ptr as usize, buf_ptr as usize + num_bytes);
        let sbi_ret = rustsbi::Forward {}.write(physical);

        return if sbi_ret.is_ok() {
            Ok(())
        } else {
            Err(ConsolePutError::SBIFailed(sbi_ret))
        };
    }
    // 此时堆未初始化，并且未启用分页机制
    let phys_addr_lo = s.as_ptr() as usize;
    if let Some(phys_addr_hi) = phys_addr_lo.checked_add(num_bytes - 1) {
        let physical = Physical::new(num_bytes, phys_addr_lo, phys_addr_hi);
        let sbi_ret = rustsbi::Forward {}.write(physical);

        return if sbi_ret.is_ok() {
            Ok(())
        } else {
            Err(ConsolePutError::SBIFailed(sbi_ret))
        };
    }

    Err(ConsolePutError::AddrInvalid)
}

/// 读取字符
pub fn sbi_console_getchar() -> Result<u8, SbiRet> {
    if unsafe { mm::init::HEAP_INITED } {
        let mut buf = vec![0u8; 1];
        let physical = Physical::new(1, buf.as_mut_ptr() as usize, buf.as_mut_ptr() as usize);
        let sbi_ret = rustsbi::Forward {}.read(physical);
        if sbi_ret.is_ok() {
            return Ok(buf[0]);
        }
        return Err(sbi_ret);
    } else {
        let mut buf = [0u8; 1];
        let physical = Physical::new(1, buf.as_mut_ptr() as usize, buf.as_mut_ptr() as usize);
        let sbi_ret = rustsbi::Forward {}.read(physical);
        if sbi_ret.is_ok() {
            return Ok(buf[0]);
        }
        return Err(sbi_ret);
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
