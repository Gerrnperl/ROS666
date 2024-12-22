//! 这个模块提供 Unix 标准的系统调用接口。
//!
//! 主要功能包括：
//! - 退出当前进程。
//! - 读取文件描述符。
//! - 写入文件描述符。
//! - 创建新进程。
//! - 执行新程序。

use common::syscall::time::TimeVal;

use crate::{sched_yield, sys::time::get_time_of_day, syscall};

/// 退出当前进程
///
/// ## 参数
/// - `code`: 退出码
/// 该函数不会返回，直接退出程序
pub fn exit(code: i32) -> ! {
    syscall::sys_exit(code as usize);
}

/// 读取文件描述符
///
/// ## 参数
/// - `fd`: 文件描述符
/// - `buffer`: 用于存储读取数据的缓冲区
/// ## 返回值
/// 返回读取的字节数
pub fn read(fd: usize, buffer: &mut [u8]) -> isize {
    syscall::sys_read(fd, buffer)
}

/// 写入文件描述符
///
/// ## 参数
/// - `fd`: 文件描述符
/// - `buffer`: 要写入的数据
/// ## 返回值
/// 返回写入的字节数
pub fn write(fd: usize, buffer: &[u8]) -> isize {
    syscall::sys_write(fd, buffer)
}

/// 创建新进程
///
/// ## 返回值
/// 返回新进程的 PID
pub fn fork() -> isize {
    syscall::sys_clone()
}

/// 执行新程序
///
/// ## 参数
/// - `path`: 程序路径
/// ## 返回值
/// 返回执行结果
pub fn execve(path: &str) -> isize {
    syscall::sys_execve(path)
}

/// 休眠指定秒数
///
/// ## 参数
/// - `seconds`: 休眠的秒数
pub fn sleep(seconds: usize) {
    let mut ts = TimeVal {
        tv_sec: 0,
        tv_usec: 0,
    };
    let _ = get_time_of_day(&mut ts, None);
    loop {
        let mut ts_now = TimeVal {
            tv_sec: 0,
            tv_usec: 0,
        };
        let _ = get_time_of_day(&mut ts_now, None);
        if ts_now.tv_sec - ts.tv_sec >= seconds {
            break;
        }
        sched_yield();
    }
}
