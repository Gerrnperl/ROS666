# Rust 内联汇编 - RISC-V

[Inline Assembly - The Rust Reference](https://doc.rust-lang.org/reference/inline-assembly.html)

在 Rust 中使用内联汇编可以通过 `asm!` 宏来实现。内联汇编是一种直接在 Rust 代码中嵌入汇编代码的方法。内联汇编可以用于访问处理器的特定功能，或者执行无法用 Rust 代码表示的操作。

## 使用

```rust
#![feature(asm)]

fn main() {
    unsafe {
        asm!(
            "nop",
            "nop",
        );
    }
}
```

## 寄存器操作数 [[asm.register-operands]](https://doc.rust-lang.org/reference/inline-assembly.html#r-asm.register-operands)

输入和输出操作数可以指定为显式寄存器或寄存器类别，寄存器分配器可以从中选择寄存器。显式寄存器以字符串字面量形式指定（例如 "eax"），而寄存器类别以标识符形式指定（例如 reg）。

使用相同的显式寄存器作为两个输入操作数或两个输出操作数会导致编译时错误。此外，在输入操作数或输出操作数中使用重叠寄存器（例如 ARM VFP）也会导致编译时错误。

只有以下类型允许作为内联汇编的操作数：

- 整数（有符号和无符号）
- 浮点数
- 指针（仅限 thin 指针）
- 函数指针
- SIMD 向量（使用 `#[repr(simd)]` 定义的结构体，并且实现了 `Copy`）。这包括 `std::arch` 中定义的体系结构特定的向量类型

以下是当前支持的寄存器类别列表：

| 架构 | 类别 | 寄存器 | 说明 |
| --- | --- | --- | --- |
| RISC-V | reg | x1, x[5-7], x[9-15], x[16-31] (non-RV32E) | r |
| RISC-V | freg | f[0-31] | f |
| RISC-V | vreg | v[0-31] | Only clobbers |

## 寄存器别名 [[asm.register-names.supported-register-aliases]](https://doc.rust-lang.org/reference/inline-assembly.html#r-asm.register-names.supported-register-aliases)

一些寄存器有别名。这些都被编译器视为与基本寄存器名称相同。

| 架构 | 基本寄存器 | 别名 |
| --- | --- | --- |
| RISC-V | x0 | zero |
| RISC-V | x1 | ra |
| RISC-V | x2 | sp |
| RISC-V | x3 | gp |
| RISC-V | x4 | tp |
| RISC-V | x[5-7] | t[0-2] |
| RISC-V | x8 | fp, s0 |
| RISC-V | x9 | s1 |
| RISC-V | x[10-17] | a[0-7] |
| RISC-V | x[18-27] | s[2-11] |
| RISC-V | x[28-31] | t[3-6] |
| RISC-V | f[0-7] | ft[0-7] |
| RISC-V | f[8-9] | fs[0-1] |
| RISC-V | f[10-17] | fa[0-7] |
| RISC-V | f[18-27] | fs[2-11] |
| RISC-V | f[28-31] | ft[8-11] |