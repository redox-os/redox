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
LLVM_TARGET=recipes/dev/llvm21/target/$(HOST_TARGET)/$(TARGET)
RUST_TARGET=recipes/dev/rust/target/$(HOST_TARGET)/$(TARGET)
CLANG_TARGET=recipes/dev/clang21/target/$(HOST_TARGET)/$(TARGET)
LLD_TARGET=recipes/dev/lld21/target/$(HOST_TARGET)/$(TARGET)

# official RISC-V support introduced in newer version
UPSTREAM_RUSTC_VERSION=2025-11-15

export PREFIX_RUSTFLAGS=-L $(ROOT)/$(PREFIX_INSTALL)/$(TARGET)/lib
export RUSTUP_TOOLCHAIN=$(ROOT)/$(PREFIX_INSTALL)
export REDOXER_TOOLCHAIN=$(RUSTUP_TOOLCHAIN)
PREFIX_CONFIG=CI=1 COOKBOOK_CLEAN_BUILD=true COOKBOOK_CLEAN_TARGET=false COOKBOOK_VERBOSE=true COOKBOOK_NONSTOP=false

prefix: $(PREFIX)/sysroot

# Remove prefix builds but retain downloaded binaries
prefix_clean:
	rm -rf $(PREFIX)/sysroot $(PREFIX)/*-install

# Remove relibc in sysroot and all statically linked recipes
static_clean: | $(FSTOOLS_TAG)
	$(MAKE) c.relibc
	$(MAKE) c.base,base-initfs,extrautils,kernel,ion,pkgutils,redoxfs
	$(MAKE) c.bash,luajit,gettext,openssl1,pcre2,sdl1,zstd,zlib,bzip2,xz
	$(MAKE) c.expat,freetype2,libffi,libiconv,libjpeg,liborbital,libpng,libxml2,ncurses,ncursesw

$(PREFIX)/relibc-install: $(PREFIX)/clang-install $(PREFIX)/rust-install $(PREFIX)/gcc-install | $(FSTOOLS_TAG) $(CONTAINER_TAG)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	@echo "\033[1;36;49mBuilding relibc-install\033[0m"
	rm -rf "$@.partial" "$@"
	mkdir "$@.partial"
	cp -r "$(PREFIX)/gcc-install/". "$@.partial"
	cp -r "$(PREFIX)/rust-install/". "$@.partial"
	cp -r "$(PREFIX)/clang-install/". "$@.partial"
	rm -rf "$@.partial/$(GNU_TARGET)/include/"*
	cp -r "$(PREFIX)/gcc-install/$(GNU_TARGET)/include/c++" "$@.partial/$(GNU_TARGET)/include/c++"
	export PATH="$(ROOT)/$@.partial/bin:$$PATH" && \
	export CARGO="env -u CARGO cargo" $(PREFIX_CONFIG) && \
	./target/release/repo cook relibc
	cp -r "$(RELIBC_TARGET)/stage/usr/". "$@.partial/$(GNU_TARGET)"
	mkdir -p "$@.partial/$(GNU_TARGET)/usr"
	ln -s "../include" "$@.partial/$(GNU_TARGET)/usr/include"
	ln -s "../lib" "$@.partial/$(GNU_TARGET)/usr/lib"
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


$(PREFIX)/sysroot: $(PREFIX)/relibc-install $(CONTAINER_TAG)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	rm -rf "$@"
	cp -r "$(PREFIX)/relibc-install/" "$@"
# adapt path for libtoolize
	sed 's|/usr/share|$(ROOT)/$@/share|g' "$@/bin/libtoolize.orig" > "$@/bin/libtoolize"
	chmod 0755 "$@/bin/libtoolize"
	touch "$@"
endif

# PREFIX_BINARY ---------------------------------------------------
ifeq ($(PREFIX_BINARY),1)

$(PREFIX)/gcc-install.tar.gz: | $(CONTAINER_TAG)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	mkdir -p "$(@D)"
	wget -O $@.partial "https://static.redox-os.org/toolchain/$(HOST_TARGET)/$(TARGET)/gcc-install.tar.gz"
	mv $@.partial $@
endif

$(PREFIX)/gcc-install: $(PREFIX)/gcc-install.tar.gz $(CONTAINER_TAG)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	rm -rf "$@.partial" "$@"
	mkdir -p "$@.partial"
	tar --extract --file "$<" --directory "$@.partial" --no-same-owner --strip-components=1
	touch "$@.partial"
	mv "$@.partial" "$@"
endif

$(PREFIX)/rust-install.tar.gz: | $(CONTAINER_TAG)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	mkdir -p "$(@D)"
	wget -O $@.partial "https://static.redox-os.org/toolchain/$(HOST_TARGET)/$(TARGET)/rust-install.tar.gz"
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

$(PREFIX)/clang-install.tar.gz: | $(CONTAINER_TAG)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	mkdir -p "$(@D)"
	wget -O $@.partial "https://static.redox-os.org/toolchain/$(HOST_TARGET)/$(TARGET)/clang-install.tar.gz"
	mv $@.partial $@
endif

$(PREFIX)/clang-install: $(PREFIX)/clang-install.tar.gz $(CONTAINER_TAG)
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
$(PREFIX)/libtool-install: | $(FSTOOLS_TAG) $(CONTAINER_TAG)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	@echo "\033[1;36;49mBuilding libtool-install\033[0m"
	rm -rf "$@.partial" "$@"
	mkdir -p "$@.partial"
	export $(PREFIX_CONFIG) COOKBOOK_HOST_SYSROOT=/usr && \
	./target/release/repo cook host:libtool
	cp -r "$(LIBTOOL_TARGET)/stage/usr/". "$@.partial"
	mv "$@.partial/bin/libtoolize" "$@.partial/bin/libtoolize.orig"
# adapt path for libtoolize
	sed 's|/usr/share|$(ROOT)/$@/share|g' "$@.partial/bin/libtoolize.orig" > "$@.partial/bin/libtoolize"
	chmod 0755 "$@.partial/bin/libtoolize"
	touch "$@.partial"
	mv "$@.partial" "$@"
endif

$(PREFIX)/binutils-install: | $(PREFIX)/libtool-install $(FSTOOLS_TAG) $(CONTAINER_TAG)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	@echo "\033[1;36;49mBuilding binutils-install\033[0m"
	rm -rf "$@.partial" "$@"
	mkdir -p "$@.partial"
	export $(PREFIX_CONFIG) PATH="$(ROOT)/$(PREFIX)/libtool-install/bin:$$PATH" \
		COOKBOOK_HOST_SYSROOT=/usr COOKBOOK_CROSS_TARGET=$(TARGET) COOKBOOK_CROSS_GNU_TARGET=$(GNU_TARGET) && \
	./target/release/repo cook host:binutils-gdb 
	cp -r "$(BINUTILS_TARGET)/stage/usr/". "$@.partial"
	touch "$@.partial"
	mv "$@.partial" "$@"
endif

$(PREFIX)/gcc-freestanding-install: $(PREFIX)/binutils-install | $(PREFIX)/libtool-install $(FSTOOLS_TAG) $(CONTAINER_TAG)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	@echo "\033[1;36;49mBuilding gcc-freestanding-install\033[0m"
	rm -rf "$@.partial" "$@" $(PREFIX)/relibc-freestanding-install $(PREFIX)/sysroot
	mkdir -p "$@.partial" $(PREFIX)/relibc-freestanding-install/$(GNU_TARGET)/include
	export $(PREFIX_CONFIG) PATH="$(ROOT)/$(PREFIX)/libtool-install/bin:$(ROOT)/$(PREFIX)/binutils-install/bin:$$PATH" \
		COOKBOOK_LIBTOOL_DIR=$(ROOT)/$(PREFIX)/libtool-install COOKBOOK_CROSS_TARGET=$(TARGET) COOKBOOK_CROSS_GNU_TARGET=$(GNU_TARGET) \
		COOKBOOK_HOST_SYSROOT=/usr COOKBOOK_CROSS_SYSROOT=$(ROOT)/$(PREFIX)/relibc-freestanding-install/$(GNU_TARGET) && \
	./target/release/repo cook host:gcc13
	cp -r "$(GCC_TARGET)/stage/usr/". "$@.partial"
	cp -r "$(GCC_TARGET)/stage.cxx/usr/". "$@.partial"
	cp -r "$(PREFIX)/binutils-install/". "$@.partial"
	rm -rf $(PREFIX)/relibc-freestanding-install
	touch "$@.partial"
	mv "$@.partial" "$@"
endif

$(PREFIX)/relibc-freestanding-install: $(PREFIX)/gcc-freestanding-install | $(FSTOOLS_TAG) $(CONTAINER_TAG)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	@echo "\033[1;36;49mBuilding relibc-freestanding-install\033[0m"
	rm -rf "$@.partial" "$@"
	mkdir -p "$@.partial"
	export CARGO="env -u CARGO -u RUSTUP_TOOLCHAIN cargo" RUSTUP="env -u CARGO -u RUSTUP_TOOLCHAIN rustup" && \
	export PATH="$(ROOT)/$(PREFIX)/gcc-freestanding-install/bin:$$PATH" && \
	export CC_$(subst -,_,$(TARGET))="$(GNU_TARGET)-gcc -isystem $(ROOT)/$@.partial/$(GNU_TARGET)/include" LINKFLAGS="" && \
	export $(PREFIX_CONFIG) COOKBOOK_HOST_SYSROOT=/usr COOKBOOK_CROSS_TARGET=$(HOST_TARGET) && \
	./target/release/repo cook relibc
	cp -r "$(RELIBC_FREESTANDING_TARGET)/stage/usr/". "$@.partial/$(GNU_TARGET)"
	touch "$@.partial"
	mv "$@.partial" "$@"
endif

$(PREFIX)/gcc-install: $(PREFIX)/relibc-freestanding-install | $(PREFIX)/libtool-install $(FSTOOLS_TAG) $(CONTAINER_TAG)
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
	cp -r "$(PREFIX)/libtool-install/". "$@.partial"
	@#TODO: how to make this not conflict with libc?
	rm -f "$@.partial/lib/gcc/$(GNU_TARGET)/13.2.0/include/limits.h"
# libgcc and freestanding libstdcxx
	export PATH="$(ROOT)/$@.partial/bin:$$PATH" && \
	$(MAKE) -C "$(ROOT)/$(GCC_TARGET)/build" all-target-libgcc all-target-libstdc++-v3 && \
	$(MAKE) -C "$(ROOT)/$(GCC_TARGET)/build" install-target-libgcc install-target-libstdc++-v3 DESTDIR="$(ROOT)/$@-build.partial/usr"
	cp -r "$@-build.partial/usr/". "$@.partial"
	@#TODO: in riscv64gc libgcc_s.so is a GNU ld script
	rm -f "$@.partial"/$(GNU_TARGET)/lib/libgcc_s.so
	ln -s libgcc_s.so.1 "$@.partial"/$(GNU_TARGET)/lib/libgcc_s.so
	@#TODO: generates wrong lib path for libtool
	rm -f "$@.partial"/$(GNU_TARGET)/lib/libstdc++.la
	rm -f "$@.partial"/$(GNU_TARGET)/lib/libsupc++.la
# hosted libstdcxx
	export PATH="$(ROOT)/$@.partial/bin:$$PATH" && \
	export $(PREFIX_CONFIG) "COOKBOOK_HOST_SYSROOT=$(ROOT)/$@.partial" COOKBOOK_CROSS_TARGET=$(HOST_TARGET) && \
	rm -rf "$(LIBSTDCXX_TARGET)/stage" && ./target/release/repo cook libstdcxx-v3
	cp -r "$(LIBSTDCXX_TARGET)/stage/usr/". "$@.partial/$(GNU_TARGET)"
	rm -rf "$@-build.partial"
	touch "$@.partial"
	mv "$@.partial" "$@"
# no longer needed, delete build files to save disk space
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
	mkdir -p "$(@D)"
	touch $@

$(PREFIX)/rustc-install.tar.xz: | $(PREFIX_RUST_VERSION_TAG)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	mkdir -p "$(@D)"
	wget -O $@.partial "https://static.rust-lang.org/dist/$(UPSTREAM_RUSTC_VERSION)/rustc-nightly-$(HOST_TARGET).tar.xz"
	mv $@.partial $@
endif

$(PREFIX)/cargo-install.tar.xz: | $(PREFIX_RUST_VERSION_TAG)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	mkdir -p "$(@D)"
	wget -O $@.partial "https://static.rust-lang.org/dist/$(UPSTREAM_RUSTC_VERSION)/cargo-nightly-$(HOST_TARGET).tar.xz"
	mv $@.partial $@
endif

$(PREFIX)/rust-std-host-install.tar.xz: | $(PREFIX_RUST_VERSION_TAG)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	mkdir -p "$(@D)"
	wget -O $@.partial "https://static.rust-lang.org/dist/$(UPSTREAM_RUSTC_VERSION)/rust-std-nightly-$(HOST_TARGET).tar.xz"
	mv $@.partial $@
endif

$(PREFIX)/rust-std-target-install.tar.xz: | $(PREFIX_RUST_VERSION_TAG)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	mkdir -p "$(@D)"
ifeq ($(TARGET),x86_64-unknown-redox)
	wget -O $@.partial "https://static.rust-lang.org/dist/$(UPSTREAM_RUSTC_VERSION)/rust-std-nightly-$(TARGET).tar.xz"
	mv $@.partial $@
else
	touch $@
endif
endif

$(PREFIX)/rust-src-install.tar.xz: | $(PREFIX_RUST_VERSION_TAG)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	mkdir -p "$(@D)"
	wget -O $@.partial "https://static.rust-lang.org/dist/$(UPSTREAM_RUSTC_VERSION)/rust-src-nightly.tar.xz"
	mv $@.partial $@
endif

$(PREFIX)/rust-install: $(PREFIX)/rustc-install.tar.xz $(PREFIX)/cargo-install.tar.xz $(PREFIX)/rust-std-host-install.tar.xz $(PREFIX)/rust-std-target-install.tar.xz $(PREFIX)/rust-src-install.tar.xz
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	@echo "\033[1;36;49mBuilding rust-install\033[0m"
	rm -rf "$@.partial" "$@"
	mkdir -p "$@.partial"
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
endif

# BUILD RUST ---------------------------------------------------
else

$(PREFIX)/rust-install: | $(PREFIX)/libtool-install $(FSTOOLS_TAG) $(CONTAINER_TAG)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	@echo "\033[1;36;49mBuilding rust-install\033[0m"
	rm -rf "$@.partial" "$@"
	export PATH="$(ROOT)/$(PREFIX)/libtool-install/bin:$$PATH" \
		$(PREFIX_CONFIG) COOKBOOK_HOST_SYSROOT=/usr COOKBOOK_CROSS_TARGET=$(TARGET) && \
		./target/release/repo cook host:llvm21 host:rust
	cp -r "$(RUST_TARGET)/stage/usr/". "$@.partial"
	cp -r "$(LLVM_TARGET)/stage/usr/". "$@.partial"
	mv "$@.partial" "$@"
# TODO: Cache from RUST_TARGET is currently not cleared.
# TIP: If you're developing std for rust, remove COOKBOOK_CLEAN_BUILD=true 
#      at the top of this file so your next rust build reuses the build cache
endif

endif

$(PREFIX)/rust-install.tar.gz: $(PREFIX)/rust-install
	tar \
		--create \
		--gzip \
		--file "$@" \
		--directory="$<" \
		.

# BUILD CLANG ---------------------------------------------------
$(PREFIX)/clang-install: | $(PREFIX)/rust-install $(PREFIX)/libtool-install $(FSTOOLS_TAG) $(CONTAINER_TAG)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	@echo "\033[1;36;49mBuilding clang-install\033[0m"
	rm -rf "$@.partial" "$@"
	export PATH="$(ROOT)/$(PREFIX)/libtool-install/bin:$$PATH" \
		$(PREFIX_CONFIG) COOKBOOK_HOST_SYSROOT=/usr COOKBOOK_CROSS_TARGET=$(TARGET) && \
		./target/release/repo cook host:llvm21 host:clang21 host:lld21
# skipping dev, llvm libraries is already in rust if building
ifeq ($(PREFIX_USE_UPSTREAM_RUST_COMPILER),1)
	cp -r "$(LLVM_TARGET)/stage/usr/". "$@.partial"
endif
	cp -r "$(LLVM_TARGET)/stage.runtime/usr/". "$@.partial"
	cp -r "$(CLANG_TARGET)/stage/usr/". "$@.partial"
	cp -r "$(LLD_TARGET)/stage/usr/". "$@.partial"
	mv "$@.partial" "$@"
# no longer needed, delete build files to save disk space
	rm -rf $(LLVM_TARGET) $(CLANG_TARGET) $(LLD_TARGET)
endif

$(PREFIX)/clang-install.tar.gz: $(PREFIX)/clang-install
	tar \
		--create \
		--gzip \
		--file "$@" \
		--directory="$<" \
		.

endif
