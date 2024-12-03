use core::arch::asm;

use crate::{APP_MANAGER, info, printkln, trap::context::TrapCtx};

pub const MAX_APP_NUM: usize = 64;
/// 与 user-build 中 linker.ld 中的 BASE_ADDRESS 保持一致
pub const APP_BASE_ADDRESS: usize = 0x80400000;
pub const KERNEL_STACK_SIZE: usize = 0x8000;
pub const USER_STACK_SIZE: usize = 0x8000;

#[repr(align(4096))]
struct KernelStack([u8; KERNEL_STACK_SIZE]);

#[repr(align(4096))]
struct UserStack([u8; USER_STACK_SIZE]);

static KERNEL_STACK: KernelStack = KernelStack([0; KERNEL_STACK_SIZE]);
static USER_STACK: UserStack = UserStack([0; USER_STACK_SIZE]);

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
        // let ctx_ptr = &ctx as *const TrapCtx;
        // let ctx_size = core::mem::size_of::<TrapCtx>();
        // let ctx_dst = self.top() - ctx_size;
        // let ctx_src = ctx_ptr;
        // let ctx_dst = ctx_dst as *mut u8;
        // let ctx_src = ctx_src as *const u8;
        // unsafe {
        //     ctx_dst.copy_from(ctx_src, ctx_size);
        // }
        // let ctx_dst = ctx_dst as *mut TrapCtx;
        // unsafe { &mut *ctx_dst }
        let cx_ptr = (self.top() - core::mem::size_of::<TrapCtx>()) as *mut TrapCtx;
        unsafe {
            *cx_ptr = ctx;
        }
        unsafe { cx_ptr.as_mut().unwrap() as *const _ as usize }
    }
}

pub struct AppManager {
    /// 应用程序总数
    pub app_count: usize,
    /// 当前运行的应用程序编号
    pub current: usize,
    /// 应用程序入口地址表
    pub app_table: [usize; MAX_APP_NUM],
    pub app_name_table: [&'static str; MAX_APP_NUM],
    /// 最后一个应用程序的结束地址
    pub apps_end: usize,
}

impl AppManager {
    /// 从应用程序表构造一个 AppManager
    ///
    /// ## 参数
    /// - `app_count`：应用程序总数
    /// - `app_table_ptr`：应用程序入口地址表（的指针）
    ///
    /// ## 应用程序表结构
    /// 应用程序表是一个数组，每个元素是一个应用程序的入口地址（64 位整数）。
    /// ```asm
    /// .global __app_table
    /// __app_table:
    ///    .quad __app_0_start
    ///    .quad __app_1_start
    ///    ; ...
    ///    .quad __app_{n-1}_start
    ///    .quad __app_{n-1}_end
    /// ```
    ///
    pub fn new(
        app_count: usize,
        app_table_ptr: *const usize,
        app_name_table_ptr: *const usize,
    ) -> Self {
        let app_table_data: &[usize] =
            unsafe { core::slice::from_raw_parts(app_table_ptr, app_count) }
                .try_into()
                .unwrap();
        let mut app_table = [0; MAX_APP_NUM];
        app_table[0..app_count].copy_from_slice(app_table_data);

        let apps_end = unsafe {
            core::slice::from_raw_parts(app_table_ptr.add(app_count), 1)
                .get(0)
                .copied()
                .unwrap()
        };

        let app_name_table_data: &[usize] =
            unsafe { core::slice::from_raw_parts(app_name_table_ptr, app_count) }
                .try_into()
                .unwrap();
        let mut app_name_table = [""; MAX_APP_NUM];
        for i in 0..app_count {
            app_name_table[i] = Self::get_app_name(app_name_table_data[i] as *const i8);
        }

        Self {
            app_count,
            current: 0,
            app_table,
            apps_end,
            app_name_table,
        }
    }

    pub fn load_app(&self, app_id: usize) {
        let app_start = self.app_table[app_id];
        let app_end = if app_id + 1 < self.app_count {
            self.app_table[app_id + 1]
        } else {
            self.apps_end
        };
        let app_size = app_end - app_start;
        let app_base = APP_BASE_ADDRESS;
        unsafe { core::slice::from_raw_parts_mut(APP_BASE_ADDRESS as *mut u8, 0x20000).fill(0) };
        let app_src = unsafe { core::slice::from_raw_parts(app_start as *const u8, app_size) };
        let app_dst = unsafe { core::slice::from_raw_parts_mut(app_base as *mut u8, app_size) };
        app_dst.copy_from_slice(app_src);
        unsafe {
            asm!("fence.i");
        }
        // let app_entry = app_base;
        // let app_entry: extern "C" fn() -> ! = unsafe { core::mem::transmute(app_entry) };
        // app_entry();
    }

    pub fn get_app_name(app_name_ptr: *const i8) -> &'static str {
        let app_name = unsafe { core::ffi::CStr::from_ptr(app_name_ptr) };
        let app_name = app_name.to_str().unwrap();
        app_name
    }

    pub fn print_apps_info(&self) {
        for i in 0..self.app_count {
            self.print_app_info(i);
        }
    }

    pub fn print_app_info(&self, app_id: usize) {
        info!(
            "App {} - Name: {}, Start: {:#x}, End: {:#x}",
            app_id,
            self.app_name_table[app_id],
            self.app_table[app_id],
            if app_id + 1 < self.app_count {
                self.app_table[app_id + 1]
            } else {
                self.apps_end
            }
        );
    }
}

pub fn run_next_app() {
    info!("Running next app...");
    let mut app_manager = APP_MANAGER.ref_cell.borrow_mut();
    app_manager.load_app(app_manager.current);
    app_manager.print_app_info(app_manager.current);
    app_manager.current = (app_manager.current + 1) % app_manager.app_count;
    drop(app_manager);
    unsafe extern "C" {
        fn __restore_trap(ctx_ptr: usize);
    }
    // panic!("{}", __restore_trap as usize);
    printkln!("{:#x}", __restore_trap as usize);
    printkln!("{:#x}", USER_STACK.top());
    unsafe {
        let a = KERNEL_STACK.push_ctx(TrapCtx::init_app_context(
            APP_BASE_ADDRESS,
            USER_STACK.top(),
        ));
        // print the context
        let ctx = &*(a as *const TrapCtx);
        printkln!("{:#x}", ctx.sstatus);
        printkln!("{:#x}", ctx.sepc);
        printkln!("{:#x}", ctx.x[2]);

        __restore_trap(a);
    }
    unreachable!();
}
