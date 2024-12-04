#  Rust 内核构建目标二进制文件名
KERNEL_BINARY_NAME := $(shell cargo metadata --no-deps --format-version 1 | jq -r ' \
  . as $$root | \
  .packages[] | \
  select(.manifest_path == ($$root.workspace_root + "/Cargo.toml")) | \
  .name \
')

# Rust 用户程序构建目标二进制文件名
USER_BINARY_NAMES := $(shell cargo metadata --no-deps --format-version 1 | jq -r ' \
  . as $$root | \
  .packages[] | \
  select(.name == "user") | \
  .metadata.applications.order[] \
')

TARGET_DIR := target/riscv64gc-unknown-none-elf
KERNEL_ELF := $(TARGET_DIR)/debug/$(KERNEL_BINARY_NAME)
KERNEL_BIN := $(TARGET_DIR)/debug/$(KERNEL_BINARY_NAME).bin
KERNEL_ELF_RELEASE := $(TARGET_DIR)/release/$(KERNEL_BINARY_NAME)
KERNEL_BIN_RELEASE := $(TARGET_DIR)/release/$(KERNEL_BINARY_NAME).bin
KERNEL_GDB_PORT ?= 25666
USER_GDB_PORT ?= 26666
BOOTLOADER ?= bootloader/rustsbi-qemu-release

# 指定当前构建的用户程序（例如通过 vscode task 传入当前活动文件路径以调试当前用户程序）
ACTIVE_USER_APP?=$(shell echo $(ACTIVE_USER_APP_PATH) | sed -E 's/.*?\/?user\/src\///' | cut -d '/' -f 1)
# 用户程序选项，用于 vscode task 传入当前活动文件路径（若当前活动文件不是用户程序则提供所有用户程序进行选择）
USER_APP_OPTION := $(if $(filter $(ACTIVE_USER_APP),$(USER_BINARY_NAMES)),$(ACTIVE_USER_APP),$(USER_BINARY_NAMES))

.PHONY: all build-kernel.dev build-kernel.release build-users.dev build-users.release build-active-user.dev build-active-user.release launch-qemu-system.dev launch-qemu-system.release launch-qemu-user.dev launch-qemu-user.release launch-qemu-user.release.run gdb-kernel.dev gdb-kernel.release gdb-user.dev check-dev-requirements echo-make-args makefile-test

#region 杂项
makefile-test:
	@echo KERNEL_BINARY_NAME: $(KERNEL_BINARY_NAME)
	@echo USER_BINARY_NAMES: $(USER_BINARY_NAMES)
	@echo ACTIVE_USER_APP: $(ACTIVE_USER_APP)
	@echo BOOTLOADER: $(BOOTLOADER)

# 检查开发环境是否满足要求
check-dev-requirements:
	@bash scripts/check-dev-requirements.sh

# 输出构建相关信息, 用于 vscode 获取构建信息
echo-make-args:
	@echo $($(GET_MAKE_ARG))

#endregion 杂项

#region 构建
# 构建内核 (开发模式)
build-kernel.dev: build-users.dev
	cargo build
	rust-objcopy --strip-all $(KERNEL_ELF) -O binary $(KERNEL_BIN)
# 构建内核 (发布模式)
build-kernel.release: build-users.release
	cargo build --release
	rust-objcopy --strip-all $(KERNEL_ELF_RELEASE) -O binary $(KERNEL_BIN_RELEASE)

# 构建所有用户程序 (开发模式)
build-users.dev:
	cargo build -p user
	for user in $(USER_BINARY_NAMES); do \
		rust-objcopy --strip-all $(TARGET_DIR)/debug/$$user -O binary $(TARGET_DIR)/debug/$$user.bin; \
	done
# 构建所有用户程序 (发布模式)
build-users.release:
	cargo build -p user --release
	for user in $(USER_BINARY_NAMES); do \
		rust-objcopy --strip-all $(TARGET_DIR)/release/$$user -O binary $(TARGET_DIR)/release/$$user.bin; \
	done

# 构建指定用户程序 (开发模式)
build-active-user.dev:
	cargo build -p user --bin $(ACTIVE_USER_APP)
	rust-objcopy --strip-all $(TARGET_DIR)/debug/$(ACTIVE_USER_APP) -O binary $(TARGET_DIR)/debug/$(ACTIVE_USER_APP).bin

# 构建指定用户程序 (发布模式)
build-active-user.release:
	cargo build -p user --bin $(ACTIVE_USER_APP) --release
	rust-objcopy --strip-all $(TARGET_DIR)/release/$(ACTIVE_USER_APP) -O binary $(TARGET_DIR)/release/$(ACTIVE_USER_APP).bin
#endregion 构建

#region 运行
__launch_qemu_startup_log:
	@echo "\033[30m[QEMU launcher] start QEMU with:\033[0m"
	@echo "\033[30m[QEMU launcher]   BOOTLOADER: $(BOOTLOADER)\033[0m"
	@echo "\033[30m[QEMU launcher]   BIN: $(KERNEL_BIN)\033[0m"
	@echo "\033[30m[QEMU launcher]   GDB port: $(KERNEL_GDB_PORT)\033[0m"
	@echo "\033[30m[QEMU launcher]\033[31m Quit QEMU: press [Ctrl-A] and then [x]\033[0m"

# 启动 QEMU 调试内核 (开发模式), 将会在启动后挂起等待 GDB 连接
launch-qemu-system.dev: build-kernel.dev __launch_qemu_startup_log
	@qemu-system-riscv64 \
		-machine virt \
		-nographic \
		-bios $(BOOTLOADER) \
		-device loader,file=$(KERNEL_BIN),addr=0x80200000 \
		-S \
		-gdb tcp::25666
	@echo "\033[30m[QEMU launcher] QEMU has exited\033[0m"

# 启动 QEMU 调试内核 (发布模式)
launch-qemu-system.release: build-kernel.release __launch_qemu_startup_log
	@qemu-system-riscv64 \
		-machine virt \
		-nographic \
		-bios $(BOOTLOADER) \
		-device loader,file=$(KERNEL_BIN_RELEASE),addr=0x80200000 \
		-gdb tcp::25666
	@echo "\033[30m[QEMU launcher] QEMU has exited\033[0m"

# 启动 QEMU 调试用户程序 (开发模式)
launch-qemu-user.dev: build-active-user.dev
	@qemu-riscv64 \
		-g $(USER_GDB_PORT) \
		$(TARGET_DIR)/debug/$(ACTIVE_USER_APP)

# 启动 QEMU 调试用户程序 (发布模式)
launch-qemu-user.release: build-active-user.release
	@qemu-riscv64 \
		-g $(USER_GDB_PORT) \
		$(TARGET_DIR)/release/$(ACTIVE_USER_APP)

# 启动 QEMU 运行用户程序 (发布模式)
launch-qemu-user.release.run: build-active-user.release
	@qemu-riscv64 \
		$(TARGET_DIR)/release/$(ACTIVE_USER_APP)

# 启动 GDB
gdb-kernel.dev:
	@riscv64-unknown-elf-gdb \
		-ex "file $(KERNEL_ELF)" \
		-ex "set arch riscv:rv64" \
		-ex "target remote :$(KERNEL_GDB_PORT)" \

gdb-kernel.release:
	@riscv64-unknown-elf-gdb \
		-ex "file $(KERNEL_ELF_RELEASE)" \
		-ex "set arch riscv:rv64" \
		-ex "target remote :$(KERNEL_GDB_PORT)"

gdb-user.dev:
	@riscv64-unknown-elf-gdb \
		-ex "file $(TARGET_DIR)/debug/$(ACTIVE_USER_APP)" \
		-ex "set arch riscv:rv64" \
		-ex "target remote :$(USER_GDB_PORT)"