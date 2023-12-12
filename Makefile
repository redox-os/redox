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
	git pull --recurse-submodules
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

gdb: FORCE
	gdb cookbook/recipes/core/kernel/target/$(TARGET)/build/kernel.sym --eval-command="target remote localhost:1234"

# An empty target
FORCE:

# Wireshark
wireshark: FORCE
	wireshark $(BUILD)/network.pcap
