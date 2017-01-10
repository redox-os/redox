# Configuration and variables
include mk/config.mk

all: build/harddrive.bin

live: build/livedisk.bin

iso: build/livedisk.iso

clean:
	cargo clean
	cargo clean --manifest-path rust/src/libcollections/Cargo.toml
	cargo clean --manifest-path rust/src/libstd/Cargo.toml
	-$(FUMOUNT) build/filesystem/ || true
	rm -rf initfs/bin
	rm -rf filesystem/bin filesystem/sbin filesystem/ui/bin
	rm -rf build

update:
	cargo update

pull:
	git pull --rebase --recurse-submodules
	git submodule sync
	git submodule update --recursive --init
	git clean -X -f -d
	make clean
	make update

# Emulation recipes
include mk/qemu.mk
include mk/bochs.mk
include mk/virtualbox.mk

# Kernel recipes
include mk/kernel.mk

# Userspace recipes
include mk/userspace/mod.mk

# Documentation
include mk/doc.mk

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
