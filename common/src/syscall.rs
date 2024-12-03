/// 系统调用号
///
/// 为调试方便，应当与 Linux 系统为 RISC-V 架构定义的系统调用号保持一致
///
/// https://gpages.juszkiewicz.com.pl/syscalls-table/syscalls.html
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Syscall {
    Read = 63,
    Write = 64,
    /// Terminate the calling process
    ///
    /// [_exit(2) — Linux manual page](https://www.man7.org/linux/man-pages/man2/exit.2.html)
    Exit = 93,
}

impl core::fmt::Display for Syscall {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<Syscall> for usize {
    fn from(syscall: Syscall) -> usize {
        syscall as usize
    }
}

impl From<usize> for Syscall {
    fn from(syscall: usize) -> Syscall {
        match syscall {
            63 => Syscall::Read,
            64 => Syscall::Write,
            93 => Syscall::Exit,
            _ => panic!("Unsupported syscall number: {}", syscall),
        }
    }
}

pub type SyscallArgs = [usize; 3];
pub type SyscallRet = isize;
