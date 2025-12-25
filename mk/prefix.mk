# Configuration file for the Rust/GCC cross-compilers, relibc and libtool

PREFIX=prefix/$(TARGET)

PREFIX_INSTALL=$(PREFIX)/sysroot/
PREFIX_PATH=$(ROOT)/$(PREFIX_INSTALL)/bin
BINUTILS_TARGET=recipes/dev/binutils-gdb/target/$(HOST_TARGET)/$(TARGET)
LIBTOOL_TARGET=recipes/dev/libtool/target/$(HOST_TARGET)
GCC_TARGET=recipes/dev/gcc13/target/$(HOST_TARGET)/$(TARGET)
LIBSTDCXX_TARGET=recipes/libs/libstdcxx-v3/target/$(TARGET)/$(HOST_TARGET)
RELIBC_FREESTANDING_TARGET=recipes/core/relibc/target/$(TARGET)/$(HOST_TARGET)
RELIBC_TARGET=recipes/core/relibc/target/$(TARGET)

# official RISC-V support introduced in newer version
UPSTREAM_RUSTC_VERSION=2025-11-15

export PREFIX_RUSTFLAGS=-L $(ROOT)/$(PREFIX_INSTALL)/$(TARGET)/lib
export RUSTUP_TOOLCHAIN=$(ROOT)/$(PREFIX_INSTALL)
export REDOXER_TOOLCHAIN=$(RUSTUP_TOOLCHAIN)

prefix: $(PREFIX)/sysroot

# Update relibc used for compiling and clean all statically linked recipes
prefix_clean: | $(FSTOOLS_TAG)
	rm -rf $(PREFIX)/relibc-install $(PREFIX)/sysroot $(REPO_TAG)
	$(MAKE) c.base,base-initfs,extrautils,kernel,ion,pkgutils,redoxfs,relibc

$(PREFIX)/relibc-install: $(PREFIX)/rust-install | $(CONTAINER_TAG)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	@echo "\033[1;36;49mBuilding relibc-install\033[0m"
	rm -rf "$@.partial" "$@"
	cp -r "$(PREFIX)/rust-install" "$@.partial"
	rm -rf "$@.partial/$(TARGET)/include/"*
	cp -r "$(PREFIX)/rust-install/$(GNU_TARGET)/include/c++" "$@.partial/$(GNU_TARGET)/include/c++"
	cp -r "$(PREFIX)/rust-install/lib/rustlib/$(HOST_TARGET)/lib/" "$@.partial/lib/rustlib/$(HOST_TARGET)/"
	export PATH="$(ROOT)/$@.partial/bin:$$PATH" && \
	export CARGO="env -u CARGO cargo" CI=1 && \
	./target/release/repo cook relibc
	cp -r "$(RELIBC_TARGET)/stage/usr/". "$@.partial/$(GNU_TARGET)"
	touch "$@.partial"
	mv "$@.partial" "$@"
endif

$(PREFIX)/relibc-install.tar.gz: $(PREFIX)/relibc-install
	tar \
		--create \
		--gzip \
		--file "$@" \
		--directory="$<" \
		.

# TODO: move this behind PREFIX_BINARY=0 when compiled prefix has it
$(PREFIX)/libtool-install: | $(FSTOOLS_TAG) $(CONTAINER_TAG)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	@echo "\033[1;36;49mBuilding libtool-install\033[0m"
	rm -rf "$@.partial" "$@"
	mkdir -p "$@.partial"
	export CI=1 COOKBOOK_CLEAN_BUILD=true COOKBOOK_HOST_SYSROOT=/usr && \
	./target/release/repo cook host:libtool
	cp -r "$(LIBTOOL_TARGET)/stage/usr/". "$@.partial"
	touch "$@.partial"
	mv "$@.partial" "$@"
endif

$(PREFIX)/sysroot: $(PREFIX)/relibc-install $(PREFIX)/libtool-install $(CONTAINER_TAG)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	rm -rf "$@"
	cp -r "$(PREFIX)/relibc-install/" "$@"
	cp -r "$(PREFIX)/libtool-install/". "$@.partial"
# adapt path for libtoolize
	$(SED) -i 's|/usr/share|$(ROOT)/$@/share|g' "$@/bin/libtoolize"
	touch "$@"
endif

# PREFIX_BINARY ---------------------------------------------------
ifeq ($(PREFIX_BINARY),1)

$(PREFIX)/rust-install.tar.gz: | $(CONTAINER_TAG)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	mkdir -p "$(@D)"
	#TODO: figure out why rust-install.tar.gz is missing /lib/rustlib/$(HOST_TARGET)/lib
	wget -O $@.partial "https://static.redox-os.org/toolchain/$(HOST_TARGET)/$(TARGET)/relibc-install.tar.gz"
	mv $@.partial $@
endif

$(PREFIX)/rust-install: $(PREFIX)/rust-install.tar.gz $(CONTAINER_TAG)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	rm -rf "$@.partial" "$@"
	mkdir -p "$@.partial"
	tar --extract --file "$<" --directory "$@.partial" --no-same-owner --strip-components=1
	touch "$@.partial"
	mv "$@.partial" "$@"
endif

else

# BUILD GCC ---------------------------------------------------
$(PREFIX)/binutils-install: | $(FSTOOLS_TAG) $(CONTAINER_TAG)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	@echo "\033[1;36;49mBuilding binutils-install\033[0m"
	rm -rf "$@.partial" "$@"
	mkdir -p "$@.partial"
	export CI=1 COOKBOOK_CLEAN_BUILD=true COOKBOOK_HOST_SYSROOT=/usr COOKBOOK_CROSS_TARGET=$(TARGET) COOKBOOK_CROSS_GNU_TARGET=$(GNU_TARGET) && \
	./target/release/repo cook host:binutils-gdb 
	cp -r "$(BINUTILS_TARGET)/stage/usr/". "$@.partial"
	touch "$@.partial"
	mv "$@.partial" "$@"
endif

$(PREFIX)/gcc-freestanding-install: $(PREFIX)/binutils-install | $(FSTOOLS_TAG) $(CONTAINER_TAG)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	@echo "\033[1;36;49mBuilding gcc-freestanding-install\033[0m"
	rm -rf "$@.partial" "$@" $(PREFIX)/relibc-freestanding-install  $(PREFIX)/sysroot
	mkdir -p "$@.partial" $(PREFIX)/relibc-freestanding-install/$(TARGET)/include
	export CI=1 PATH="$(ROOT)/$(PREFIX)/binutils-install/bin:$$PATH" \
		COOKBOOK_CLEAN_BUILD=true COOKBOOK_CROSS_TARGET=$(TARGET) COOKBOOK_CROSS_GNU_TARGET=$(GNU_TARGET) \
		COOKBOOK_HOST_SYSROOT=/usr COOKBOOK_CROSS_SYSROOT=$(ROOT)/$(PREFIX)/relibc-freestanding-install/$(TARGET) && \
	./target/release/repo cook host:gcc13
	cp -r "$(GCC_TARGET)/stage/usr/". "$@.partial"
	cp -r "$(GCC_TARGET)/stage.cxx/usr/". "$@.partial"
	rm -rf $(PREFIX)/relibc-freestanding-install
	touch "$@.partial"
	mv "$@.partial" "$@"
endif

$(PREFIX)/relibc-freestanding-install: $(PREFIX)/gcc-freestanding-install $(PREFIX)/binutils-install | $(FSTOOLS_TAG) $(CONTAINER_TAG)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	@echo "\033[1;36;49mBuilding relibc-freestanding-install\033[0m"
	rm -rf "$@.partial" "$@"
	mkdir -p "$@.partial/$(TARGET)"
	export CARGO="env -u CARGO -u RUSTUP_TOOLCHAIN cargo" && \
	export PATH="$(ROOT)/$(PREFIX)/gcc-freestanding-install/bin:$(ROOT)/$(PREFIX)/binutils-install/bin:$$PATH" && \
	export CC_$(subst -,_,$(TARGET))="$(GNU_TARGET)-gcc -isystem $(ROOT)/$@.partial/$(GNU_TARGET)/include" && \
	export CI=1 COOKBOOK_CLEAN_BUILD=true COOKBOOK_HOST_SYSROOT=/usr COOKBOOK_CROSS_TARGET=$(HOST_TARGET) && \
	./target/release/repo cook relibc
	cp -r "$(RELIBC_FREESTANDING_TARGET)/stage/usr/". "$@.partial/$(TARGET)"
	touch "$@.partial"
	mv "$@.partial" "$@"
endif

$(PREFIX)/gcc-install: $(PREFIX)/relibc-freestanding-install $(PREFIX)/binutils-install $(PREFIX)/libtool-install | $(FSTOOLS_TAG) $(CONTAINER_TAG)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	@echo "\033[1;36;49mBuilding gcc-install\033[0m"
	rm -rf "$@.partial" "$@-build.partial" "$@"
	if [ ! -d "$(ROOT)/$(GCC_TARGET)" ]; then \
		echo "\033[1;38;5;196m Incomplete build stages. Please re-run the build\033[0m"; \
		rm -rf "$(PREFIX)"/gcc-freestanding-install && "$(PREFIX)"/relibc-freestanding-install && \
		exit 1; fi
	mkdir -p "$@.partial" "$@-build.partial"
	cp -r "$(PREFIX)/gcc-freestanding-install/". "$@.partial"
	cp -r "$(PREFIX)/relibc-freestanding-install/". "$@.partial"
	cp -r "$(PREFIX)/binutils-install/". "$@.partial"
	cp -r "$(PREFIX)/libtool-install/". "$@.partial"
# libgcc
	export PATH="$(ROOT)/$@.partial/bin:$$PATH" && \
	$(MAKE) -C "$(ROOT)/$(GCC_TARGET)/build" all-target-libgcc && \
	$(MAKE) -C "$(ROOT)/$(GCC_TARGET)/build" install-target-libgcc DESTDIR="$(ROOT)/$@-build.partial/usr"
	cp -r "$@-build.partial/usr/". "$@.partial"
# libstdcxx, bare features
	export PATH="$(ROOT)/$@.partial/bin:$$PATH" && \
	export CI=1 COOKBOOK_CLEAN_BUILD=true "COOKBOOK_HOST_SYSROOT=$(ROOT)/$@.partial" COOKBOOK_CROSS_TARGET=$(HOST_TARGET) COOKBOOK_CROSS_GNU_TARGET=$(HOST_GNU_TARGET) && \
	./target/release/repo cook libstdcxx-v3
	cp -r "$(LIBSTDCXX_TARGET)/stage/usr/". "$@.partial"
# libstdcxx, full features
	export PATH="$(ROOT)/$@.partial/bin:$$PATH" && \
	export CI=1 COOKBOOK_CLEAN_BUILD=true "COOKBOOK_HOST_SYSROOT=$(ROOT)/$@.partial" COOKBOOK_CROSS_TARGET=$(HOST_TARGET) && \
	rm -rf "$(LIBSTDCXX_TARGET)/stage" && ./target/release/repo cook libstdcxx-v3
	cp -r "$(LIBSTDCXX_TARGET)/stage/usr/". "$@.partial/$(GNU_TARGET)"
	rm -rf "$@-build.partial"
	touch "$@.partial"
	mv "$@.partial" "$@"
# no longer needed, delete to save disk space
	rm -rf $(BINUTILS_TARGET) $(LIBTOOL_TARGET) $(GCC_TARGET) $(LIBSTDCXX_TARGET) $(RELIBC_FREESTANDING_TARGET)
endif

$(PREFIX)/gcc-install.tar.gz: $(PREFIX)/gcc-install
	tar \
		--create \
		--gzip \
		--file "$@" \
		--directory="$<" \
		.

# RUST FROM UPSTREAM COMPILER ---------------------------------------------------
ifeq ($(PREFIX_USE_UPSTREAM_RUST_COMPILER),1)

PREFIX_RUST_VERSION_TAG=$(PREFIX)/rustc-version-tag-$(UPSTREAM_RUSTC_VERSION)

$(PREFIX_RUST_VERSION_TAG):
	rm -f "$(PREFIX)"/rustc-version-tag-*
	rm -f "$(PREFIX)"/rustc-install.tar.xz
	rm -f "$(PREFIX)"/cargo-install.tar.xz
	rm -f "$(PREFIX)"/rust-std-host-install.tar.xz
	rm -f "$(PREFIX)"/rust-std-target-install.tar.xz
	rm -f "$(PREFIX)"/rust-src-install.tar.xz:
	touch $@

$(PREFIX)/rustc-install.tar.xz: | $(PREFIX_RUST_VERSION_TAG)
	mkdir -p "$(@D)"
	wget -O $@.partial "https://static.rust-lang.org/dist/$(UPSTREAM_RUSTC_VERSION)/rustc-nightly-$(HOST_TARGET).tar.xz"
	mv $@.partial $@

$(PREFIX)/cargo-install.tar.xz: | $(PREFIX_RUST_VERSION_TAG)
	mkdir -p "$(@D)"
	wget -O $@.partial "https://static.rust-lang.org/dist/$(UPSTREAM_RUSTC_VERSION)/cargo-nightly-$(HOST_TARGET).tar.xz"
	mv $@.partial $@

$(PREFIX)/rust-std-host-install.tar.xz: | $(PREFIX_RUST_VERSION_TAG)
	mkdir -p "$(@D)"
	wget -O $@.partial "https://static.rust-lang.org/dist/$(UPSTREAM_RUSTC_VERSION)/rust-std-nightly-$(HOST_TARGET).tar.xz"
	mv $@.partial $@

$(PREFIX)/rust-std-target-install.tar.xz: | $(PREFIX_RUST_VERSION_TAG)
	mkdir -p "$(@D)"
ifeq ($(TARGET),x86_64-unknown-redox)
	wget -O $@.partial "https://static.rust-lang.org/dist/$(UPSTREAM_RUSTC_VERSION)/rust-std-nightly-$(TARGET).tar.xz"
	mv $@.partial $@
else
	touch $@
endif

$(PREFIX)/rust-src-install.tar.xz: | $(PREFIX_RUST_VERSION_TAG)
	mkdir -p "$(@D)"
	wget -O $@.partial "https://static.rust-lang.org/dist/$(UPSTREAM_RUSTC_VERSION)/rust-src-nightly.tar.xz"
	mv $@.partial $@

$(PREFIX)/rust-install: $(PREFIX)/gcc-install $(PREFIX)/rustc-install.tar.xz $(PREFIX)/cargo-install.tar.xz $(PREFIX)/rust-std-host-install.tar.xz $(PREFIX)/rust-std-target-install.tar.xz $(PREFIX)/rust-src-install.tar.xz
	@echo "\033[1;36;49mBuilding rust-install\033[0m"
	rm -rf "$@.partial" "$@"
	mkdir -p "$@.partial"
	cp -r "$(PREFIX)/gcc-install/". "$@.partial"
	tar --extract --file "$(PREFIX)/rustc-install.tar.xz" -C "$@.partial" rustc-nightly-$(HOST_TARGET)/rustc/ --strip-components=2
	tar --extract --file "$(PREFIX)/cargo-install.tar.xz" --directory "$@.partial" cargo-nightly-$(HOST_TARGET)/cargo/ --strip-components=2
	tar --extract --file "$(PREFIX)/rust-std-host-install.tar.xz" --directory "$@.partial" rust-std-nightly-$(HOST_TARGET)/rust-std-$(HOST_TARGET)/ --strip-components=2
	tar --extract --file "$(PREFIX)/rust-src-install.tar.xz" --directory "$@.partial" rust-src-nightly/rust-src/ --strip-components=2
ifeq ($(TARGET),x86_64-unknown-redox)
	tar --extract --file "$(PREFIX)/rust-std-target-install.tar.xz" --directory "$@.partial" rust-std-nightly-$(TARGET)/rust-std-$(TARGET)/ --strip-components=2
endif
	rm -f "$@.partial/manifest.in"
	touch "$@.partial"
	mv "$@.partial" "$@"

# BUILD RUST ---------------------------------------------------
else 

$(ROOT)/rust/configure:
	git submodule sync --recursive
	git submodule update --progress --init --recursive --checkout rust

$(PREFIX)/rust-install: $(ROOT)/rust/configure | $(PREFIX)/gcc-install $(PREFIX)/relibc-freestanding-install $(CONTAINER_TAG)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	@echo "\033[1;36;49mBuilding rust-install\033[0m"
	rm -rf "$(PREFIX)/rust-build" "$@.partial" "$@"
	mkdir -p "$(PREFIX)/rust-build"
	cp -r "$(PREFIX)/gcc-install" "$@.partial"
	cp -r "$(PREFIX)/relibc-freestanding-install/$(GNU_TARGET)" "$@.partial"
	cd "$(PREFIX)/rust-build" && \
	export PATH="$(ROOT)/$@.partial/bin:$$PATH" && \
	"$<" \
		--prefix="/" \
		--disable-docs \
		--disable-download-ci-llvm \
		--enable-cargo-native-static \
		--enable-dist-src \
		--enable-extended \
		--enable-lld \
		--enable-llvm-static-stdcpp \
		--tools=cargo,src \
		--target="$(HOST_TARGET),$(TARGET)" \
		&& \
	$(MAKE) -j `$(NPROC)` && \
	rm -rf $(ROOT)/$@.partial/lib/rustlib/{components,install.log,rust-installer-version,uninstall.sh,manifest-*} "$(ROOT)/$@.partial/share/doc/rust" && \
	$(MAKE) -j `$(NPROC)` install DESTDIR="$(ROOT)/$@.partial"
	rm -rf "$(PREFIX)/rust-build"
	mkdir -p "$@.partial/lib/rustlib/$(HOST_TARGET)/bin"
	mkdir -p "$@.partial/lib/rustlib/$(HOST_TARGET)/lib"
	cd "$@.partial" && find . -name *.old -exec rm {} ';' && $(PREFIX_STRIP)
	touch "$@.partial"
	mv "$@.partial" "$@"
endif

endif

$(PREFIX)/rust-install.tar.gz: $(PREFIX)/rust-install
	tar \
		--create \
		--gzip \
		--file "$@" \
		--directory="$<" \
		.
endif
