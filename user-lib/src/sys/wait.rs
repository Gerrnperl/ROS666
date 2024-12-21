//! 这个模块包含与等待相关的系统调用接口。
//!
//! 主要功能包括：
//! - 等待任意子进程退出并获取其退出码。
//! - 等待指定的子进程退出并获取其退出码。

use crate::sched_yield;

/// 等待任意子进程退出并获取其退出码
///
/// ## 参数
/// - `exit_code`: 指向 `i32` 类型的指针，用于存储子进程的退出码
/// ## 返回值
/// 返回系统调用的结果
pub fn wait(exit_code: &mut i32) -> isize {
    waitpid(/* -1 */ usize::MAX, exit_code)
}

/// 等待指定的子进程退出并获取其退出码
///
/// ## 参数
/// - `pid`: 要等待的子进程的进程 ID
/// - `exit_code`: 指向 `i32` 类型的指针，用于存储子进程的退出码
/// ## 返回值
/// 返回系统调用的结果
pub fn waitpid(pid: usize, exit_code: &mut i32) -> isize {
    loop {
        let ret = crate::syscall::sys_wait4(pid as isize, exit_code);
        if ret == -2 {
            sched_yield();
        } else {
            return ret;
        }
    }
}
