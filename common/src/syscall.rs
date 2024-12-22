//! 系统调用

use bitflags::bitflags;

/// 系统调用号
///
/// 为调试方便，应当与 Linux 系统为 RISC-V 架构定义的系统调用号保持一致
///
/// https://gpages.juszkiewicz.com.pl/syscalls-table/syscalls.html
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Syscall {
    /// 打开文件
    ///
    /// [openat(2) — Linux manual page](https://www.man7.org/linux/man-pages/man2/openat.2.html)
    OpenAt = 56,
    /// 关闭文件
    ///
    /// [close(2) — Linux manual page](https://www.man7.org/linux/man-pages/man2/close.2.html)
    Close = 57,
    /// 从文件描述符读取
    ///
    /// [read(2) — Linux manual page](https://www.man7.org/linux/man-pages/man2/read.2.html)
    Read = 63,
    /// 写入文件描述符
    ///
    /// [write(2) — Linux manual page](https://www.man7.org/linux/man-pages/man2/write.2.html)
    Write = 64,
    /// 终止当前进程
    ///
    /// [_exit(2) — Linux manual page](https://www.man7.org/linux/man-pages/man2/exit.2.html)
    Exit = 93,
    /// 让出处理器
    ///
    /// [sched_yield(2) — Linux manual page](https://www.man7.org/linux/man-pages/man2/sched_yield.2.html)
    SchedYield = 124,
    /// 读取当前时间
    ///
    /// 目前只返回系统 uptime, 未实现获取当前时间
    ///
    /// [gettimeofday(2) — Linux manual page](https://www.man7.org/linux/man-pages/man2/gettimeofday.2.html)
    GetTimeOfDay = 169,
    /// 关机 (shutdown)
    ///
    /// [shutdown(2) — Linux manual page](https://www.man7.org/linux/man-pages/man2/shutdown.2.html)
    Shutdown = 210,
    /// 创建子进程 (fork)
    ///
    /// [clone(2) — Linux manual page](https://www.man7.org/linux/man-pages/man2/clone.2.html)
    Clone = 220,
    /// 执行程序 (exec)
    ///
    /// [execve(2) — Linux manual page](https://www.man7.org/linux/man-pages/man2/execve.2.html)
    Execve = 221,
    /// 等待子进程 (waitpid)
    ///
    /// [wait4(2) — Linux manual page](https://www.man7.org/linux/man-pages/man2/wait4.2.html)
    Wait4 = 260,
}

impl core::fmt::Display for Syscall {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<Syscall> for usize {
    /// 将 `Syscall` 枚举转换为 `usize` 类型。
    ///
    /// ## 参数
    /// * `syscall` - 要转换的 `Syscall` 枚举值。
    /// ## 返回值
    /// 返回对应的 `usize` 类型值。
    fn from(syscall: Syscall) -> usize {
        syscall as usize
    }
}

impl From<usize> for Syscall {
    /// 将 `usize` 类型转换为 `Syscall` 枚举。
    ///
    /// ## 参数
    /// * `syscall` - 要转换的 `usize` 类型值。
    /// ## 返回值
    /// 返回对应的 `Syscall` 枚举值。
    fn from(syscall: usize) -> Syscall {
        // 使用 `match` 表达式匹配 `syscall` 的值
        match syscall {
            56 => Syscall::OpenAt,
            57 => Syscall::Close,
            63 => Syscall::Read,
            64 => Syscall::Write,
            93 => Syscall::Exit,
            124 => Syscall::SchedYield,
            169 => Syscall::GetTimeOfDay,
            210 => Syscall::Shutdown,
            220 => Syscall::Clone,
            221 => Syscall::Execve,
            260 => Syscall::Wait4,
            _ => panic!("Unsupported syscall number: {}", syscall),
        }
    }
}

/// 系统调用参数
///
/// 一个长度为 3 的数组，用于传递系统调用的参数。
pub type SyscallArgs = [usize; 3];
/// 系统调用返回值
pub type SyscallRet = isize;

/// 时间相关结构和函数的模块。
pub mod time {
    /// 表示一个包含秒和微秒的时间值。
    #[repr(C)]
    pub struct TimeVal {
        /// 自纪元以来的秒数。
        pub tv_sec: usize,
        /// 自秒开始以来的微秒数。
        pub tv_usec: usize,
    }

    /// 表示一个包含时区偏移和夏令时信息的时区。
    ///
    /// 仅占位，未实现。
    #[repr(C)]
    pub struct TimeZone {
        /// 西格林尼治的分钟数。
        pub tz_minuteswest: i32,
        /// 夏令时校正类型。
        pub tz_dsttime: i32,
    }
}

bitflags! {
    /// 文件打开标志
    pub struct OpenFlags: usize {
        const READONLY  = 0b00000000000;
        const WRITEONLY = 0b00000000001;
        const READWRITE = 0b00000000010;
        const CREATE    = 0b01000000000;
        const TRUNCATE  = 0b10000000000;
    }
}

impl OpenFlags {
    /// 判断是否可读
    pub fn readable(&self) -> bool {
        !self.contains(OpenFlags::WRITEONLY)
    }
    /// 判断是否可写
    pub fn writable(&self) -> bool {
        !self.is_empty()
    }
}
