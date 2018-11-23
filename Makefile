# Configuration and variables
include mk/config.mk

all: build/harddrive.bin

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
	make clean
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
	cd cookbook && ./fetch.sh \
		"$$(cargo run --manifest-path ../installer/Cargo.toml -- --list-packages -c ../initfs.toml)" \
		"$$(cargo run --manifest-path ../installer/Cargo.toml -- --list-packages -c ../filesystem.toml)"

# Cross compiler recipes
include mk/prefix.mk

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
	make INSTALLER_FLAGS= build/harddrive.bin.gz build/livedisk.iso #build/harddrive-efi.bin.gz build/livedisk-efi.iso
	rm -rf build/img
	mkdir build/img
	mv build/harddrive.bin.gz build/img/redox_$(IMG_TAG)_harddrive.bin.gz
	mv build/livedisk.iso build/img/redox_$(IMG_TAG)_livedisk.iso
	#mv build/harddrive-efi.bin.gz build/img/redox_$(IMG_TAG)_harddrive-efi.bin.gz
	#mv build/livedisk-efi.iso build/img/redox_$(IMG_TAG)_livedisk-efi.iso
	cd build/img && sha256sum -b * > SHA256SUM

# CI packaging target
ci-pkg: prefix FORCE
	export PATH="$(PREFIX_PATH):$$PATH" && \
	PACKAGES="$$(cargo run --manifest-path installer/Cargo.toml -- --list-packages -c ci.toml)" && \
	cd cookbook && \
	./fetch.sh "$${PACKAGES}" && \
	./repo.sh "$${PACKAGES}"

env: prefix FORCE
	export PATH="$(PREFIX_PATH):$$PATH" && \
	bash

# An empty target
FORCE:

# A method of creating a listing for any binary
%.list: %
	objdump -C -M intel -D $< > $@

# Wireshark
wireshark: FORCE
	wireshark build/network.pcap
