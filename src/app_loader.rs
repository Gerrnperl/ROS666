use core::{arch::asm, cell::RefCell};

use lazy_static::lazy_static;

use crate::{
    extern_global, info, task::manager::TaskManager, trap::context::TrapCtx,
    utils::safety::SyncRefCell,
};

pub const MAX_APP_NUM: usize = 64;
/// 与 user-build 中 linker.ld 中的 BASE_ADDRESS 保持一致
pub const USER_BASE_ADDRESS: usize = 0x80400000;
pub const USER_SPACE_SIZE: usize = 0x00200000;

pub const KERNEL_STACK_SIZE: usize = 4096 * 2;
pub const USER_STACK_SIZE: usize = 4096 * 2;

#[repr(align(4096))]
#[derive(Copy, Clone)]
pub struct KernelStack([u8; KERNEL_STACK_SIZE]);

#[repr(align(4096))]
#[derive(Copy, Clone)]
pub struct UserStack([u8; USER_STACK_SIZE]);

pub static mut KERNEL_STACK: [KernelStack; MAX_APP_NUM] =
    [KernelStack([0; KERNEL_STACK_SIZE]); MAX_APP_NUM];
static mut USER_STACK: [UserStack; MAX_APP_NUM] = [UserStack {
    0: [0; USER_STACK_SIZE],
}; MAX_APP_NUM];
trait Stack {
    fn top(&self) -> usize;
}

impl Stack for KernelStack {
    fn top(&self) -> usize {
        self.0.as_ptr() as usize + KERNEL_STACK_SIZE
    }
}

impl Stack for UserStack {
    fn top(&self) -> usize {
        self.0.as_ptr() as usize + USER_STACK_SIZE
    }
}

impl KernelStack {
    fn push_ctx(&self, ctx: TrapCtx) -> usize {
        let ctx_ptr = &ctx as *const TrapCtx;
        let ctx_size = core::mem::size_of::<TrapCtx>();
        let ctx_dst = self.top() - ctx_size;
        let ctx_src = ctx_ptr;
        let ctx_dst = ctx_dst as *mut u8;
        let ctx_src = ctx_src as *const u8;
        unsafe {
            ctx_dst.copy_from(ctx_src, ctx_size);
        }
        let ctx_dst = ctx_dst as *mut TrapCtx;
        unsafe { &mut *ctx_dst as *mut TrapCtx as usize }
    }
}

pub fn get_add_count() -> usize {
    let app_count = extern_global!(__app_count) as *const usize;
    let app_count = unsafe { app_count.read_volatile() };
    app_count
}

pub struct AppData {
    pub app_id: usize,
    pub app_name: &'static str,
    pub data: &'static [u8],
}

pub fn load_app_data(app_id: usize) -> AppData {
    let app_count = get_add_count();
    assert!(app_id < app_count);
    let app_table = extern_global!(__app_table) as *const usize;
    let app_name_table = extern_global!(__app_name_table) as *const usize;
    let app_start = unsafe {
        core::slice::from_raw_parts(app_table.add(app_id), 1)
            .get(0)
            .copied()
            .unwrap()
    };
    let app_end = if app_id + 1 < app_count {
        unsafe {
            core::slice::from_raw_parts(app_table.add(app_id + 1), 1)
                .get(0)
                .copied()
                .unwrap()
        }
    } else {
        extern_global!(__apps_end) as usize
    };
    let app_name_ptr = unsafe {
        core::slice::from_raw_parts(app_name_table.add(app_id), 1)
            .get(0)
            .copied()
            .unwrap()
    };
    let app_name = get_app_name(app_name_ptr as *const i8);
    let app_data =
        unsafe { core::slice::from_raw_parts(app_start as *const u8, app_end - app_start) };
    AppData {
        app_id,
        app_name,
        data: app_data,
    }
}

pub fn get_app_name(app_name_ptr: *const i8) -> &'static str {
    let app_name = unsafe { core::ffi::CStr::from_ptr(app_name_ptr) };
    let app_name = app_name.to_str().unwrap();
    app_name
}

impl AppData {
    pub fn print_info(&self) {
        info!(
            "App {} - Name: {}, Size: {:#x}",
            self.app_id,
            self.app_name,
            self.data.len(),
        );
    }
}

pub fn new_app_ctx(appid: usize) -> usize {
    let base = todo!();
    unsafe {
        KERNEL_STACK[appid].push_ctx(TrapCtx::init_app_context(base, USER_STACK[appid].top()))
    }
}
