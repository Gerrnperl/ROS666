use core::arch::asm;

/// 系统调用号
///
/// 为调试方便，应当与 Linux 系统为 RISC-V 架构定义的系统调用号保持一致
///
/// https://gpages.juszkiewicz.com.pl/syscalls-table/syscalls.html
pub enum Syscall {
    Read = 63,
    Write = 64,
    /// Terminate the calling process
    ///
    /// [_exit(2) — Linux manual page](https://www.man7.org/linux/man-pages/man2/exit.2.html)
    Exit = 93,
}

impl From<Syscall> for usize {
    fn from(syscall: Syscall) -> usize {
        syscall as usize
    }
}

pub type SyscallArgs = [usize; 3];
pub type SyscallRet = isize;

/// 调用系统调用
///
/// 通过 ecall 指令触发 Trap，进入 M 态
pub fn syscall(call: Syscall, args: SyscallArgs) -> SyscallRet {
    let mut ret: SyscallRet;
    let callid: usize = call.into();
    unsafe {
        // Ecall
        // 输入: a0 - a2 存放参数，a7 存放系统调用号
        // 输出: a0 存放返回值
        asm!(
            "ecall",
            inlateout("a0") args[0] => ret,
            in("a1") args[1],
            in("a2") args[2],
            in("a7") callid,
        );
    }
    ret
}

pub fn sys_write(fd: usize, buffer: &[u8]) -> SyscallRet {
    syscall(Syscall::Write, [fd, buffer.as_ptr() as usize, buffer.len()])
}

/// 退出当前进程, 报告返回值
pub fn sys_exit(code: usize) -> ! {
    syscall(Syscall::Exit, [code, 0, 0]);
    unreachable!("Unreachable after sys_exit");
}
