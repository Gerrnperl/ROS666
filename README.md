# startup-test

A attempt to create a simple os.

## Reference

| Project | License |
| --- | --- |
| [rCore-Tutorial-v3](https://github.com/rcore-os/rCore-Tutorial-v3) | GPL-3.0 |
| [rCore](https://github.com/rcore-os/rCore) | MIT |

## Requirements

项目构建运行配置针对 Linux (Ubuntu) 系统配置，未在其他系统上测试。建议使用 WSL2 或 VMWare / VirtualBox 等虚拟机运行 Ubuntu 系统。

Ubuntu 版本建议在 **24.04 noble** 及以上版本，避免从源码编译 QEMU。

运行 `scripts/check-dev-requirements.sh` 检查开发环境是否满足要求。

### WSL 

参阅:

[Install Ubuntu on WSL2](https://documentation.ubuntu.com/wsl/en/latest/guides/install-ubuntu-wsl2/)

[Ubuntu 24.04.1 LTS | Microsoft Store](https://apps.microsoft.com/detail/9nz3klhxdjp5)

[Developing in WSL with Visual Studio Code](https://code.visualstudio.com/docs/remote/wsl)

### Rust 安装

<img src="https://pic4.zhimg.com/v2-4f3c8ea0b71e54ea4aeb98c1747ae81b_xld.png" width="100px" alt="rust"
title="&#x4F60;&#x8BF4;&#x5F97;&#x5BF9;&#xFF0C;&#x4F46;&#x662F; Rust &#x662F;&#x7531; Mozilla &#x81EA;&#x4E3B;&#x7814;&#x53D1;&#x7684;&#x4E00;&#x6B3E;&#x5168;&#x65B0;&#x7684;&#x7F16;&#x8BD1;&#x671F;&#x683C;&#x6597;&#x6E38;&#x620F;&#x3002;&#x7F16;&#x8BD1;&#x5C06;&#x53D1;&#x751F;&#x5728;&#x4E00;&#x4E2A;&#x88AB;&#x79F0;&#x4F5C;&#x300C;Cargo&#x300D;&#x7684;&#x6784;&#x5EFA;&#x7CFB;&#x7EDF;&#x4E2D;&#x3002;&#x5728;&#x8FD9;&#x91CC;&#xFF0C;&#x88AB;&#x5F15;&#x7528;&#x7684;&#x6307;&#x9488;&#x5C06;&#x88AB;&#x6388;&#x4E88;&#x300C;&#x751F;&#x547D;&#x5468;&#x671F;&#x300D;&#x4E4B;&#x529B;&#xFF0C;&#x5BFC;&#x5F15;&#x5BF9;&#x8C61;&#x5B89;&#x5168;&#x3002;&#x4F60;&#x5C06;&#x626E;&#x6F14;&#x4E00;&#x4F4D;&#x540D;&#x4E3A;&#x300C;Rustacean&#x300D;&#x7684;&#x795E;&#x79D8;&#x89D2;&#x8272;, &#x5728;&#x4E0E;&#x300C;Rustc&#x300D;&#x7684;&#x640F;&#x6597;&#x4E2D;&#x9082;&#x9005;&#x5404;&#x79CD;&#x9AA8;&#x9ABC;&#x60CA;&#x5947;&#x7684;&#x50B2;&#x5A07;&#x62A5;&#x9519;&#x3002;&#x5F81;&#x670D;&#x5B83;&#x4EEC;&#x3001;&#x901A;&#x8FC7;&#x7F16;&#x8BD1;&#x540C;&#x65F6;&#xFF0C;&#x9010;&#x6B65;&#x53D1;&#x6398;&#x300C;C++&#x300D;&#x7A0B;&#x5E8F;&#x5D29;&#x6E83;&#x7684;&#x771F;&#x76F8;.">

#### 检查 Rust 是否安装

确保系统中已安装 Rust，并且 `rustc` 命令在 `$PATH` 中可用。运行以下命令进行检查：

```shell
rustc --version
```

如果未安装 Rust 或 `rustc` 不在 `$PATH` 中，请按照以下步骤安装 Rust。

#### 安装 Rust

您可以通过以下命令安装 Rust：

```shell
curl https://sh.rustup.rs -sSf | sh
```

或者使用镜像站点安装：

```shell
export RUSTUP_DIST_SERVER=https://mirrors.tuna.edu.cn/rustup
export RUSTUP_UPDATE_ROOT=https://mirrors.tuna.edu.cn/rustup/rustup
curl https://sh.rustup.rs -sSf | sh
```

#### 检查 Rust 版本

Rust 1.85.0 版本 / Rust 2024 版本将于 2025-02-20 进入 `stable` 通道。在此之前，我们可能需要使用 Rust 的 `nightly` 版本来编译项目。

确保 Rust 版本 >= 1.85.0。运行以下命令检查版本：

```shell
rustc --version
```

如果版本不满足要求，请安装 `nightly` 版本：

```shell
rustup install nightly
rustup default nightly
```

### QEMU 安装

#### 检查 QEMU 是否安装

确保您的系统中已安装 `qemu-system-riscv64`，并且该命令在 `$PATH` 中可用。运行以下命令进行检查：

```shell
qemu-system-riscv64 --version
```

如果未安装 QEMU 或 `qemu-system-riscv64` 不在 `$PATH` 中，请按照以下步骤安装 QEMU。

#### 安装 QEMU

对于 Ubuntu >= 23.04，可以通过以下命令安装 QEMU（>= 7.0.0）：

```shell
sudo apt install qemu-system
```

对于旧版本的 Ubuntu，您需要从源码编译 QEMU。请参考以下链接：

- [rCore-Tutorial-Book-v3](https://rcore-os.cn/rCore-Tutorial-Book-v3/chapter0/5setup-devel-env.html#qemu)
- [QEMU 官方下载页面](https://www.qemu.org/download/#source)

#### 检查 QEMU 版本

确保 QEMU 版本 >= 7.0.0。运行以下命令检查版本：

```shell
qemu-system-riscv64 --version
```

### riscv64-unknown-elf-gdb 安装

#### 检查 riscv64-unknown-elf-gdb 是否安装

确保您的系统中已安装 `riscv64-unknown-elf-gdb`，并且该命令在 `$PATH` 中可用。运行以下命令进行检查：

```shell
riscv64-unknown-elf-gdb --version
```

如果未安装 `riscv64-unknown-elf-gdb` 或该命令不在 `$PATH` 中，请按照以下步骤安装。

#### 安装 riscv64-unknown-elf-gdb

1. 下载 [riscv64-unknown-elf-gcc-8.3.0-2020.04.1-x86_64-linux-ubuntu14.tar.gz](https://static.dev.sifive.com/dev-tools/riscv64-unknown-elf-gcc-8.3.0-2020.04.1-x86_64-linux-ubuntu14.tar.gz) 或从 [GitHub releases](https://github.com/sifive/freedom-tools/releases) 下载其他版本。
2. 解压 tar 包，并将 `bin` 目录添加到您的 `$PATH` 中。

更多详细信息请参考 [rCore-Tutorial-Book-v3](https://rcore-os.cn/rCore-Tutorial-Book-v3/chapter0/5setup-devel-env.html#gdb)。
