#!/usr/bin/env bash
# 检查开发环境配置

# 用于记录检查版本号不匹配的次数
failedCount=0

# 该函数检查给定版本是否满足所需的最低版本。
# 参数:
#   $1 - 要检查的版本（格式："name x.y.z"）。
#   $2 - 所需的主版本号。
#   $3 - 所需的次版本号。
#   $4 - 所需的补丁版本号。
#   $5 - 如果版本低于所需版本时显示的错误信息。
# 如果版本低于所需版本，该函数会增加全局变量 `failedCount` 的值。
function minimal_version() {
    # 获取版本号
    local version=$1
    # 从版本号中提取主、次、补丁版本号
    local major=$(echo $version | cut -d ' ' -f 2 | cut -d '.' -f 1)
    local minor=$(echo $version | cut -d ' ' -f 2 | cut -d '.' -f 2)
    local patch=$(echo $version | cut -d ' ' -f 2 | cut -d '.' -f 3)
    # 获取所需的主、次、补丁版本号
    local required_major=$2
    local required_minor=$3
    local required_patch=$4

    # 如果版本低于所需版本，显示错误信息
    if [ $major -lt $required_major ] || ([ $major -eq $required_major ] && [ $minor -lt $required_minor ]) || ([ $major -eq $required_major ] && [ $minor -eq $required_minor ] && [ $patch -lt $required_patch ]); then
        echo -e "$5"
        # 增加错误计数
        failedCount=$((failedCount+1))
    fi
}

# 该函数检查给定命令是否存在。
function command_exists() {
    type "$1" &> /dev/null
}

# 终端输出的颜色定义
RED='\033[0;31m'
YELLOW='\033[0;33m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

# 检查 build-essential 是否已安装
unsatisfied_message="${RED}Error: build-essential is not installed $NC
    install build-essential with:$YELLOW
        sudo apt update
        sudo apt install build-essential $NC"
dpkg -l build-essential &> /dev/null || {
    echo -e "$unsatisfied_message"
    # 增加错误计数
    failedCount=$((failedCount+1))
}

# 检查 rustc 是否已安装
unsatisfied_message="${RED}Error: Rust is not installed or not in \$PATH $NC
    install Rust with:$YELLOW
        curl https://sh.rustup.rs -sSf | sh $NC
    or with mirror site:$YELLOW
        export RUSTUP_DIST_SERVER=https://mirrors.tuna.tsinghua.edu.cn/rustup
        export RUSTUP_UPDATE_ROOT=https://mirrors.tuna.tsinghua.edu.cn/rustup/rustup 
        curl https://sh.rustup.rs -sSf | sh $NC"

command_exists rustc || { 
    echo -e "$unsatisfied_message"; 
    # 增加错误计数
    failedCount=$((failedCount+1))
}

# rustc >= 1.85.0
# Rust 1.85.0 版本 / Rust 2024 版本将于 2025-02-20 进入稳定通道。
# 使用 Rust 的 nightly 版本来编译项目。
_rustc_version=$(rustc --version)
rustc_version=$(echo $_rustc_version | cut -d ' ' -f 2 | cut -d '-' -f 1)
unsatisfied_message="${RED}Error: Rust version >= 1.85.0 is required $NC
    install nightly channel with:$YELLOW
        rustup install nightly
        rustup default nightly $NC"

minimal_version "$rustc_version" 1 85 0 "$unsatisfied_message"

# 检查 rustup 目标 riscv64gc-unknown-none-elf 是否已安装
unsatisfied_message="${RED}Error: Rust target riscv64gc-unknown-none-elf is not installed $NC
    install riscv64gc-unknown-none-elf with:$YELLOW
        rustup target add riscv64gc-unknown-none-elf
        cargo install cargo-binutils
        rustup component add llvm-tools-preview
        rustup component add rust-src $NC"
rustup target list | grep "riscv64gc-unknown-none-elf (installed)" &> /dev/null || {
    echo -e "$unsatisfied_message"
    # 增加错误计数
    failedCount=$((failedCount+1))
}

# 检查 qemu-system-riscv64 是否已安装
unsatisfied_message="For ubuntu >= 23.04, install QEMU (>= 7.0.0) with:$YELLOW
        sudo apt update
        sudo apt install qemu-system 
        sudo apt install qemu-user $NC
    Older versions of ubuntu do not have the required version of QEMU, you need to compile it from source. $YELLOW
        See https://rcore-os.cn/rCore-Tutorial-Book-v3/chapter0/5setup-devel-env.html#qemu
        and https://www.qemu.org/download/#source $NC"

command_exists qemu-system-riscv64 && command_exists qemu-riscv64 || { 
    echo -e "${RED}Error: QEMU is not installed or not in \$PATH $NC
    $unsatisfied_message"; 
    # 增加错误计数
    failedCount=$((failedCount+1))
}

# qemu-system-riscv64 >= 7.0.0
# 检查 qemu-system-riscv64 版本是否满足要求
_qemu_version=$(qemu-system-riscv64 --version | head -n 1)
qemu_version=$(echo $_qemu_version | cut -d ' ' -f 4)
unsatisfied_message="${RED}Error: QEMU version >= 7.0.0 is required $NC
    $unsatisfied_message"
minimal_version "$qemu_version" 7 0 0 "$unsatisfied_message"

# 检查 riscv64-unknown-elf-gdb 是否已安装
unsatisfied_message="${RED}Error: riscv64-unknown-elf-gdb is not installed or not in \$PATH $NC
    install riscv64-unknown-elf-gdb with:$YELLOW
        1. Download https://static.dev.sifive.com/dev-tools/riscv64-unknown-elf-gcc-8.3.0-2020.04.1-x86_64-linux-ubuntu14.tar.gz 
            or other versions from https://github.com/sifive/freedom-tools/releases.
        2. Extract the tarball and add the bin directory to your \$PATH $NC
    see https://rcore-os.cn/rCore-Tutorial-Book-v3/chapter0/5setup-devel-env.html#gdb"
command_exists riscv64-unknown-elf-gdb || { 
    echo -e "$unsatisfied_message"; 
    # 增加错误计数
    failedCount=$((failedCount+1))
}

# 该脚本检查是否满足所有开发环境要求。
# 如果所有要求都满足（failedCount 为 0），脚本以状态码 0 退出。
# 否则，以红色打印错误信息并以状态码 1 退出。
if [ $failedCount -eq 0 ]; then
    exit 0
else
    echo -e "${RED}Some development environment requirements are not satisfied $NC"
    exit 1
fi