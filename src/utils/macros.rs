//! 一些常用的宏定义。

/// 返回一个指向全局符号的指针。
/// 符号必须在二进制文件中定义。
///
/// ```no_run
/// extern_global!(symbol_name)
/// ```
/// ->
/// ```no_run
/// {
///      unsafe extern "C" {
///          fn symbol_name();
///      }
///      symbol_name as *const usize
/// }
/// ```
/// ## Example
/// ```asm
/// .section .data
/// .global data_name
/// data_name:
///    .quad 0x12345678
/// ```
/// ```no_run
/// let data_name: *const usize = extern_global!(data_name);
/// ```
#[macro_export]
macro_rules! extern_global {
    ($symbol_name:ident) => {{
        unsafe extern "C" {
            fn $symbol_name();
        }
        $symbol_name as *const usize
    }};
}
