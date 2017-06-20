# Configuration
ARCH?=x86_64

# Per host variables
UNAME := $(shell uname)
ifeq ($(UNAME),Darwin)
	ECHO=/bin/echo
	FUMOUNT=sudo umount
	export LD=$(ARCH)-elf-ld
	export NPROC=sysctl -n hw.ncpu
	export STRIP=$(ARCH)-elf-strip
	VB_AUDIO=coreaudio
	VBM="/Applications/VirtualBox.app/Contents/MacOS/VBoxManage"
else
	ECHO=echo
	FUMOUNT=fusermount -u
	export LD=ld
	export NPROC=nproc
	export STRIP=strip
	VB_AUDIO="pulse"
	VBM=VBoxManage
endif

# Automatic variables
ROOT=$(PWD)
export INITFS_FOLDER=$(ROOT)/build/initfs
export RUST_TARGET_PATH=$(ROOT)/kernel/targets
export XARGO_HOME=$(ROOT)/build/xargo
export XARGO_RUST_SRC=$(ROOT)/rust/src

# Kernel variables
KTARGET=$(ARCH)-unknown-none
KBUILD=build/kernel

# Userspace variables
export TARGET=$(ARCH)-unknown-redox
BUILD=build/userspace
