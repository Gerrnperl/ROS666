//! 这个模块包含与时间相关的系统调用接口。
//!
//! 主要功能包括：
//! - 获取当前时间并存储在 `TimeVal` 结构体中。
//! - 可选地获取时区信息并存储在 `TimeZone` 结构体中。

use common::syscall::time::{TimeVal, TimeZone};

use crate::sys_get_time_of_day;

/// 获取当前时间
///
/// ## 参数
/// - `ts`: 指向 `TimeVal` 结构体的指针，用于存储当前时间
/// - `tz`: 可选的 `TimeZone` 结构体，用于存储时区信息
/// ## 返回值
/// 返回系统调用的结果
pub fn get_time_of_day(ts: *mut TimeVal, tz: Option<TimeZone>) -> isize {
    if let Some(mut tz) = tz {
        sys_get_time_of_day(ts, &mut tz)
    } else {
        sys_get_time_of_day(ts, &mut TimeZone {
            tz_minuteswest: 0,
            tz_dsttime: 0,
        })
    }
}
