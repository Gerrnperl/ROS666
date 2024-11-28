#!/usr/bin/env bash
# 构建内核镜像

if [ "$#" -ne 1 ]; then
    echo "Usage: $0 <ELF>"
    exit 1
fi

ELF=$1

# check if file exists
function check_file() {
    if [ ! -f $1 ]; then
        echo "File $1 not found!"
        exit 1
    fi
}

check_file $ELF

rust-objcopy --strip-all $ELF -O binary $ELF.bin