# Configuration and variables
include mk/config.mk

all: build/harddrive.bin

live: build/livedisk.bin

iso: build/livedisk.iso

clean:
	cargo clean --manifest-path cookbook/pkgutils/Cargo.toml
	cargo clean --manifest-path installer/Cargo.toml
	cargo clean --manifest-path installer/redoxfs/Cargo.toml
	cargo clean --manifest-path kernel/Cargo.toml
	-$(FUMOUNT) build/filesystem/ || true
	rm -rf build

pull:
	git pull --rebase --recurse-submodules
	git submodule sync
	git submodule update --recursive --init
	git clean -X -f -d
	make clean
	make update

update:
	cargo update --manifest-path cookbook/pkgutils/Cargo.toml
	cargo update --manifest-path installer/Cargo.toml
	cargo update --manifest-path installer/redoxfs/Cargo.toml
	cargo update --manifest-path kernel/Cargo.toml

# Emulation recipes
include mk/qemu.mk
include mk/bochs.mk
include mk/virtualbox.mk

# Kernel recipes
include mk/kernel.mk

# Filesystem recipes
include mk/initfs.mk
include mk/filesystem.mk

# Disk images
include mk/disk.mk

# An empty target
FORCE:

# A method of creating a listing for any binary
%.list: %
	objdump -C -M intel -D $< > $@

# Wireshark
wireshark: FORCE
	wireshark build/network.pcap
