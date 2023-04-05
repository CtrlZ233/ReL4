# Building
TARGET := riscv64gc-unknown-none-elf
MODE := release
ROOT_SERVER_ELF := target/$(TARGET)/$(MODE)/root_server
ROOT_SERVER_BIN := $(ROOT_SERVER_ELF).bin

# Building mode argument
ifeq ($(MODE), release)
	MODE_ARG := --release
endif

# BOARD
BOARD := qemu


# Binutils
OBJDUMP := rust-objdump --arch-name=riscv64
OBJCOPY := rust-objcopy --binary-architecture=riscv64

# Disassembly
DISASM ?= -x

build:  $(ROOT_SERVER_BIN)


$(ROOT_SERVER_BIN): root_server
	@$(OBJCOPY) $(ROOT_SERVER_ELF) --strip-all -O binary $@

root_server:
	@echo Platform: $(BOARD)
	@cargo build $(MODE_ARG)

clean:
	@cargo clean


.PHONY: build root_server clean
