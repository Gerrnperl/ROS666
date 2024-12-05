use core::arch::asm;

use common::syscall::{Syscall, SyscallArgs, SyscallRet};

pub use common::syscall::time::{TimeVal, TimeZone};

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

pub fn sys_sched_yield() -> SyscallRet {
    syscall(Syscall::SchedYield, [0, 0, 0])
}

pub fn sys_get_time_of_day(ts: *mut TimeVal, tz: *mut TimeZone) -> SyscallRet {
    syscall(Syscall::GetTimeOfDay, [ts as usize, tz as usize, 0])
}
