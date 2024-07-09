# Configuration file of the build system environment variables

-include .config

HOST_ARCH?=$(shell uname -m)

# Configuration
## Architecture to build Redox for (aarch64, i686, or x86_64). Defaults to a host one
ARCH?=$(HOST_ARCH)
## Sub-device type for aarch64 if needed
BOARD?=
## Enable to use binary prefix (much faster)
PREFIX_BINARY?=1
## Enable to use binary packages (much faster)
REPO_BINARY?=0
## Name of the configuration to include in the image name e.g. desktop or server
CONFIG_NAME?=desktop
## Ignore errors when building the repo, attempt to build every package
## REPO_NONSTOP?=--nonstop
REPO_NONSTOP?=
## Select filesystem config
ifeq ($(BOARD),)
FILESYSTEM_CONFIG?=config/$(ARCH)/$(CONFIG_NAME).toml
else
FILESYSTEM_CONFIG?=config/$(ARCH)/$(BOARD)/$(CONFIG_NAME).toml
endif
HOST_CARGO=env -u RUSTUP_TOOLCHAIN -u CC -u TARGET cargo
## Filesystem size in MB (default comes from filesystem_size in the FILESYSTEM_CONFIG)
FILESYSTEM_SIZE?=$(shell $(HOST_CARGO) run --release --manifest-path installer/Cargo.toml -- --filesystem-size -c $(FILESYSTEM_CONFIG))
## Flags to pass to redoxfs-mkfs. Add --encrypt to set up disk encryption
REDOXFS_MKFS_FLAGS?=
## Set to 1 to enable Podman build, any other value will disable it
PODMAN_BUILD?=0
## The containerfile to use for the Podman base image
CONTAINERFILE?=podman/redox-base-containerfile

# Per host variables
export NPROC=nproc
export REDOX_MAKE=make
HOST_TARGET := $(shell env -u RUSTUP_TOOLCHAIN rustc -vV | grep host | cut -d: -f2 | tr -d " ")
ifneq ($(HOST_TARGET),x86_64-unknown-linux-gnu)
	# The binary prefix is only built for x86_64 Linux hosts
	PREFIX_BINARY=0
endif
UNAME := $(shell uname)
ifeq ($(UNAME),Darwin)
	FUMOUNT=umount
	export NPROC=sysctl -n hw.ncpu
	VB_AUDIO=coreaudio
	VBM=/Applications/VirtualBox.app/Contents/MacOS/VBoxManage
else ifeq ($(UNAME),FreeBSD)
	FUMOUNT=sudo umount
	export REDOX_MAKE=gmake
	VB_AUDIO=pulse # To check, will probably be OSS on most setups
	VBM=VBoxManage
else
	# Detect which version of the fusermount binary is available.
	ifneq (, $(shell which fusermount3))
		FUMOUNT=fusermount3 -u
	else
		FUMOUNT=fusermount -u
	endif

	VB_AUDIO=pulse
	VBM=VBoxManage
endif

ifneq ($(UNAME),Linux)
	PREFIX_BINARY=0
endif
ifneq ($(HOST_ARCH),x86_64)
	PREFIX_BINARY=0
endif

# Automatic variables
ROOT=$(CURDIR)
export RUST_COMPILER_RT_ROOT=$(ROOT)/rust/src/llvm-project/compiler-rt

## Userspace variables
export TARGET=$(ARCH)-unknown-redox
BUILD=build/$(ARCH)/$(CONFIG_NAME)
INSTALLER=installer/target/release/redox_installer
ifeq ($(REPO_BINARY),0)
INSTALLER+=--cookbook=cookbook
REPO_TAG=$(BUILD)/repo.tag
else
INSTALLER+=--cookbook=cookbook --repo-binary
REPO_TAG=$(BUILD)/repo.tag
endif

FSTOOLS_TAG=build/fstools.tag
export BOARD

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


## If Podman is being used, a container is required
ifeq ($(PODMAN_BUILD),1)
CONTAINER_TAG=build/container.tag
else
CONTAINER_TAG=
endif
