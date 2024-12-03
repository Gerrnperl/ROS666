use core::arch::asm;

pub const MAX_APP_NUM: usize = 64;
/// 与 user-build 中 linker.ld 中的 BASE_ADDRESS 保持一致
pub const APP_BASE_ADDRESS: usize = 0x80400000;

pub struct AppManager {
    /// 应用程序总数
    pub app_count: usize,
    /// 当前运行的应用程序编号
    pub current: usize,
    /// 应用程序入口地址表
    pub app_table: [usize; MAX_APP_NUM],
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
    pub fn new(app_count: usize, app_table_ptr: *const usize) -> Self {
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

        Self {
            app_count,
            current: 0,
            app_table,
            apps_end,
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
        let app_src = unsafe { core::slice::from_raw_parts(app_start as *const u8, app_size) };
        let app_dst = unsafe { core::slice::from_raw_parts_mut(app_base as *mut u8, app_size) };
        app_dst.copy_from_slice(app_src);
        unsafe {
            asm!("fence.i");
        }
        let app_entry = app_base;
        let app_entry: extern "C" fn() -> ! = unsafe { core::mem::transmute(app_entry) };
        app_entry();
    }
}
