# Configuration file of the build system environment variables

-include .config

HOST_ARCH?=$(shell uname -m)

# Configuration
## Architecture to build Redox for (aarch64, i586, or x86_64). Defaults to a host one
ARCH?=$(HOST_ARCH)
## Sub-device type for aarch64 if needed
BOARD?=
## Enable to use binary prefix (much faster)
PREFIX_BINARY?=1
## Enable to use binary packages (much faster)
REPO_BINARY?=0
## Name of the configuration to include in the image name e.g. desktop or server
CONFIG_NAME?=desktop
## Build appstream data for repo
REPO_APPSTREAM?=0
## Ignore errors when building the repo, attempt to build every package
REPO_NONSTOP?=0
## Do not update source repos, attempt to build in offline condition
REPO_OFFLINE?=0
## Do not strip debug info for local build
REPO_DEBUG?=0
## Old config value that need to be corrected
ifeq ($(ARCH),i686)
	ARCH=i586
endif
## Select filesystem config
ifeq ($(BOARD),)
FILESYSTEM_CONFIG?=config/$(ARCH)/$(CONFIG_NAME).toml
else
FILESYSTEM_CONFIG?=config/$(ARCH)/$(BOARD)/$(CONFIG_NAME).toml
endif
HOST_CARGO=env -u RUSTUP_TOOLCHAIN -u CC -u TARGET cargo
## Filesystem size in MB (default comes from filesystem_size in the FILESYSTEM_CONFIG)
## FILESYSTEM_SIZE?=$(shell $(INSTALLER) --filesystem-size -c $(FILESYSTEM_CONFIG))
## Flags to pass to redoxfs-mkfs. Add --encrypt to set up disk encryption
REDOXFS_MKFS_FLAGS?=
## Set to 1 to enable Podman build, any other value will disable it
PODMAN_BUILD?=1
## Set to 1 to put filesystem tools inside podman, any other value will install it to host
FSTOOLS_IN_PODMAN?=0
## Enable sccache to speed up cargo builds
## only do this by default if this is inside podman
SCCACHE_BUILD?=$(shell [ -f /run/.containerenv ] && echo 1 || echo 0)
## The containerfile to use for the Podman base image
CONTAINERFILE?=podman/redox-base-containerfile

# Per host variables
export NPROC=nproc
export REDOX_MAKE=make

ifneq ($(PODMAN_BUILD),1)
FSTOOLS_IN_PODMAN=0
HOST_TARGET := $(shell env -u RUSTUP_TOOLCHAIN rustc -vV | grep host | cut -d: -f2 | tr -d " ")
# x86_64 linux hosts have all toolchains
ifneq ($(HOST_TARGET),x86_64-unknown-linux-gnu)
	ifeq ($(ARCH),aarch64)
		# aarch64 linux hosts have aarch64 toolchain
		ifneq ($(HOST_TARGET),aarch64-unknown-linux-gnu)
            $(info The $(ARCH) binary prefix is only built for x86_64 and aarch64 Linux hosts)
			PREFIX_BINARY=0
		endif
	else
        $(info The $(ARCH) binary prefix is only built for x86_64 Linux hosts)
		PREFIX_BINARY=0
	endif
endif
endif

ifeq ($(SCCACHE_BUILD),1)
ifeq (,$(shell command -v sccache))
    $(info sccache not found in PATH)
	SCCACHE_BUILD=0
endif
endif

ifeq ($(REPO_APPSTREAM),1)
	REPO_APPSTREAM=--appstream
else ifeq ($(REPO_APPSTREAM),0)
	REPO_APPSTREAM=
endif
ifeq ($(REPO_NONSTOP),1)
	REPO_NONSTOP=--nonstop
else ifeq ($(REPO_NONSTOP),0)
	REPO_NONSTOP=
endif
ifeq ($(REPO_OFFLINE),1)
	REPO_OFFLINE=--offline
else ifeq ($(REPO_OFFLINE),0)
	REPO_OFFLINE=
endif
ifeq ($(REPO_DEBUG),1)
	export COOKBOOK_NOSTRIP=true
	export COOKBOOK_DEBUG=true
#TODO: https://gitlab.redox-os.org/redox-os/relibc/-/issues/226
#	export PROFILE=debug
#	export RUSTCFLAGS="-Cdebuginfo=2"
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

# Automatic variables
ROOT=$(CURDIR)
export RUST_COMPILER_RT_ROOT=$(ROOT)/rust/src/llvm-project/compiler-rt

## Userspace variables
ifeq ($(ARCH),riscv64gc)
	export TARGET=riscv64gc-unknown-redox
	export GNU_TARGET=riscv64-unknown-redox
else
	export TARGET=$(ARCH)-unknown-redox
	export GNU_TARGET=$(ARCH)-unknown-redox
endif
BUILD=build/$(ARCH)/$(CONFIG_NAME)
MOUNT_DIR=$(BUILD)/filesystem
FSTOOLS=build/fstools
INSTALLER=$(FSTOOLS)/bin/redox_installer
REDOXFS=$(FSTOOLS)/bin/redoxfs
REDOXFS_MKFS=$(FSTOOLS)/bin/redoxfs-mkfs
INSTALLER_OPTS=
COOKBOOK_OPTS="--filesystem=$(FILESYSTEM_CONFIG)"
ifeq ($(REPO_BINARY),0)
INSTALLER_OPTS+=--cookbook=.
else
INSTALLER_OPTS+=--cookbook=. --repo-binary
COOKBOOK_OPTS+=" --repo-binary"
endif

REPO_TAG=$(BUILD)/repo.tag
FSTOOLS_TAG=build/fstools.tag
export BOARD

## Cross compiler variables
AR=$(GNU_TARGET)-gcc-ar
AS=$(GNU_TARGET)-as
CC=$(GNU_TARGET)-gcc
CXX=$(GNU_TARGET)-g++
LD=$(GNU_TARGET)-ld
NM=$(GNU_TARGET)-gcc-nm
OBJCOPY=$(GNU_TARGET)-objcopy
OBJDUMP=$(GNU_TARGET)-objdump
RANLIB=$(GNU_TARGET)-gcc-ranlib
READELF=$(GNU_TARGET)-readelf
STRIP=$(GNU_TARGET)-strip

ifeq ($(SCCACHE_BUILD),1)
	export CC_WRAPPER:=sccache
	export RUSTC_WRAPPER:=$(CC_WRAPPER)
	CC=$(CC_WRAPPER) $(GNU_TARGET)-gcc
	CXX=$(CC_WRAPPER) $(GNU_TARGET)-g++
endif

## Rust cross compile variables
export AR_$(subst -,_,$(TARGET)):=$(AR)
export CC_$(subst -,_,$(TARGET)):=$(CC)
export CXX_$(subst -,_,$(TARGET)):=$(CXX)

## If Podman is being used, a container is required
ifeq ($(PODMAN_BUILD),1)
CONTAINER_TAG=build/container.tag
else
CONTAINER_TAG=
endif
