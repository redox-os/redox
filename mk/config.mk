# Configuration
ARCH?=x86_64
INSTALLER_FLAGS?=--cookbook=cookbook
PREFIX_RUSTC?=0

# Per host variables
UNAME := $(shell uname)
ifeq ($(UNAME),Darwin)
	ECHO=/bin/echo
	FUMOUNT=sudo umount
	export NPROC=sysctl -n hw.ncpu
	VB_AUDIO=coreaudio
	VBM="/Applications/VirtualBox.app/Contents/MacOS/VBoxManage"
else
	ECHO=echo
	FUMOUNT=fusermount -u
	export NPROC=nproc
	VB_AUDIO="pulse"
	VBM=VBoxManage
endif

# Automatic variables
ROOT=$(PWD)
export RUST_TARGET_PATH=$(ROOT)/kernel/targets
export XARGO_HOME=$(ROOT)/build/xargo
export XARGO_RUST_SRC=$(ROOT)/rust/src

# Cross compiler variables
AR=$(TARGET)-ar
AS=$(TARGET)-as
CC=$(TARGET)-gcc
CXX=$(TARGET)-g++
LD=$(TARGET)-ld
NM=$(TARGET)-nm
OBJCOPY=$(TARGET)-objcopy
OBJDUMP=$(TARGET)-objdump
RANLIB=$(TARGET)-ranlib
READELF=$(TARGET)-readelf
STRIP=$(TARGET)-strip

# Rust cross compile variables
export AR_$(subst -,_,$(TARGET))=$(TARGET)-ar
export CC_$(subst -,_,$(TARGET))=$(TARGET)-gcc
export CXX_$(subst -,_,$(TARGET))=$(TARGET)-g++

# Kernel variables
KTARGET=$(ARCH)-unknown-none
KBUILD=build/kernel

# Userspace variables
export TARGET=$(ARCH)-unknown-redox
BUILD=build/userspace
