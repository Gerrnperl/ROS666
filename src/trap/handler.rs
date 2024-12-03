use core::usize;

use common::syscall::Syscall;
use riscv::{
    interrupt::Exception,
    register::{scause, stval},
};

use crate::syscall::syscall;

use super::context::{Riscv64RegAlias, TrapCtx};

#[unsafe(no_mangle)]
pub fn trap_handler(ctx: &mut TrapCtx) -> &mut TrapCtx {
    let scause = scause::read().cause();
    let stval = stval::read();
    match scause {
        scause::Trap::Exception(const { Exception::UserEnvCall as usize }) => {
            ctx.sepc += 4;
            let ret = syscall(Syscall::from(*ctx.a(7)), [*ctx.a(0), *ctx.a(1), *ctx.a(2)]);
            *ctx.a(0) = ret as usize;
        }
        scause::Trap::Exception(const { Exception::StoreFault as usize })
        | scause::Trap::Exception(const { Exception::StorePageFault as usize }) => {
            todo!("Kill the process");
        }
        scause::Trap::Exception(const { Exception::IllegalInstruction as usize }) => {
            todo!("Kill the process");
        }
        _ => {
            panic!("Unhandled trap: {:?}, stval: {:#x}", scause, stval);
        }
    };
    ctx
}
