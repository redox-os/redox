# Configuration
ARCH?=x86_64

# Per host variables
UNAME := $(shell uname)
ifeq ($(UNAME),Darwin)
	ECHO=/bin/echo
	FUMOUNT=sudo umount
	export LD=$(ARCH)-elf-ld
	export LDFLAGS=--gc-sections
	export NPROC=sysctl -n hw.ncpu
	export STRIP=$(ARCH)-elf-strip
	VB_AUDIO=coreaudio
	VBM="/Applications/VirtualBox.app/Contents/MacOS/VBoxManage"
	export GCC="x86_64-elf-gcc"
else
	ECHO=echo
	ifeq ($(FUMOUNT),)
		FUMOUNT=fusermount -u
	endif
	export LD=ld
	export LDFLAGS=--gc-sections
	export NPROC=nproc
	export STRIP=strip
	VB_AUDIO="pulse"
	VBM=VBoxManage
	ifeq ($(GCC),)
		export GCC="gcc"
	endif
endif


# Automatic variables
ROOT=$(PWD)
export RUST_TARGET_PATH=$(ROOT)/kernel/targets
export CC=$(ROOT)/libc-artifacts/gcc.sh
export CFLAGS=-fno-stack-protector -U_FORTIFY_SOURCE

# Kernel variables
KTARGET=$(ARCH)-unknown-none
KBUILD=build/kernel
KRUSTC=./krustc.sh
KRUSTDOC=./krustdoc.sh
KCARGO=RUSTC="$(KRUSTC)" RUSTDOC="$(KRUSTDOC)" CARGO_INCREMENTAL=1 cargo
KCARGOFLAGS=--target $(KTARGET) --release -- -C soft-float

# Userspace variables
export TARGET=$(ARCH)-unknown-redox
BUILD=build/userspace
export INITFS_FOLDER=$(ROOT)/initfs
RUSTC=$(PWD)/rustc.sh
RUSTDOC=./rustdoc.sh
CARGO=RUSTC="$(RUSTC)" RUSTDOC="$(RUSTDOC)" CARGO_INCREMENTAL=1 cargo
CARGOFLAGS=--target $(TARGET) --release -- -C codegen-units=`$(NPROC)`
