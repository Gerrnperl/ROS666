build-kernel-debug:
	cargo build -p user
	cargo build
build-kernel-release:
	cargo build -p user --release
	cargo build --release
debug-user-app:
	# get the first part of the string after "/path/to/user/src/"
	cargo build -p user --bin $(shell echo $(dir) | sed -E 's/.*?\/?user\/src\///' | cut -d '/' -f 1)
	qemu-riscv64 -g 26666 target/riscv64gc-unknown-none-elf/debug/$(shell echo $(dir) | sed -E 's/.*?\/?user\/src\///' | cut -d '/' -f 1)