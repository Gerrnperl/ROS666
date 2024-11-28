#!/usr/bin/env bash
# 检查开发环境配置

function minimal_version() {
    local version=$1
    local major=$(echo $version | cut -d ' ' -f 2 | cut -d '.' -f 1)
    local minor=$(echo $version | cut -d ' ' -f 2 | cut -d '.' -f 2)
    local patch=$(echo $version | cut -d ' ' -f 2 | cut -d '.' -f 3)
    local required_major=$2
    local required_minor=$3
    local required_patch=$4

    if [ $major -lt $required_major ] || ([ $major -eq $required_major ] && [ $minor -lt $required_minor ]) || ([ $major -eq $required_major ] && [ $minor -eq $required_minor ] && [ $patch -lt $required_patch ]); then
        echo -e "$5"
        exit 1
    fi
}

function command_exists() {
    type "$1" &> /dev/null
}

RED='\033[0;31m'
YELLOW='\033[0;33m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

# rustc exists
unsatisfied_message="${RED}Rust is not installed or not in \$PATH $NC
    install Rust with:$YELLOW
        curl https://sh.rustup.rs -sSf | sh $NC
    or with mirror site:$YELLOW
        export RUSTUP_DIST_SERVER=https://mirrors.tuna.edu.cn/rustup
        export RUSTUP_UPDATE_ROOT=https://mirrors.tuna.edu.cn/rustup/rustup 
        curl https://sh.rustup.rs -sSf | sh $NC"

command_exists rustc || { echo -e "$unsatisfied_message"; exit 1; }

# rustc >= 1.85.0
# Rust 1.85.0 version / Rust 2024 edition will enter the stable channel on 2025-02-20.
# By then, we need to use the nightly version of Rust to compile the project.
_rustc_version=$(rustc --version)
rustc_version=$(echo $_rustc_version | cut -d ' ' -f 2 | cut -d '-' -f 1)
unsatisfied_message="Rust version >= 1.85.0 is required
    install nightly channel with:$YELLOW
        rustup install nightly
        rustup default nightly $NC"

minimal_version "$rustc_version" 1 85 0 "$unsatisfied_message"

# qemu-system-riscv64 exists
unsatisfied_message="For ubuntu >= 23.04, install QEMU (>= 7.0.0) with:$YELLOW
        sudo apt install qemu-system $NC
    Older versions of ubuntu do not have the required version of QEMU, you need to compile it from source. $YELLOW
        See https://rcore-os.cn/rCore-Tutorial-Book-v3/chapter0/5setup-devel-env.html#qemu
        and https://www.qemu.org/download/#source $NC"

command_exists qemu-system-riscv64 || { 
    echo -e "${RED}QEMU is not installed or not in \$PATH $NC
    $unsatisfied_message"; 
    exit 1;
}

# qemu-system-riscv64 >= 7.0.0
_qemu_version=$(qemu-system-riscv64 --version | head -n 1)
qemu_version=$(echo $_qemu_version | cut -d ' ' -f 4)
unsatisfied_message="${RED}QEMU version >= 7.0.0 is required $NC
    $unsatisfied_message"
minimal_version "$qemu_version" 7 0 0 "$unsatisfied_message"

# riscv64-unknown-elf-gdb exists
unsatisfied_message="${RED}riscv64-unknown-elf-gdb is not installed or not in \$PATH $NC
    install riscv64-unknown-elf-gdb with:$YELLOW
        1. Download https://static.dev.sifive.com/dev-tools/riscv64-unknown-elf-gcc-8.3.0-2020.04.1-x86_64-linux-ubuntu14.tar.gz 
            or other versions from https://github.com/sifive/freedom-tools/releases.
        2. Extract the tarball and add the bin directory to your \$PATH $NC
    see https://rcore-os.cn/rCore-Tutorial-Book-v3/chapter0/5setup-devel-env.html#gdb"
command_exists riscv64-unknown-elf-gdb1 || { echo -e "$unsatisfied_message"; exit 1; }
        
