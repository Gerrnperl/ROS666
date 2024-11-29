#[macro_export]
macro_rules! error {
    ($fmt:expr) => {
        $crate::printk!("\x1b[1;31m[ERROR]\x1b[0m {}\n", $fmt)
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::printk!("\x1b[1;31m[ERROR]\x1b[0m {}\n", format_args!($fmt, $($arg)*))
    };
}

#[macro_export]
macro_rules! warn {
    ($fmt:expr) => {
        $crate::printk!("\x1b[1;33m[WARN]\x1b[0m {}\n", $fmt)
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::printk!("\x1b[1;33m[WARN]\x1b[0m {}\n", format_args!($fmt, $($arg)*))
    };
}

#[macro_export]
macro_rules! info {
    ($fmt:expr) => {
        $crate::printk!("\x1b[1;32m[INFO]\x1b[0m {}\n", $fmt)
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::printk!("\x1b[1;32m[INFO]\x1b[0m {}\n", format_args!($fmt, $($arg)*))
    };
}

#[macro_export]
macro_rules! debug {
    ($fmt:expr) => {
        $crate::printk!("\x1b[1;34m[DEBUG]\x1b[0m {}\n", $fmt)
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::printk!("\x1b[1;34m[DEBUG]\x1b[0m {}\n", format_args!($fmt, $($arg)*))
    };
}

#[macro_export]
macro_rules! trace {
    ($fmt:expr) => {
        $crate::printk!("\x1b[1;35m[TRACE]\x1b[0m {}\n", $fmt)
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::printk!("\x1b[1;35m[TRACE]\x1b[0m {}\n", format_args!($fmt, $($arg)*))
    };
}

pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

#[macro_export]
macro_rules! log {
    ($level:expr, $fmt:expr) => {
        match $level {
            $crate::LogLevel::Error => $crate::error!($fmt),
            $crate::LogLevel::Warn => $crate::warn!($fmt),
            $crate::LogLevel::Info => $crate::info!($fmt),
            $crate::LogLevel::Debug => $crate::debug!($fmt),
            $crate::LogLevel::Trace => $crate::trace!($fmt),
        }
    };
    ($level:expr, $fmt:expr, $($arg:tt)*) => {
        match $level {
            $crate::LogLevel::Error => $crate::error!($fmt, $($arg)*),
            $crate::LogLevel::Warn => $crate::warn!($fmt, $($arg)*),
            $crate::LogLevel::Info => $crate::info!($fmt, $($arg)*),
            $crate::LogLevel::Debug => $crate::debug!($fmt, $($arg)*),
            $crate::LogLevel::Trace => $crate::trace!($fmt, $($arg)*),
        }
    };
}
