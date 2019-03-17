# Configuration
## Architecture to build Redox for (aarch64 or x86_64)
ARCH?=x86_64
## Flags to pass to the installer (empty to download binary packages)
INSTALLER_FLAGS?=--cookbook=cookbook
## Enabled to use binary prefix (much faster)
PREFIX_BINARY?=1
## Enabled to build custom rustc
PREFIX_RUSTC?=0
## Filesystem size in MB (256 is the default)
FILESYSTEM_SIZE?=256

# Per host variables
UNAME := $(shell uname)
ifeq ($(UNAME),Darwin)
	FUMOUNT=sudo umount
	export NPROC=sysctl -n hw.ncpu
	PREFIX_BINARY=0
	VB_AUDIO=coreaudio
	VBM=/Applications/VirtualBox.app/Contents/MacOS/VBoxManage
else
	FUMOUNT=fusermount -u
	export NPROC=nproc
	VB_AUDIO=pulse
	VBM=VBoxManage
endif

# Automatic variables
ROOT=$(CURDIR)
export RUST_TARGET_PATH=$(ROOT)/kernel/targets
export XARGO_HOME=$(ROOT)/build/xargo
export XARGO_RUST_SRC=$(ROOT)/rust/src

## Kernel variables
KTARGET=$(ARCH)-unknown-none
KBUILD=build/kernel

## Userspace variables
export TARGET=$(ARCH)-unknown-redox
BUILD=build/userspace

## Bootloader variables
EFI_TARGET=$(ARCH)-efi-pe

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
