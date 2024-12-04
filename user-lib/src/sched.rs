pub fn sched_yield() -> isize {
    crate::syscall::sys_sched_yield()
}
