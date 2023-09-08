PREFIX=prefix/$(TARGET)

PREFIX_INSTALL=$(PREFIX)/relibc-install
PREFIX_PATH=$(ROOT)/$(PREFIX_INSTALL)/bin

BINUTILS_BRANCH=redox-2.41
GCC_BRANCH=redox-13.2.0

export PREFIX_RUSTFLAGS=-L $(ROOT)/$(PREFIX_INSTALL)/$(TARGET)/lib
export RUSTUP_TOOLCHAIN=$(ROOT)/$(PREFIX_INSTALL)
export REDOXER_TOOLCHAIN=$(RUSTUP_TOOLCHAIN)

export CC=
export CXX=

prefix: $(PREFIX_INSTALL)

PREFIX_STRIP=\
	mkdir -p bin libexec "$(TARGET)/bin" && \
	find bin libexec "$(TARGET)/bin" "$(TARGET)/lib" \
		-type f \
		-exec strip --strip-unneeded {} ';' \
		2> /dev/null

$(PREFIX)/relibc-install: $(ROOT)/relibc | $(PREFIX)/rust-install $(CONTAINER_TAG)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) $(MAKE) $@
else
	rm -rf "$@.partial" "$@"
	cp -r "$(PREFIX)/rust-install" "$@.partial"
	rm -rf "$@.partial/$(TARGET)/include/"*
	cp -r "$(PREFIX)/rust-install/$(TARGET)/include/c++" "$@.partial/$(TARGET)/include/c++"
	cp -r "$(PREFIX)/rust-install/lib/rustlib/$(HOST_TARGET)/lib/" "$@.partial/lib/rustlib/$(HOST_TARGET)/"
# Temporary hack to avoid breaking stuff before new pre-build prefix tarball with fixed rustlib/src is build and served at https://static.redox-os.org/toolchain/$(TARGET)/relibc-install.tar.gz 
ifeq ($(PREFIX_BINARY),1)
	rm -rf $@.partial/lib/rustlib/src
	mkdir $@.partial/lib/rustlib/src
	ln -s $(ROOT)/rust $@.partial/lib/rustlib/src
endif
	cd "$<" && \
	export PATH="$(ROOT)/$@.partial/bin:$$PATH" && \
	export CARGO="env -u CARGO cargo" && \
	$(MAKE) -j `$(NPROC)` all && \
	$(MAKE) -j `$(NPROC)` install DESTDIR="$(ROOT)/$@.partial/$(TARGET)"
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

PREFIX_BASE_INSTALL=$(PREFIX)/rust-freestanding-install
PREFIX_FREESTANDING_INSTALL=$(PREFIX)/gcc-freestanding-install

PREFIX_BASE_PATH=$(ROOT)/$(PREFIX_BASE_INSTALL)/bin
PREFIX_FREESTANDING_PATH=$(ROOT)/$(PREFIX_FREESTANDING_INSTALL)/bin

$(PREFIX)/binutils-$(BINUTILS_BRANCH).tar.bz2:
	mkdir -p "$(@D)"
	rm -fv $(PREFIX)/binutils*.tar.bz2*
	wget -O $@.partial "https://gitlab.redox-os.org/redox-os/binutils-gdb/-/archive/$(BINUTILS_BRANCH)/binutils-gdb-$(BINUTILS_BRANCH).tar.bz2"
	mv $@.partial $@

$(PREFIX)/binutils: $(PREFIX)/binutils.tar.bz2
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
		--target="$(TARGET)" \
		--program-prefix="$(TARGET)-" \
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
		--target="$(TARGET)" \
		--program-prefix="$(TARGET)-" \
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

$(PREFIX)/rust-freestanding-install: $(ROOT)/rust | $(PREFIX)/binutils-install $(CONTAINER_TAG)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) $(MAKE) $@
else
	rm -rf "$(PREFIX)/rust-freestanding-build" "$@.partial" "$@"
	mkdir -p "$(PREFIX)/rust-freestanding-build"
	cp -r "$(PREFIX)/binutils-install" "$@.partial"
	cd "$(PREFIX)/rust-freestanding-build" && \
	export PATH="$(ROOT)/$@.partial/bin:$$PATH" && \
	"$</configure" \
		--prefix="/" \
		--disable-docs \
		--disable-download-ci-llvm \
		--enable-cargo-native-static \
		--enable-extended \
		--enable-lld \
		--enable-llvm-static-stdcpp \
		--set 'llvm.targets=AArch64;X86' \
		--set 'llvm.experimental-targets=' \
		--tools=cargo,src \
		&& \
	$(MAKE) -j `$(NPROC)` && \
	$(MAKE) -j `$(NPROC)` install DESTDIR="$(ROOT)/$@.partial"
	rm -rf "$(PREFIX)/rust-freestanding-build"
	mkdir -p "$@.partial/lib/rustlib/$(HOST_TARGET)/bin"
	cd "$@.partial" && $(PREFIX_STRIP)
	touch "$@.partial"
	mv "$@.partial" "$@"
endif

$(PREFIX)/relibc-freestanding-install: $(ROOT)/relibc | $(PREFIX_BASE_INSTALL) $(PREFIX_FREESTANDING_INSTALL) $(CONTAINER_TAG)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) $(MAKE) $@
else
	rm -rf "$@.partial" "$@"
	mkdir -p "$@.partial"
	cd "$<" && \
	export PATH="$(PREFIX_BASE_PATH):$(PREFIX_FREESTANDING_PATH):$$PATH" && \
	export CARGO="env -u CARGO -u RUSTUP_TOOLCHAIN cargo" && \
	export CC_$(subst -,_,$(TARGET))="$(TARGET)-gcc -isystem $(ROOT)/$@.partial/$(TARGET)/include" && \
	$(MAKE) -j `$(NPROC)` all && \
	$(MAKE) -j `$(NPROC)` install DESTDIR="$(ROOT)/$@.partial/$(TARGET)"
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
		--target="$(TARGET)" \
		--program-prefix="$(TARGET)-" \
		--prefix="" \
		--with-sysroot \
		--with-build-sysroot="$(ROOT)/$(PREFIX)/relibc-freestanding-install/$(TARGET)" \
		--with-native-system-header-dir="/include" \
		--disable-multilib \
		--disable-nls \
		--disable-werror \
		--enable-languages=c,c++ \
		--enable-shared \
		--enable-threads=posix \
		&& \
	$(MAKE) -j `$(NPROC)` all-gcc all-target-libgcc all-target-libstdc++-v3 && \
	$(MAKE) -j `$(NPROC)` install-gcc install-target-libgcc install-target-libstdc++-v3 DESTDIR="$(ROOT)/$@.partial" && \
	rm $(ROOT)/$@.partial/$(TARGET)/lib/*.la
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

$(PREFIX)/rust-install: $(ROOT)/rust | $(PREFIX)/gcc-install $(PREFIX)/relibc-freestanding-install $(CONTAINER_TAG)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) $(MAKE) $@
else
	rm -rf "$(PREFIX)/rust-build" "$@.partial" "$@"
	mkdir -p "$(PREFIX)/rust-build"
	cp -r "$(PREFIX)/gcc-install" "$@.partial"
	cp -r "$(PREFIX)/relibc-freestanding-install/$(TARGET)" "$@.partial"
	cd "$(PREFIX)/rust-build" && \
	export PATH="$(ROOT)/$@.partial/bin:$$PATH" && \
	"$</configure" \
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
