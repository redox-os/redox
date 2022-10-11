IMG_TAG?=$(shell git describe --tags)

# CI image target
ci-img: FORCE
	$(MAKE) REPO_BINARY=1 \
		build/harddrive.img \
		build/livedisk.iso
	rm -rf build/img
	mkdir -p build/img
	cp "build/harddrive.img" "build/img/redox_$(IMG_TAG)_harddrive.img"
	cp "build/livedisk.iso" "build/img/redox_$(IMG_TAG)_livedisk.iso"
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
