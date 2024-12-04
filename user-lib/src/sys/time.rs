use common::syscall::time::{TimeVal, TimeZone};

use crate::sys_get_time_of_day;

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
