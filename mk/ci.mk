# Configuration file of the build system commands for the build server

IMG_TAG?=$(shell git describe --tags)
IMG_SEPARATOR?=_
IMG_DIR?=build/img/$(ARCH)

# CI image target - build standard images
# To leave out the build tag, set both IMG_TAG and IMG_SEPARATOR to null
ci-img: FORCE
	rm -rf $(IMG_DIR)
	mkdir -p $(IMG_DIR)
	$(MAKE) minimal minimal-net server desktop demo
	cd $(IMG_DIR) && zstd --rm *
	cd $(IMG_DIR) && sha256sum -b * > SHA256SUM

# The name of the target must match the name of the filesystem config file
minimal minimal-net server desktop demo: FORCE
	rm -f "build/$(ARCH)/$@/harddrive.img" "build/$(ARCH)/$@/redox-live.iso"
	$(MAKE) CONFIG_NAME=$@ build/$(ARCH)/$@/harddrive.img build/$(ARCH)/$@/redox-live.iso
	mkdir -p $(IMG_DIR)
	cp "build/$(ARCH)/$@/harddrive.img" "$(IMG_DIR)/redox_$(@)$(IMG_SEPARATOR)$(IMG_TAG)_harddrive.img"
	cp "build/$(ARCH)/$@/redox-live.iso" "$(IMG_DIR)/redox_$(@)$(IMG_SEPARATOR)$(IMG_TAG)_redox-live.iso"

# CI packaging target
ci-pkg: prefix $(FSTOOLS_TAG) $(CONTAINER_TAG) FORCE
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) $(MAKE) $@
else
	$(HOST_CARGO) build --manifest-path cookbook/Cargo.toml --release
	$(HOST_CARGO) build --manifest-path cookbook/pkgar/Cargo.toml --release
	$(HOST_CARGO) build --manifest-path installer/Cargo.toml --release
	export PATH="$(PREFIX_PATH):$$PATH" && \
	export COOKBOOK_HOST_SYSROOT="$(ROOT)/$(PREFIX_INSTALL)" && \
	PACKAGES="$$($(LIST_PACKAGES) $(LIST_PACKAGES_OPTS) -c config/$(ARCH)/ci.toml)" && \
	cd cookbook && \
	./fetch.sh "$${PACKAGES}" && \
	./repo.sh $(REPO_NONSTOP) "$${PACKAGES}"
endif

# CI toolchain
ci-toolchain: $(CONTAINER_TAG) FORCE
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) $(MAKE) $@
else
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
endif
