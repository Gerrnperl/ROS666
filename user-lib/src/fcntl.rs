pub use common::syscall::OpenFlags;

use crate::{sys_close, sys_openat};

pub fn openat(path: &str, flags: OpenFlags) -> i32 {
    let ret = sys_openat(path, flags.bits() as usize);
    ret as i32
}

pub fn close(fd: i32) -> i32 {
    let ret = sys_close(fd as usize);
    ret as i32
}
