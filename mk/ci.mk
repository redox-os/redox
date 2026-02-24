# Configuration file of the build system commands for the build server

IMG_TAG?=$(shell git describe --tags)
IMG_SEPARATOR?=_
IMG_DIR?=build/img/$(ARCH)
OS_TEST_DIR?=build/os-test/$(ARCH)

# CI image target - build standard images
# To leave out the build tag, set both IMG_TAG and IMG_SEPARATOR to null
ci-img: FORCE
	rm -rf $(IMG_DIR)
	mkdir -p $(IMG_DIR)
	$(MAKE) server desktop demo
	cd $(IMG_DIR) && zstd --rm *
	cd $(IMG_DIR) && sha256sum -b * > SHA256SUM

# The name of the target must match the name of the filesystem config file
server desktop demo: FORCE
	rm -f "build/$(ARCH)/$@/harddrive.img" "build/$(ARCH)/$@/redox-live.iso"
	$(MAKE) CONFIG_NAME=$@ build/$(ARCH)/$@/harddrive.img build/$(ARCH)/$@/redox-live.iso
	mkdir -p $(IMG_DIR)
	cp "build/$(ARCH)/$@/harddrive.img" "$(IMG_DIR)/redox_$(@)$(IMG_SEPARATOR)$(IMG_TAG)_harddrive.img"
	cp "build/$(ARCH)/$@/redox-live.iso" "$(IMG_DIR)/redox_$(@)$(IMG_SEPARATOR)$(IMG_TAG)_livedisk.iso"

ci-os-test: FORCE
	make CONFIG_NAME=os-test unmount
	rm -f "build/$(ARCH)/os-test/harddrive.img"
	$(MAKE) CONFIG_NAME=os-test qemu gpu=no
	rm -rf $(OS_TEST_DIR)
	mkdir -p $(OS_TEST_DIR)
	$(MAKE) CONFIG_NAME=os-test mount
	cp -rv build/$(ARCH)/os-test/filesystem/usr/share/os-test/html $(OS_TEST_DIR)
	cp -v build/$(ARCH)/os-test/filesystem/usr/share/os-test/os-test.json $(OS_TEST_DIR)
	tar \
		--create \
		--gzip \
		--file "$(OS_TEST_DIR)/out.tar.gz" \
		--directory="build/$(ARCH)/os-test/filesystem/usr/share/os-test" \
		out
	$(MAKE) CONFIG_NAME=os-test unmount

# CI packaging target
ci-pkg: prefix $(FSTOOLS_TAG) $(CONTAINER_TAG) FORCE
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	$(HOST_CARGO) build --manifest-path Cargo.toml --release
	export CI=1 COOKBOOK_LOGS=true COOKBOOK_CLEAN_BUILD=true PATH="$(PREFIX_PATH):$$PATH" COOKBOOK_HOST_SYSROOT="$(ROOT)/$(PREFIX_INSTALL)" && \
	./target/release/repo cook --with-package-deps "--filesystem=config/$(ARCH)/ci.toml"
endif

# CI toolchain
ci-toolchain: $(CONTAINER_TAG) FORCE
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	$(MAKE) PREFIX_BINARY=0 \
		"prefix/$(TARGET)/gcc-install.tar.gz" \
		"prefix/$(TARGET)/relibc-install.tar.gz" \
		"prefix/$(TARGET)/rust-install.tar.gz" \
		"prefix/$(TARGET)/clang-install.tar.gz"
	rm -rf "build/toolchain/$(HOST_TARGET)/$(TARGET)"
	mkdir -p "build/toolchain/$(HOST_TARGET)/$(TARGET)"
	cp "prefix/$(TARGET)/gcc-install.tar.gz" "build/toolchain/$(HOST_TARGET)/$(TARGET)/gcc-install.tar.gz"
	cp "prefix/$(TARGET)/relibc-install.tar.gz" "build/toolchain/$(HOST_TARGET)/$(TARGET)/relibc-install.tar.gz"
	cp "prefix/$(TARGET)/rust-install.tar.gz" "build/toolchain/$(HOST_TARGET)/$(TARGET)/rust-install.tar.gz"
	cp "prefix/$(TARGET)/clang-install.tar.gz" "build/toolchain/$(HOST_TARGET)/$(TARGET)/clang-install.tar.gz"
	cd "build/toolchain/$(HOST_TARGET)/$(TARGET)" && sha256sum -b * > SHA256SUM
endif
