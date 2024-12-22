//! 调度相关的系统调用接口。

/// 让出 CPU 给其他进程
///
/// ## 返回值
/// 返回系统调用的结果
pub fn sched_yield() -> isize {
    crate::syscall::sys_sched_yield()
}
