use crate::syscall;

pub fn exit(code: i32) -> ! {
    syscall::sys_exit(code as usize);
}

pub fn write(fd: usize, buffer: &[u8]) -> isize {
    syscall::sys_write(fd, buffer)
}
