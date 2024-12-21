//! 日志模块。

#[macro_export]
/// 生成一个错误日志消息，并将其打印到控制台。
///
/// ## 参数
/// - `$fmt`: 格式化字符串，用于指定错误消息的格式。
/// - `$($arg:tt)*`: 可选参数，用于格式化字符串中的占位符。
/// ## 示例
/// ```rust
/// error!("发生了一个错误");
/// error!("错误代码: {}", 404);
/// ```
/// 以上示例将会在控制台输出带有红色 "[ERROR]" 标签的错误消息。
macro_rules! error {
    ($fmt:expr) => {
        $crate::printk!("\x1b[1;31m[ERROR]\x1b[0m {}\n", $fmt)
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::printk!("\x1b[1;31m[ERROR]\x1b[0m {}\n", format_args!($fmt, $($arg)*))
    };
}

#[macro_export]
/// `warn!` 宏用于打印带有黄色警告标签的日志消息。
///
/// # 用法
/// ```
/// warn!("This is a warning message.");
/// warn!("This is a warning message with arguments: {}", 42);
/// ```
/// # 参数
/// - `$fmt`: 格式化字符串，用于指定日志消息的内容。
/// - `$($arg:tt)*`: 可选参数，用于格式化字符串中的占位符。
/// # 示例
/// ```
/// warn!("Disk space is low.");
/// warn!("User {} has logged in.", "Alice");
/// ```
macro_rules! warn {
    ($fmt:expr) => {
        $crate::printk!("\x1b[1;33m[WARN]\x1b[0m {}\n", $fmt)
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::printk!("\x1b[1;33m[WARN]\x1b[0m {}\n", format_args!($fmt, $($arg)*))
    };
}

#[macro_export]
/// 记录信息级别的日志消息。
///
/// 这个宏提供了两种用法:
/// 1. 不带参数的格式化字符串:
/// ```
/// info!("这是一个信息日志");
/// ```
/// 2. 带参数的格式化字符串:
/// ```
/// info!("这是一个带参数的信息日志: {}", "参数值");
/// ```
/// 宏会在日志消息前添加绿色的 `[INFO]` 标签。
/// ## 参数
/// - `fmt`: 格式化字符串。
/// - `arg`: 可选的格式化参数。
macro_rules! info {
    ($fmt:expr) => {
        $crate::printk!("\x1b[1;32m[INFO]\x1b[0m {}\n", $fmt)
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::printk!("\x1b[1;32m[INFO]\x1b[0m {}\n", format_args!($fmt, $($arg)*))
    };
}

#[macro_export]
/// 打印调试信息的宏。
///
/// ## 参数
/// - `$fmt`: 格式化字符串。
/// - `$($arg:tt)*`: 可选的格式化参数。
/// ## 示例
/// ```rust
/// debug!("这是一个调试信息");
/// debug!("这是一个带参数的调试信息: {}", 42);
/// ```
/// 该宏会在输出中添加 `[DEBUG]` 标签，并使用蓝色高亮显示。
macro_rules! debug {
    ($fmt:expr) => {
        $crate::printk!("\x1b[1;34m[DEBUG]\x1b[0m {}\n", $fmt)
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::printk!("\x1b[1;34m[DEBUG]\x1b[0m {}\n", format_args!($fmt, $($arg)*))
    };
}

#[macro_export]
/// 这个宏用于打印跟踪级别的日志消息。
///
/// ## 用法
/// ```
/// trace!("这是一个简单的跟踪消息");
/// trace!("这是一个带参数的跟踪消息: {}", 42);
/// ```
/// ## 参数
/// - `$fmt`: 格式化字符串，用于描述日志消息。
/// - `$($arg:tt)*`: 可选的参数，用于格式化字符串中的占位符。
/// ## 输出
/// 该宏会输出带有紫色 `[TRACE]` 标签的日志消息。
/// ## 示例
/// ```
/// trace!("系统启动");
/// trace!("用户 {} 登录", "Alice");
/// ```
macro_rules! trace {
    ($fmt:expr) => {
        $crate::printk!("\x1b[1;35m[TRACE]\x1b[0m {}\n", $fmt)
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::printk!("\x1b[1;35m[TRACE]\x1b[0m {}\n", format_args!($fmt, $($arg)*))
    };
}

/// 日志级别枚举。
///
/// 该枚举定义了不同的日志级别，用于控制日志的输出。
/// ## 变体
/// - `Error`: 错误级别日志，用于记录错误信息。
/// - `Warn`: 警告级别日志，用于记录警告信息。
/// - `Info`: 信息级别日志，用于记录一般信息。
/// - `Debug`: 调试级别日志，用于记录调试信息。
/// - `Trace`: 跟踪级别日志，用于记录详细的跟踪信息。
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

#[macro_export]
/// `log!` 宏用于根据日志级别记录日志信息。
///
/// ## 参数
/// * `$level` - 日志级别，类型为 `$crate::LogLevel`。
/// * `$fmt` - 格式化字符串，与 `format!` 宏的格式化字符串类似。
/// * `$($arg:tt)*` - 可选的格式化参数。
/// ## 用法
/// ```rust
/// log!(LogLevel::Error, "This is an error message");
/// log!(LogLevel::Info, "This is an info message with a value: {}", 42);
/// ```
/// 根据传入的日志级别，宏会调用相应的日志记录宏（如 `error!`、`warn!`、`info!`、`debug!`、`trace!`）。
/// ## 示例
/// ```rust
/// log!(LogLevel::Warn, "This is a warning message");
/// log!(LogLevel::Debug, "Debugging value: {:?}", some_value);
/// ```
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
