# Configuration and variables
include mk/config.mk

# Dependencies
include mk/depends.mk

all: build/harddrive.img

live: build/livedisk.iso

rebuild:
	-$(FUMOUNT) build/filesystem/ || true
	rm -rf build
	$(MAKE) all

clean:
	cd cookbook && ./clean.sh
	cargo clean --manifest-path cookbook/pkgutils/Cargo.toml
	cargo clean --manifest-path installer/Cargo.toml
	cargo clean --manifest-path redoxfs/Cargo.toml
	cargo clean --manifest-path relibc/Cargo.toml
	-$(FUMOUNT) build/filesystem/ || true
	rm -rf build

distclean:
	$(MAKE) clean
	cd cookbook && ./unfetch.sh

pull:
	git pull --recurse-submodules
	git submodule sync --recursive
	git submodule update --recursive --init

fetch: build/fetch.tag

repo: build/repo.tag

# Cross compiler recipes
include mk/prefix.mk

# Repository maintenance
include mk/repo.mk

# Disk images
include mk/disk.mk

# Emulation recipes
include mk/qemu.mk
include mk/bochs.mk
include mk/virtualbox.mk

# CI image target
IMG_TAG?=$(shell git describe --tags)
ci-img: FORCE
	$(MAKE) REPO_BINARY=1 \
		build/harddrive.img.gz \
		build/livedisk.iso.gz
	rm -rf build/img
	mkdir -p build/img
	cp "build/harddrive.img.gz" "build/img/redox_$(IMG_TAG)_harddrive.img.gz"
	cp "build/livedisk.iso.gz" "build/img/redox_$(IMG_TAG)_livedisk.iso.gz"
	cd build/img && sha256sum -b * > SHA256SUM

# CI packaging target
ci-pkg: prefix FORCE
	$(HOST_CARGO) build --manifest-path cookbook/Cargo.toml --release
	$(HOST_CARGO) build --manifest-path installer/Cargo.toml --release
	export PATH="$(PREFIX_PATH):$$PATH" && \
	PACKAGES="$$($(INSTALLER) --list-packages -c ci.toml)" && \
	cd cookbook && \
	./fetch.sh "$${PACKAGES}" && \
	./repo.sh "$${PACKAGES}"

# CI toolchain
ci-toolchain: FORCE
	$(MAKE) PREFIX_BINARY=0 \
		"prefix/$(TARGET)/gcc-install.tar.gz" \
		"prefix/$(TARGET)/relibc-install.tar.gz" \
		"prefix/$(TARGET)/rust-install.tar.gz"
	rm -rf "build/toolchain/$(TARGET)"
	mkdir -p "build/toolchain/$(TARGET)"
	cp "prefix/$(TARGET)/gcc-install.tar.gz" "build/toolchain/$(TARGET)/gcc-install.tar.gz"
	cp "prefix/$(TARGET)/relibc-install.tar.gz" "build/toolchain/$(TARGET)/relibc-install.tar.gz"
	cp "prefix/$(TARGET)/rust-install.tar.gz" "build/toolchain/$(TARGET)/rust-install.tar.gz"
	cd "build/toolchain/$(TARGET)" && sha256sum -b * > SHA256SUM

env: prefix FORCE
	export PATH="$(PREFIX_PATH):$$PATH" && \
	bash

gdb: FORCE
	gdb cookbook/recipes/kernel/build/kernel.sym --eval-command="target remote localhost:1234"

# An empty target
FORCE:

# Gzip any binary
%.gz: %
	gzip -k -f $<

# Wireshark
wireshark: FORCE
	wireshark build/network.pcap
