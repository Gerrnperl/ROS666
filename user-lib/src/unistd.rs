use crate::syscall;

pub fn exit(code: i32) -> ! {
    syscall::sys_exit(code as usize);
}

pub fn read(fd: usize, buffer: &mut [u8]) -> isize {
    syscall::sys_read(fd, buffer)
}

pub fn write(fd: usize, buffer: &[u8]) -> isize {
    syscall::sys_write(fd, buffer)
}

pub fn fork() -> isize {
    syscall::sys_clone()
}

pub fn execve(path: &str) -> isize {
    syscall::sys_execve(path)
}
