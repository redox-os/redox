-include .config

# Configuration
## Architecture to build Redox for (aarch64 or x86_64)
ARCH?=x86_64
## Flags to pass to the installer (empty to download binary packages)
INSTALLER_FLAGS?=--cookbook=cookbook
## Enabled to use binary prefix (much faster)
PREFIX_BINARY?=1
## Filesystem size in MB (256 is the default)
FILESYSTEM_SIZE?=256
## Flags to pass to redoxfs-mkfs. Add --encrypt to set up disk encryption
REDOXFS_MKFS_FLAGS?=

# Per host variables
# TODO: get host arch automatically
HOST_ARCH=x86_64
HOST_CARGO=env --unset=RUSTUP_TOOLCHAIN cargo
UNAME := $(shell uname)
ifeq ($(UNAME),Darwin)
	FUMOUNT=sudo umount
	export NPROC=sysctl -n hw.ncpu
	export REDOX_MAKE=make
	PREFIX_BINARY=0
	VB_AUDIO=coreaudio
	VBM=/Applications/VirtualBox.app/Contents/MacOS/VBoxManage
	HOST_TARGET ?= $(HOST_ARCH)-apple-darwin
else ifeq ($(UNAME),FreeBSD)
	FUMOUNT=sudo umount
	export NPROC=sysctl -n hw.ncpu
	export REDOX_MAKE=gmake
	PREFIX_BINARY=0
	VB_AUDIO=pulse # To check, will probaly be OSS on most setups
	VBM=VBoxManage
	HOST_TARGET ?= $(HOST_ARCH)-unknown-freebsd
else
	FUMOUNT=fusermount -u
	export NPROC=nproc
	export REDOX_MAKE=make
	VB_AUDIO=pulse
	VBM=VBoxManage
	HOST_TARGET ?= $(HOST_ARCH)-unknown-linux-gnu
endif

# Automatic variables
ROOT=$(CURDIR)
export RUST_COMPILER_RT_ROOT=$(ROOT)/rust/src/llvm-project/compiler-rt
export XARGO_RUST_SRC=$(ROOT)/rust/src

## Userspace variables
export TARGET=$(ARCH)-unknown-redox
BUILD=build/userspace
INSTALLER=\
	export PATH="$(PREFIX_PATH):$$PATH" && \
 	installer/target/release/redox_installer $(INSTALLER_FLAGS)

## Bootloader variables
ifeq ($(ARCH),x86_64)
	BOOTLOADER_EFI_PATH=efi/boot/bootx64.efi
	BOOTLOADER_TARGET=x86-unknown-none
else ifeq ($(ARCH),i686)
	BOOTLOADER_EFI_PATH=efi/boot/bootia32.efi
	BOOTLOADER_TARGET=x86-unknown-none
else ifeq ($(ARCH),aarch64)
	BOOTLOADER_EFI_PATH=efi/boot/bootaa64.efi
	BOOTLOADER_TARGET=aarch64-unknown-none
else
$(error Unsupported ARCH for bootloader "$(ARCH)")
endif
EFI_TARGET=$(ARCH)-unknown-uefi
PARTED=/sbin/parted

## Cross compiler variables
AR=$(TARGET)-gcc-ar
AS=$(TARGET)-as
CC=$(TARGET)-gcc
CXX=$(TARGET)-g++
LD=$(TARGET)-ld
NM=$(TARGET)-gcc-nm
OBJCOPY=$(TARGET)-objcopy
OBJDUMP=$(TARGET)-objdump
RANLIB=$(TARGET)-gcc-ranlib
READELF=$(TARGET)-readelf
STRIP=$(TARGET)-strip

## Rust cross compile variables
export AR_$(subst -,_,$(TARGET))=$(TARGET)-ar
export CC_$(subst -,_,$(TARGET))=$(TARGET)-gcc
export CXX_$(subst -,_,$(TARGET))=$(TARGET)-g++
