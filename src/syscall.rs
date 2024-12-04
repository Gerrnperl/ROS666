use common::syscall::{Syscall, SyscallArgs, SyscallRet};

use crate::{printk, task};

pub fn syscall(call: Syscall, args: SyscallArgs) -> SyscallRet {
    match call {
        Syscall::Read => todo!(),
        Syscall::Write => sys_write(args[0], args[1] as *const u8, args[2]),
        Syscall::Exit => {
            sys_exit(args[0]);
            0
        }
        Syscall::SchedYield => sys_yield(),
        #[allow(
            unreachable_patterns,
            reason = "we may receive syscall numbers not defined in the enum"
        )]
        _ => panic!("Unsupported syscall: {}", call),
    }
}

const FD_STDOUT: usize = 1;

pub fn sys_write(fd: usize, buffer: *const u8, len: usize) -> SyscallRet {
    match fd {
        FD_STDOUT => {
            let buffer = unsafe { core::slice::from_raw_parts(buffer, len) };
            let s = core::str::from_utf8(buffer).unwrap();
            printk!("{}", s);
            buffer.len() as SyscallRet
        }
        _ => panic!("Unsupported file descriptor: {}", fd),
    }
}

pub fn sys_exit(code: usize) {
    printk!("Process exited with code {}\n", code);
    task::manager::TaskManager::replace_to_next();
}

pub fn sys_yield() -> SyscallRet {
    task::manager::TaskManager::cycle_to_next();
    0
}
