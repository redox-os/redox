IMG_TAG?=$(shell git describe --tags)
IMG_SEPARATOR?=_
IMG_DIR?=$(BUILD)/img

# CI image target - build desktop, server and demo images
# To leave out the build tag, set both IMG_TAG and IMG_SEPARATOR to null
ci-img: FORCE
	rm -rf $(IMG_DIR)
	mkdir -p $(IMG_DIR)
	$(MAKE) demo desktop server
	cd $(IMG_DIR) && sha256sum -b * > SHA256SUM

# The name of the target must match the name of the filesystem config file
desktop server demo: FORCE
	rm -f "$(BUILD)/harddrive.img" "$(BUILD)/livedisk.iso"
	$(MAKE) REPO_BINARY=1 \
		FILESYSTEM_CONFIG=config/$(ARCH)/$@.toml \
		$(BUILD)/harddrive.img \
		$(BUILD)/livedisk.iso
	cp "$(BUILD)/harddrive.img" "$(IMG_DIR)/redox_$(@)$(IMG_SEPARATOR)$(IMG_TAG)_harddrive.img"
	cp "$(BUILD)/livedisk.iso" "$(IMG_DIR)/redox_$(@)$(IMG_SEPARATOR)$(IMG_TAG)_livedisk.iso"

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
	rm -rf "$(BUILD)/toolchain"
	mkdir -p "$(BUILD)/toolchain"
	cp "prefix/$(TARGET)/gcc-install.tar.gz" "$(BUILD)/toolchain/gcc-install.tar.gz"
	cp "prefix/$(TARGET)/relibc-install.tar.gz" "$(BUILD)/toolchain/relibc-install.tar.gz"
	cp "prefix/$(TARGET)/rust-install.tar.gz" "$(BUILD)/toolchain/rust-install.tar.gz"
	cd "$(BUILD)/toolchain" && sha256sum -b * > SHA256SUM
