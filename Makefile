# Configuration and variables
include mk/config.mk

# Dependencies
include mk/depends.mk

all: build/harddrive.bin

coreboot: build/coreboot.elf

live: build/livedisk.bin

iso: build/livedisk.iso

clean:
	cd cookbook && ./clean.sh
	cargo clean --manifest-path cookbook/pkgutils/Cargo.toml
	cargo clean --manifest-path installer/Cargo.toml
	cargo clean --manifest-path kernel/Cargo.toml
	cargo clean --manifest-path kernel/syscall/Cargo.toml
	cargo clean --manifest-path redoxfs/Cargo.toml
	-$(FUMOUNT) build/filesystem/ || true
	rm -rf build

distclean:
	$(MAKE) clean
	cd cookbook && ./unfetch.sh

pull:
	git pull --recurse-submodules
	git submodule sync --recursive
	git submodule update --recursive --init

update:
	cd cookbook && ./update.sh \
		"$$(cargo run --manifest-path ../installer/Cargo.toml -- --list-packages -c ../initfs.toml)" \
		"$$(cargo run --manifest-path ../installer/Cargo.toml -- --list-packages -c ../filesystem.toml)"
	cargo update --manifest-path cookbook/pkgutils/Cargo.toml
	cargo update --manifest-path installer/Cargo.toml
	cargo update --manifest-path kernel/Cargo.toml
	cargo update --manifest-path redoxfs/Cargo.toml

fetch:
	cargo build --manifest-path cookbook/Cargo.toml --release
	cd cookbook && ./fetch.sh \
		"$$(cargo run --manifest-path ../installer/Cargo.toml -- --list-packages -c ../initfs.toml)" \
		"$$(cargo run --manifest-path ../installer/Cargo.toml -- --list-packages -c ../filesystem.toml)"

# Cross compiler recipes
include mk/prefix.mk

# Bootloader recipes
include mk/bootloader.mk

# Kernel recipes
include mk/kernel.mk

# Filesystem recipes
include mk/initfs.mk
include mk/filesystem.mk

# Disk images
include mk/disk.mk

# Emulation recipes
include mk/qemu.mk
include mk/bochs.mk
include mk/virtualbox.mk

# CI image target
ci-img: FORCE
	$(MAKE) INSTALLER_FLAGS= \
		build/harddrive.bin.gz \
		build/livedisk.bin.gz \
		build/livedisk.iso.gz \
		build/harddrive-efi.bin.gz \
		build/livedisk-efi.bin.gz
	rm -rf build/img
	mkdir -p build/img
	cp "build/harddrive.bin.gz" "build/img/redox_$(IMG_TAG)_harddrive.bin.gz"
	cp "build/livedisk.bin.gz" "build/img/redox_$(IMG_TAG)_livedisk.bin.gz"
	cp "build/livedisk.iso.gz" "build/img/redox_$(IMG_TAG)_livedisk.iso.gz"
	cp "build/harddrive-efi.bin.gz" "build/img/redox_$(IMG_TAG)_harddrive-efi.bin.gz"
	cp "build/livedisk-efi.bin.gz" "build/img/redox_$(IMG_TAG)_livedisk-efi.bin.gz"
	cd build/img && sha256sum -b * > SHA256SUM

# CI packaging target
ci-pkg: prefix FORCE
	cargo build --manifest-path cookbook/Cargo.toml --release
	export PATH="$(PREFIX_PATH):$$PATH" && \
	PACKAGES="$$(cargo run --manifest-path installer/Cargo.toml -- --list-packages -c ci.toml)" && \
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
	gdb build/kernel.sym --eval-command="target remote localhost:1234"

# An empty target
FORCE:

# Gzip any binary
%.gz: %
	gzip -k -f $<

# Create a listing for any binary
%.list: %
	export PATH="$(PREFIX_PATH):$$PATH" && \
	$(OBJDUMP) -C -M intel -D $< > $@

# Wireshark
wireshark: FORCE
	wireshark build/network.pcap
