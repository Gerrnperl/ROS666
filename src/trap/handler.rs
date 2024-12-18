use core::{arch::asm, usize};

use common::syscall::Syscall;
use riscv::{
    interrupt::{Exception, supervisor::Interrupt},
    register::{
        scause, stval,
        stvec::{self, TrapMode},
    },
};

use crate::{
    extern_global,
    mm::memory_set::TRAMPOLINE,
    printk,
    syscall::syscall,
    task::{manager::TaskManager, processor::Processor, task::TRAP_CONTEXT},
    timer::set_next_timeout,
};

use super::context::{Riscv64RegAlias, TrapCtx};

pub const TIMER_INTERVAL_USEC: usize = 10_000;

#[unsafe(no_mangle)]
pub fn trap_handler() -> ! {
    set_user_trap_entry();
    let scause = scause::read().cause();
    let stval = stval::read();
    match scause {
        scause::Trap::Exception(const { Exception::UserEnvCall as usize }) => {
            let ctx = Processor::current_trap_cx().unwrap();
            ctx.sepc += 4;
            let ret = syscall(Syscall::from(*ctx.a(7)), [*ctx.a(0), *ctx.a(1), *ctx.a(2)]);
            // ctx 在 syscall 之后会被修改，所以这里需要重新获取
            let ctx = Processor::current_trap_cx().unwrap();
            *ctx.a(0) = ret as usize;
        }
        scause::Trap::Exception(const { Exception::StoreFault as usize })
        | scause::Trap::Exception(const { Exception::StorePageFault as usize }) => {
            todo!("Kill the process");
        }
        scause::Trap::Exception(const { Exception::IllegalInstruction as usize }) => {
            todo!("Kill the process");
        }
        scause::Trap::Interrupt(const { Interrupt::SupervisorTimer as usize }) => {
            // todo!("Timer interrupt");
            set_next_timeout(TIMER_INTERVAL_USEC);
            TaskManager::cycle_to_next();
        }
        _ => {
            panic!("Unhandled trap: {:?}, stval: {:#x}", scause, stval);
        }
    };
    trap_return();
}

fn set_user_trap_entry() {
    unsafe {
        stvec::write(TRAMPOLINE as usize, TrapMode::Direct);
    }
}

#[unsafe(no_mangle)]
pub fn trap_return() -> ! {
    set_user_trap_entry();
    let trap_cx_ptr = TRAP_CONTEXT;
    let user_satp = Processor::current_user_token().unwrap();
    let restore_va =
        extern_global!(__restore_trap) as usize - extern_global!(__save_trap) as usize + TRAMPOLINE;
    unsafe {
        asm!(
            "fence.i",
            "jr {restore_va}",             // jump to new addr of __restore asm function
            restore_va = in(reg) restore_va,
            in("a0") trap_cx_ptr,      // a0 = virt addr of Trap Context
            in("a1") user_satp,        // a1 = phy addr of usr page table
            options(noreturn)
        );
    }
}

#[unsafe(no_mangle)]
pub fn trap_from_kernel() -> ! {
    panic!("A trap from kernel occurs!");
}
