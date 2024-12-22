//! 异常处理模块。

pub mod context;
pub mod handler;
pub mod init;
/// 初始化模块。
///
/// 这个模块提供了一个 `init` 函数，用于初始化系统或应用程序的某些部分。
/// # 示例
/// ```rust
/// use crate::trap::init;
/// fn main() {
///     init();
/// }
/// ```
/// # 注意
/// 确保在调用 `init` 函数之前，所有必要的依赖项都已正确配置。
pub use init::init;
