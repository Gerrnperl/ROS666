use crate::sched_yield;

pub fn wait(exit_code: &mut i32) -> isize {
    waitpid(/* -1 */ usize::MAX, exit_code)
}

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
