# Configuration and variables
include mk/config.mk

# Dependencies
include mk/depends.mk

all: $(BUILD)/harddrive.img

live:
	-$(FUMOUNT) $(BUILD)/filesystem/ || true
	-$(FUMOUNT) /tmp/redox_installer/ || true
	rm -f $(BUILD)/livedisk.iso
	$(MAKE) $(BUILD)/livedisk.iso

popsicle: $(BUILD)/livedisk.iso
	popsicle-gtk $(BUILD)/livedisk.iso

image:
	-$(FUMOUNT) $(BUILD)/filesystem/ || true
	-$(FUMOUNT) /tmp/redox_installer/ || true
	rm -f $(BUILD)/harddrive.img $(BUILD)/livedisk.iso
	$(MAKE) all

rebuild:
	-$(FUMOUNT) $(BUILD)/filesystem/ || true
	-$(FUMOUNT) /tmp/redox_installer/ || true
	rm -rf $(BUILD)
	$(MAKE) all

clean: $(CONTAINER_TAG)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) $(MAKE) $@
else
	cd cookbook && ./clean.sh
	-rm -rf cookbook/repo
	cargo clean --manifest-path cookbook/pkgutils/Cargo.toml
	cargo clean --manifest-path installer/Cargo.toml
	cargo clean --manifest-path redoxfs/Cargo.toml
	cargo clean --manifest-path relibc/Cargo.toml
endif
	-$(FUMOUNT) $(BUILD)/filesystem/ || true
	-$(FUMOUNT) /tmp/redox_installer/ || true
	rm -rf $(BUILD)

distclean: $(CONTAINER_TAG)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) $(MAKE) $@
else
	$(MAKE) clean
	cd cookbook && ./unfetch.sh
endif

pull:
	git pull
	git submodule sync --recursive
	git submodule update --recursive --init

fetch: $(BUILD)/fetch.tag

repo: $(BUILD)/repo.tag

# Podman build recipes and vars
include mk/podman.mk

# Disk Imaging and Cookbook tools
include mk/fstools.mk

# Cross compiler recipes
include mk/prefix.mk

# Repository maintenance
include mk/repo.mk

# Disk images
include mk/disk.mk

# Emulation recipes
include mk/qemu.mk
include mk/virtualbox.mk

# CI
include mk/ci.mk

env: prefix FORCE $(CONTAINER_TAG)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) $(MAKE) $@
else
	export PATH="$(PREFIX_PATH):$$PATH" && \
	bash
endif

export RUST_GDB=gdb-multiarch # Necessary when debugging for another architecture than the host
GDB_KERNEL_FILE=cookbook/recipes/core/kernel/target/$(TARGET)/build/kernel.sym
gdb: FORCE
	rust-gdb $(GDB_KERNEL_FILE) --eval-command="target remote :1234"

# This target allows debugging a userspace application without requiring gdbserver running inside
# the VM. Because gdb doesn't know when the userspace application is scheduled by the kernel and as
# it stops the entire VM rather than just the userspace application that the user wants to debug,
# connecting to a gdbserver running inside the VM is highly encouraged when possible. This target
# should only be used when the application to debug runs early during boot before the network stack
# has started or you need to debug the interaction between the application and the kernel.
# tl;dr: DO NOT USE THIS TARGET UNLESS YOU HAVE TO
gdb-userspace: FORCE
	rust-gdb $(GDB_APP_FILE) --eval-command="add-symbol-file $(GDB_KERNEL_FILE) 0x$(shell readelf -S $(GDB_KERNEL_FILE) | grep .text | cut -c43-58)" --eval-command="target remote :1234"

# An empty target
FORCE:

# Wireshark
wireshark: FORCE
	wireshark $(BUILD)/network.pcap
