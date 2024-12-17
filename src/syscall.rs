use common::syscall::{
    Syscall, SyscallArgs, SyscallRet,
    time::{TimeVal, TimeZone},
};

use crate::{
    io::stdio::read_str,
    mm::page_table::{get_mut_translated_byte_slices, get_translated_byte_slices},
    printk,
    task::{self, manager::TaskManager},
    timer::{get_time, get_time_us},
};

pub fn syscall(call: Syscall, args: SyscallArgs) -> SyscallRet {
    match call {
        Syscall::Read => sys_read(args[0], args[1] as *mut u8, args[2]),
        Syscall::Write => sys_write(args[0], args[1] as *const u8, args[2]),
        Syscall::Exit => {
            sys_exit(args[0]);
            0
        }
        Syscall::SchedYield => sys_yield(),
        Syscall::GetTimeOfDay => sys_get_time_of_day(args[0] as *mut _, args[1] as *mut _),
        Syscall::Clone => sys_clone(),
        Syscall::Execve => sys_execve(args[0] as *const u8),
        Syscall::Wait4 => sys_wait4(args[0] as isize, args[1] as *mut i32),
        #[allow(
            unreachable_patterns,
            reason = "we may receive syscall numbers not defined in the enum"
        )]
        _ => panic!("Unsupported syscall: {}", call),
    }
}

const FD_STDIN: usize = 0;
const FD_STDOUT: usize = 1;

pub fn sys_read(fd: usize, buffer: *mut u8, len: usize) -> SyscallRet {
    match fd {
        FD_STDIN => {
            let buffers =
                get_mut_translated_byte_slices(TaskManager::current_user_token(), buffer, len);
            for buffer in buffers {
                read_str(buffer, buffer.len());
            }
            len as SyscallRet
        }
        _ => panic!("Unsupported file descriptor: {}", fd),
    }
}

pub fn sys_write(fd: usize, buffer: *const u8, len: usize) -> SyscallRet {
    match fd {
        FD_STDOUT => {
            let buffers =
                get_translated_byte_slices(TaskManager::current_user_token(), buffer, len);
            for buffer in buffers {
                let s = core::str::from_utf8(buffer).unwrap();
                printk!("{}", s);
            }
            len as SyscallRet
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

pub fn sys_get_time_of_day(ts: *mut TimeVal, tz: *mut TimeZone) -> SyscallRet {
    let ts_buffer = get_translated_byte_slices(
        TaskManager::current_user_token(),
        ts as *const u8,
        core::mem::size_of::<TimeVal>(),
    );
    let ts = unsafe { &mut *(ts_buffer[0].as_ptr() as *mut TimeVal) };
    let tz_buffer = get_translated_byte_slices(
        TaskManager::current_user_token(),
        tz as *const u8,
        core::mem::size_of::<TimeZone>(),
    );
    let tz = unsafe { &mut *(tz_buffer[0].as_ptr() as *mut TimeZone) };
    let time = get_time_us();
    ts.tv_sec = time / 1_000_000;
    ts.tv_usec = time % 1_000_000;
    tz.tz_minuteswest = 0;
    tz.tz_dsttime = 0;
    0
}

pub fn sys_clone() -> SyscallRet {
    todo!()
}

pub fn sys_execve(path: *const u8) -> SyscallRet {
    todo!()
}

pub fn sys_wait4(pid: isize, exit_code: *mut i32) -> SyscallRet {
    todo!()
}
