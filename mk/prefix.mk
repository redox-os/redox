# Configuration file for the Rust/GCC cross-compilers, relibc and libtool

PREFIX=prefix/$(TARGET)

PREFIX_INSTALL=$(PREFIX)/sysroot/
PREFIX_PATH=$(ROOT)/$(PREFIX_INSTALL)/bin

BINUTILS_BRANCH=redox-2.43.1
GCC_BRANCH=redox-13.2.0
LIBTOOL_VERSION=2.5.4

export PREFIX_RUSTFLAGS=-L $(ROOT)/$(PREFIX_INSTALL)/$(TARGET)/lib
export RUSTUP_TOOLCHAIN=$(ROOT)/$(PREFIX_INSTALL)
export REDOXER_TOOLCHAIN=$(RUSTUP_TOOLCHAIN)

export CC=
export CXX=

ifeq ($(TARGET),riscv64gc-unknown-redox)
	GCC_ARCH?=--with-arch=rv64gc --with-abi=lp64d
else
	GCC_ARCH?=
endif

# TODO(andypython): Upstream libtool patches to remove the need to locally build libtool.
# Cannot be CI built, i.e. be a part of relibc-install.tar.gz, as the prefix has to be correctly
# set while building. Otherwise aclocal will not be able to find libtool's files. Furthermore, doing
# so would break non-podman builds (not sure if they are still supported though).
prefix: $(PREFIX)/sysroot

PREFIX_STRIP=\
	mkdir -p bin libexec "$(GCC_TARGET)/bin" && \
	find bin libexec "$(GCC_TARGET)/bin" "$(GCC_TARGET)/lib" \
		-type f \
		-exec strip --strip-unneeded {} ';' \
		2> /dev/null

$(PREFIX)/relibc: $(ROOT)/relibc
	mkdir -p "$(@D)"
	rm -rf "$@.partial" "$@"
	cp -r "$^" "$@.partial"
	touch "$@.partial"
	mv "$@.partial" "$@"

$(PREFIX)/relibc-install: $(PREFIX)/relibc | $(PREFIX)/rust-install $(CONTAINER_TAG)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) $(MAKE) $@
else
	rm -rf "$@.partial" "$@"
	cp -r "$(PREFIX)/rust-install" "$@.partial"
	rm -rf "$@.partial/$(TARGET)/include/"*
	cp -r "$(PREFIX)/rust-install/$(GNU_TARGET)/include/c++" "$@.partial/$(GNU_TARGET)/include/c++"
	cp -r "$(PREFIX)/rust-install/lib/rustlib/$(HOST_TARGET)/lib/" "$@.partial/lib/rustlib/$(HOST_TARGET)/"
	cd "$<" && \
	export PATH="$(ROOT)/$@.partial/bin:$$PATH" && \
	export CARGO="env -u CARGO cargo" && \
	$(MAKE) clean && \
	$(MAKE) -j `$(NPROC)` all && \
	$(MAKE) -j `$(NPROC)` install DESTDIR="$(ROOT)/$@.partial/$(GNU_TARGET)"
	cd "$@.partial" && $(PREFIX_STRIP)
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

$(PREFIX)/libtool:
	rm -rf "$@.partial" "$@"
	mkdir -p "$@.partial"

	git clone \
		--recurse-submodules \
		"https://gitlab.redox-os.org/andypython/libtool/" \
		--branch "v$(LIBTOOL_VERSION)-redox" \
		--depth 1 \
		"$@.partial"

	touch "$@.partial"
	mv "$@.partial" "$@"

$(PREFIX)/libtool-build: $(PREFIX)/libtool $(CONTAINER_TAG)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) $(MAKE) $@
else
	mkdir -p "$@.partial"
	cd "$(PREFIX)/libtool" && \
		./bootstrap \
			--skip-po \
			--skip-git \
			--force \
			--gnulib-srcdir=./gnulib
	cd "$@.partial" && \
	"$(ROOT)/$</configure" \
		--target="$(TARGET)" \
		--prefix=$(abspath $(PREFIX)/sysroot) \
		&& \
	$(MAKE) -j `$(NPROC)`
	touch "$@.partial"
	mv "$@.partial" "$@"
endif

$(PREFIX)/sysroot: $(PREFIX)/relibc-install $(PREFIX)/libtool-build $(CONTAINER_TAG)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) $(MAKE) $@
else
	cp -r "$(PREFIX)/relibc-install/" "$@"
	cd "$(PREFIX)/libtool-build" && \
		$(MAKE) install -j `$(NPROC)`
	cd "$@" && $(PREFIX_STRIP)
endif

ifeq ($(PREFIX_BINARY),1)

$(PREFIX)/rust-install.tar.gz:
	mkdir -p "$(@D)"
	#TODO: figure out why rust-install.tar.gz is missing /lib/rustlib/$(HOST_TARGET)/lib
	wget -O $@.partial "https://static.redox-os.org/toolchain/$(TARGET)/relibc-install.tar.gz"
	mv $@.partial $@

$(PREFIX)/rust-install: $(PREFIX)/rust-install.tar.gz
	rm -rf "$@.partial" "$@"
	mkdir -p "$@.partial"
	tar --extract --file "$<" --directory "$@.partial" --strip-components=1
	touch "$@.partial"
	mv "$@.partial" "$@"

else

$(ROOT)/rust/configure:
	git submodule update --progress --init --recursive --checkout rust

PREFIX_BASE_INSTALL=$(PREFIX)/rust-freestanding-install
PREFIX_FREESTANDING_INSTALL=$(PREFIX)/gcc-freestanding-install

PREFIX_BASE_PATH=$(ROOT)/$(PREFIX_BASE_INSTALL)/bin
PREFIX_FREESTANDING_PATH=$(ROOT)/$(PREFIX_FREESTANDING_INSTALL)/bin


$(PREFIX)/binutils-$(BINUTILS_BRANCH).tar.bz2:
	mkdir -p "$(@D)"
	rm -fv $(PREFIX)/binutils*.tar.bz2*
	wget -O $@.partial "https://gitlab.redox-os.org/redox-os/binutils-gdb/-/archive/$(BINUTILS_BRANCH)/binutils-gdb-$(BINUTILS_BRANCH).tar.bz2"
	mv $@.partial $@

$(PREFIX)/binutils: $(PREFIX)/binutils-$(BINUTILS_BRANCH).tar.bz2
	rm -rf "$@.partial" "$@"
	mkdir -p "$@.partial"
	tar --extract --file "$<" --directory "$@.partial" --strip-components=1
	touch "$@.partial"
	mv "$@.partial" "$@"

$(PREFIX)/binutils-install: $(PREFIX)/binutils $(CONTAINER_TAG)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) $(MAKE) $@
else
	rm -rf "$<-build" "$@.partial" "$@"
	mkdir -p "$<-build" "$@.partial"
	cd "$<-build" && \
	"$(ROOT)/$</configure" \
		--target="$(GNU_TARGET)" \
		$(GCC_ARCH) \
		--program-prefix="$(GNU_TARGET)-" \
		--prefix="" \
		--disable-werror \
		&& \
	$(MAKE) -j `$(NPROC)` all && \
	$(MAKE) -j `$(NPROC)` install DESTDIR="$(ROOT)/$@.partial"
	rm -rf "$<-build"
	cd "$@.partial" && $(PREFIX_STRIP)
	touch "$@.partial"
	mv "$@.partial" "$@"
endif

$(PREFIX)/gcc-$(GCC_BRANCH).tar.bz2:
	mkdir -p "$(@D)"
	rm -fv $(PREFIX)/gcc*.tar.bz2*
	wget -O $@.partial "https://gitlab.redox-os.org/redox-os/gcc/-/archive/$(GCC_BRANCH)/gcc-$(GCC_BRANCH).tar.bz2"
	mv "$@.partial" "$@"

$(PREFIX)/gcc: $(PREFIX)/gcc-$(GCC_BRANCH).tar.bz2
	mkdir -p "$@.partial"
	tar --extract --file "$<" --directory "$@.partial" --strip-components=1
	cd "$@.partial" && ./contrib/download_prerequisites
	touch "$@.partial"
	mv "$@.partial" "$@"

$(PREFIX)/gcc-freestanding-install: $(PREFIX)/gcc | $(PREFIX)/binutils-install $(CONTAINER_TAG)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) $(MAKE) $@
else
	rm -rf "$<-freestanding-build" "$@.partial" "$@"
	mkdir -p "$<-freestanding-build"
	cp -r "$(PREFIX)/binutils-install" "$@.partial"
	cd "$<-freestanding-build" && \
	export PATH="$(ROOT)/$@.partial/bin:$$PATH" && \
	"$(ROOT)/$</configure" \
		--target="$(GNU_TARGET)" \
		$(GCC_ARCH) \
		--program-prefix="$(GNU_TARGET)-" \
		--prefix="" \
		--disable-nls \
		--enable-languages=c,c++ \
		--without-headers \
		&& \
	$(MAKE) -j `$(NPROC)` all-gcc && \
	$(MAKE) -j `$(NPROC)` install-gcc DESTDIR="$(ROOT)/$@.partial"
	rm -rf "$<-freestanding-build"
	cd "$@.partial" && $(PREFIX_STRIP)
	touch "$@.partial"
	mv "$@.partial" "$@"
endif

$(PREFIX)/rust-freestanding-install: $(ROOT)/rust/configure | $(PREFIX)/binutils-install $(CONTAINER_TAG)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) $(MAKE) $@
else
	rm -rf "$(PREFIX)/rust-freestanding-build" "$@.partial" "$@"
	mkdir -p "$(PREFIX)/rust-freestanding-build"
	cp -r "$(PREFIX)/binutils-install" "$@.partial"
	cd "$(PREFIX)/rust-freestanding-build" && \
	export PATH="$(ROOT)/$@.partial/bin:$$PATH" && \
	"$<" \
		--prefix="/" \
		--disable-docs \
		--disable-download-ci-llvm \
		--enable-cargo-native-static \
		--enable-extended \
		--enable-lld \
		--enable-llvm-static-stdcpp \
		--set 'llvm.targets=AArch64;X86;RISCV' \
		--set 'llvm.experimental-targets=' \
		--tools=cargo,src \
		&& \
	$(MAKE) -j `$(NPROC)` && \
	$(MAKE) -j `$(NPROC)` install DESTDIR="$(ROOT)/$@.partial"
	rm -rf "$(PREFIX)/rust-freestanding-build"
	mkdir -p "$@.partial/lib/rustlib/$(HOST_TARGET)/bin"
	mkdir -p "$@.partial/lib/rustlib/$(HOST_TARGET)/lib"
	cd "$@.partial" && $(PREFIX_STRIP)
	touch "$@.partial"
	mv "$@.partial" "$@"
endif

$(PREFIX)/relibc-freestanding: $(ROOT)/relibc
	mkdir -p "$(@D)"
	rm -rf "$@.partial" "$@"
	cp -r "$^" "$@.partial"
	touch "$@.partial"
	mv "$@.partial" "$@"


$(PREFIX)/relibc-freestanding-install: $(PREFIX)/relibc-freestanding | $(PREFIX_BASE_INSTALL) $(PREFIX_FREESTANDING_INSTALL) $(CONTAINER_TAG)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) $(MAKE) $@
else
	rm -rf "$@.partial" "$@"
	mkdir -p "$@.partial"
	cd "$<" && \
	export PATH="$(PREFIX_BASE_PATH):$(PREFIX_FREESTANDING_PATH):$$PATH" && \
	export CARGO="env -u CARGO -u RUSTUP_TOOLCHAIN cargo" && \
	export CC_$(subst -,_,$(TARGET))="$(GNU_TARGET)-gcc -isystem $(ROOT)/$@.partial/$(GNU_TARGET)/include" && \
	$(MAKE) clean && \
	$(MAKE) -j `$(NPROC)` all && \
	$(MAKE) -j `$(NPROC)` install DESTDIR="$(ROOT)/$@.partial/$(GNU_TARGET)"
	cd "$@.partial" && $(PREFIX_STRIP)
	touch "$@.partial"
	mv "$@.partial" "$@"
endif

$(PREFIX)/gcc-install: $(PREFIX)/gcc | $(PREFIX)/relibc-freestanding-install $(CONTAINER_TAG)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) $(MAKE) $@
else
	rm -rf "$<-build" "$@.partial" "$@"
	mkdir -p "$<-build"
	cp -r "$(PREFIX_BASE_INSTALL)" "$@.partial"
	cd "$<-build" && \
	export PATH="$(ROOT)/$@.partial/bin:$$PATH" && \
	"$(ROOT)/$</configure" \
		--target="$(GNU_TARGET)" \
		$(GCC_ARCH) \
		--program-prefix="$(GNU_TARGET)-" \
		--prefix="" \
		--with-sysroot \
		--with-build-sysroot="$(ROOT)/$(PREFIX)/relibc-freestanding-install/$(GNU_TARGET)" \
		--with-native-system-header-dir="/include" \
		--disable-multilib \
		--disable-nls \
		--disable-werror \
		--enable-languages=c,c++ \
		--enable-shared \
		--enable-threads=posix \
		&& \
	$(MAKE) -j `$(NPROC)` all-gcc all-target-libgcc all-target-libstdc++-v3 && \
	$(MAKE) -j `$(NPROC)` install-gcc install-target-libgcc install-target-libstdc++-v3 DESTDIR="$(ROOT)/$@.partial"
	rm $(ROOT)/$@.partial/$(GNU_TARGET)/lib/*.la
	rm -rf "$<-build"
	cd "$@.partial" && $(PREFIX_STRIP)
	touch "$@.partial"
	mv "$@.partial" "$@"
endif

$(PREFIX)/gcc-install.tar.gz: $(PREFIX)/gcc-install
	tar \
		--create \
		--gzip \
		--file "$@" \
		--directory="$<" \
		.

$(PREFIX)/rust-install: $(ROOT)/rust/configure | $(PREFIX)/gcc-install $(PREFIX)/relibc-freestanding-install $(CONTAINER_TAG)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) $(MAKE) $@
else
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

$(PREFIX)/rust-install.tar.gz: $(PREFIX)/rust-install
	tar \
		--create \
		--gzip \
		--file "$@" \
		--directory="$<" \
		.
endif
