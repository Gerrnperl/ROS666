//! 调用系统调用的封装

use core::arch::asm;

use common::syscall::{Syscall, SyscallArgs, SyscallRet};

pub use common::syscall::time::{TimeVal, TimeZone};

/// 调用系统调用
///
/// 通过 ecall 指令触发 Trap，进入 M 态
///
/// ## 参数
/// - `call`: 系统调用号
/// - `args`: 系统调用参数
/// ## 返回值
/// 返回系统调用的结果
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

pub fn sys_openat(path: &str, flags: usize) -> SyscallRet {
    syscall(Syscall::OpenAt, [path.as_ptr() as usize, flags, 0])
}

pub fn sys_close(fd: usize) -> SyscallRet {
    syscall(Syscall::Close, [fd, 0, 0])
}

/// 读取文件描述符
///
/// ## 参数
/// - `fd`: 文件描述符
/// - `buffer`: 用于存储读取数据的缓冲区
/// ## 返回值
/// 返回系统调用的结果
pub fn sys_read(fd: usize, buffer: &mut [u8]) -> SyscallRet {
    syscall(Syscall::Read, [fd, buffer.as_ptr() as usize, buffer.len()])
}

/// 写入文件描述符
///
/// ## 参数
/// - `fd`: 文件描述符
/// - `buffer`: 要写入的数据
/// ## 返回值
/// 返回系统调用的结果
pub fn sys_write(fd: usize, buffer: &[u8]) -> SyscallRet {
    syscall(Syscall::Write, [fd, buffer.as_ptr() as usize, buffer.len()])
}

/// 退出当前进程, 报告返回值
///
/// ## 参数
/// - `code`: 退出码
/// 该函数不会返回，直接退出程序
pub fn sys_exit(code: usize) -> ! {
    syscall(Syscall::Exit, [code, 0, 0]);
    unreachable!("Unreachable after sys_exit");
}

/// 让出 CPU
///
/// ## 返回值
/// 返回系统调用的结果
pub fn sys_sched_yield() -> SyscallRet {
    syscall(Syscall::SchedYield, [0, 0, 0])
}

/// 获取当前时间
///
/// ## 参数
/// - `ts`: 指向 `TimeVal` 结构体的指针，用于存储当前时间
/// - `tz`: 指向 `TimeZone` 结构体的指针，用于存储时区信息
/// ## 返回值
/// 返回系统调用的结果
pub fn sys_get_time_of_day(ts: *mut TimeVal, tz: *mut TimeZone) -> SyscallRet {
    syscall(Syscall::GetTimeOfDay, [ts as usize, tz as usize, 0])
}

/// 关机
///
/// 该函数不会返回，直接关机
pub fn sys_shutdown() -> ! {
    syscall(Syscall::Shutdown, [0, 0, 0]);
    unreachable!("Unreachable after sys_shutdown");
}

/// 创建新进程
///
/// ## 返回值
/// 返回系统调用的结果
pub fn sys_clone() -> SyscallRet {
    syscall(Syscall::Clone, [0, 0, 0])
}

/// 执行新程序
///
/// ## 参数
/// - `path`: 程序路径
/// ## 返回值
/// 返回系统调用的结果
pub fn sys_execve(path: &str, args: Option<&[&str]>) -> SyscallRet {
    syscall(Syscall::Execve, [
        path.as_ptr() as usize,
        args.map(|args| args.as_ptr() as usize).unwrap_or(0),
        args.map(|args| args.len()).unwrap_or(0),
    ])
}

/// 等待子进程退出
///
/// ## 参数
/// - `pid`: 子进程的进程 ID
/// - `exit_code`: 指向 `i32` 类型的指针，用于存储子进程的退出码
/// ## 返回值
/// 返回系统调用的结果
pub fn sys_wait4(pid: isize, exit_code: *mut i32) -> SyscallRet {
    syscall(Syscall::Wait4, [pid as usize, exit_code as usize, 0])
}
