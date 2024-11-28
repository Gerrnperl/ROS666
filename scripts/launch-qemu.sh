#!/usr/bin/env bash
# 启动 QEMU

if [ "$#" -ne 2 ] && [ "$#" -ne 3 ]; then
    echo "Usage: $0 <bios> <bin> [-dbg]"
    exit 1
fi

bios=$1
bin=$2
dbg=$3

# check if file exists
function check_file() {
    if [ ! -f $1 ]; then
        echo "File $1 not found!"
        exit 1
    fi
}

function startup_log() {
    echo -e "\033[30m[QEMU launcher] start QEMU with:\033[0m"
    echo -e "\033[30m[QEMU launcher]   BIOS: $bios\033[0m"
    echo -e "\033[30m[QEMU launcher]   BIN: $bin\033[0m"
}

function quit_log() {
    echo -e "\033[30m[QEMU launcher] QEMU exited\033[30m"
}

check_file $bios
check_file $bin



if [ "$dbg" != "-dbg" ]; then
    startup_log
    qemu-system-riscv64 \
        -machine virt \
        -nographic \
        -bios $bios \
        -device loader,file=$bin,addr=0x80200000 \
        -gdb tcp::25666
else
    startup_log
    qemu-system-riscv64 \
        -machine virt \
        -nographic \
        -bios $bios \
        -device loader,file=$bin,addr=0x80200000 \
        -S \
        -gdb tcp::25666
fi

STATUS=$?
quit_log
exit $STATUS

