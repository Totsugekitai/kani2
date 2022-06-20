SHELL := bash
.RECIPEPREFIX = >
.ONESHELL:
MAKEFLAGS += --no-builtin-rules --no-builtin-variables

#.SILENT:

export RELEASE ?=
export QEMU ?=
export QEMU_SYSTEM ?= qemu-system-x86_64
export LOG ?= info
export ARCH ?= x64

target_json := kernel/arch/$(ARCH)/$(ARCH).json
build_mode := $(if $(RELEASE),release,debug)
features :=
qemu :=

ifeq ($(QEMU),1)
features += qemu
qemu =-qemu
endif

# ifeq ($(LOG),error)
# features += log_error
# else ifeq ($(LOG),warn)
# features += log_warn
# else ifeq ($(LOG),debug)
# features += log_debug
# else ifeq ($(LOG),info)
# features += log_info
# else ifeq ($(LOG),trace)
# features += log_trace
# else
# features += log_info
# endif

export RUSTFLAGS = -Z emit-stack-sizes
CARGO ?= cargo +nightly
# CARGOFLAGS += -Z build-std=core,alloc -Z build-std-features=compiler-builtins-mem
# CARGOFLAGS += --target $(target_json)
CARGOFLAGS += --features "$(features)"
CARGOFLAGS += $(if $(RELEASE),--release,)

.PHONY: build-kernel
build-kernel:
> cd kernel; $(CARGO) build $(CARGOFLAGS)

.PHONY: build-loader
build-loader:
> cd loader; $(CARGO) build $(CARGOFLAGS)

.PHONY: build
build: build-kernel build-loader

.PHONY: fmt
fmt:
> pushd kernel
> $(CARGO) fmt --all -- --check
> popd; cd loader
> $(CARGO) fmt --all -- --check

.PHONY: clippy
clippy:
> pushd kernel
> $(CARGO) clippy $(CARGOFLAGS) -- -D warnings
> popd; cd loader
> $(CARGO) clippy $(CARGOFLAGS) -- -D warnings

.PHONY: test
test:
> $(CARGO) test $(CARGOFLAGS) --all -- --nocapture

QEMUFLAGS += -drive if=pflash,format=raw,readonly,file=./ovmf/OVMF_CODE.fd \
-drive if=pflash,format=raw,file=./ovmf/OVMF_VARS.fd \
-drive if=ide,file=fat:rw:image,index=0,media=disk \
-serial stdio

.PHONY: run
run:
> cp target/x86_64-kani2-kernel/$(build_mode)/kani2_kernel.elf image/kani2_kernel.elf
> cp target/x86_64-unknown-uefi/$(build_mode)/kani2_loader.efi image/EFI/BOOT/BOOTX64.EFI
> $(QEMU_SYSTEM) \
-drive if=pflash,format=raw,readonly,file=./ovmf/OVMF_CODE.fd \
-drive if=pflash,format=raw,file=./ovmf/OVMF_VARS.fd \
-drive if=ide,file=fat:rw:image,index=0,media=disk \
-serial stdio

.PHONY: debug-run
debug-run:
> $(QEMU_SYSTEM) $(QEMUFLAGS) -no-shutdown -no-reboot -monitor telnet::1234,server,nowait -gdb tcp::12345 -S #-d int

.PHONY: debug-attach
debug-attach:
> gdb -ex 'file target/x86_64-kani2-kernel/$(build_mode)/kani2_kernel.elf' -ex 'target remote localhost:12345'

.PHONY: all
all: build-kernel build-loader run

.PHONY: clean
clean:
> cargo clean
> rm -rf build kani2.map
