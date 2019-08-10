PREFIX=prefix/$(TARGET)

PREFIX_BASE_INSTALL=$(PREFIX)/binutils-install
PREFIX_FREESTANDING_INSTALL=$(PREFIX)/gcc-freestanding-install
PREFIX_INSTALL=$(PREFIX)/relibc-install

ifeq ($(PREFIX_RUSTC),1)
	PREFIX_BASE_INSTALL=$(PREFIX)/rust-freestanding-install
	ifeq ($(PREFIX_BINARY),1)
		export RUSTUP_TOOLCHAIN=$(ROOT)/$(PREFIX)/gcc-install
	else
		export RUSTUP_TOOLCHAIN=$(ROOT)/$(PREFIX)/rust-freestanding-install
	endif
endif

PREFIX_BASE_PATH=$(ROOT)/$(PREFIX_BASE_INSTALL)/bin
PREFIX_FREESTANDING_PATH=$(ROOT)/$(PREFIX_FREESTANDING_INSTALL)/bin
PREFIX_PATH=$(ROOT)/$(PREFIX_INSTALL)/bin

export PREFIX_RUSTFLAGS=-L $(ROOT)/$(PREFIX_INSTALL)/$(TARGET)/lib

prefix-base: $(PREFIX_BASE_INSTALL)

prefix-freestanding: $(PREFIX_FREESTANDING_INSTALL)

prefix: $(PREFIX_INSTALL)

PREFIX_STRIP=\
	mkdir -p bin libexec "$(TARGET)/bin" && \
	find bin libexec "$(TARGET)/bin" "$(TARGET)/lib" \
		-type f \
		-exec strip --strip-unneeded {} ';' \
		2> /dev/null

$(PREFIX)/relibc-install: $(ROOT)/relibc | $(PREFIX)/gcc-install
	rm -rf "$@.partial" "$@"
	cp -r "$(PREFIX)/gcc-install" "$@.partial"
	cd "$<" && \
	export PATH="$(ROOT)/$@.partial/bin:$$PATH" && \
	export CARGO="env -u CARGO xargo" && \
	make -j `$(NPROC)` headers && \
	make -j `$(NPROC)` install-headers DESTDIR="$(ROOT)/$@.partial/$(TARGET)" && \
	make -j `$(NPROC)` all && \
	make -j `$(NPROC)` install DESTDIR="$(ROOT)/$@.partial/$(TARGET)"
	cd "$@.partial" && $(PREFIX_STRIP)
	touch "$@.partial"
	mv "$@.partial" "$@"

$(PREFIX)/relibc-install.tar.gz: $(PREFIX)/relibc-install
	tar \
		--create \
		--gzip \
		--file "$@" \
		--directory="$<" \
		.

$(PREFIX)/rust-install: $(ROOT)/rust | $(PREFIX)/relibc-install
	rm -rf "$(PREFIX)/rust-build" "$@.partial" "$@"
	mkdir -p "$(PREFIX)/rust-build"
	cp -r "$(PREFIX)/relibc-install" "$@.partial"
	cd "$(PREFIX)/rust-build" && \
	export PATH="$(ROOT)/$@.partial/bin:$$PATH" && \
	"$</configure" \
		--prefix="/" \
		--disable-docs \
		--enable-cargo-native-static \
		--enable-extended \
		--enable-llvm-static-stdcpp \
		--tools=cargo \
		--target="$(TARGET)" \
		&& \
	make -j `$(NPROC)` && \
	rm -rf "$(ROOT)/$@.partial/lib/rustlib" "$(ROOT)/$@.partial/share/doc/rust" && \
	make -j `$(NPROC)` install DESTDIR="$(ROOT)/$@.partial"
	rm -rf "$(PREFIX)/rust-build"
	mkdir -p "$@.partial/lib/rustlib/x86_64-unknown-linux-gnu/bin"
	cd "$@.partial" && find . -name *.old -exec rm {} ';' && $(PREFIX_STRIP)
	touch "$@.partial"
	mv "$@.partial" "$@"

$(PREFIX)/rust-install.tar.gz: $(PREFIX)/rust-install
	tar \
		--create \
		--gzip \
		--file "$@" \
		--directory="$<" \
		.

ifeq ($(PREFIX_BINARY),1)

$(PREFIX)/gcc-install.tar.gz:
	mkdir -p "$(@D)"
	wget -O $@.partial "https://static.redox-os.org/toolchain/$(TARGET)/gcc-install.tar.gz"
	mv $@.partial $@

$(PREFIX)/gcc-install: $(PREFIX)/gcc-install.tar.gz
	rm -rf "$@.partial" "$@"
	mkdir -p "$@.partial"
	tar --extract --file "$<" --directory "$@.partial" --strip-components=1
	touch "$@.partial"
	mv "$@.partial" "$@"

else

$(PREFIX)/binutils.tar.bz2:
	mkdir -p "$(@D)"
	wget -O $@.partial "https://gitlab.redox-os.org/redox-os/binutils-gdb/-/archive/redox/binutils-gdb-redox.tar.bz2"
	mv $@.partial $@

$(PREFIX)/binutils: $(PREFIX)/binutils.tar.bz2
	rm -rf "$@.partial" "$@"
	mkdir -p "$@.partial"
	tar --extract --file "$<" --directory "$@.partial" --strip-components=1
	touch "$@.partial"
	mv "$@.partial" "$@"

$(PREFIX)/binutils-install: $(PREFIX)/binutils
	rm -rf "$<-build" "$@.partial" "$@"
	mkdir -p "$<-build" "$@.partial"
	cd "$<-build" && \
	"$(ROOT)/$</configure" \
		--target="$(TARGET)" \
		--program-prefix="$(TARGET)-" \
		--prefix="" \
		--disable-werror \
		&& \
	make -j `$(NPROC)` all && \
	make -j `$(NPROC)` install DESTDIR="$(ROOT)/$@.partial"
	rm -rf "$<-build"
	cd "$@.partial" && $(PREFIX_STRIP)
	touch "$@.partial"
	mv "$@.partial" "$@"

$(PREFIX)/gcc.tar.bz2:
	mkdir -p "$(@D)"
	wget -O $@.partial "https://gitlab.redox-os.org/redox-os/gcc/-/archive/redox/gcc-redox.tar.bz2"
	mv "$@.partial" "$@"

$(PREFIX)/gcc: $(PREFIX)/gcc.tar.bz2
	mkdir -p "$@.partial"
	tar --extract --file "$<" --directory "$@.partial" --strip-components=1
	cd "$@.partial" && ./contrib/download_prerequisites
	touch "$@.partial"
	mv "$@.partial" "$@"

$(PREFIX)/gcc-freestanding-install: $(PREFIX)/gcc | $(PREFIX)/binutils-install
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
	make -j `$(NPROC)` all-gcc && \
	make -j `$(NPROC)` install-gcc DESTDIR="$(ROOT)/$@.partial"
	rm -rf "$<-freestanding-build"
	cd "$@.partial" && $(PREFIX_STRIP)
	touch "$@.partial"
	mv "$@.partial" "$@"

$(PREFIX)/rust-freestanding-install: $(ROOT)/rust | $(PREFIX)/binutils-install
	rm -rf "$(PREFIX)/rust-freestanding-build" "$@.partial" "$@"
	mkdir -p "$(PREFIX)/rust-freestanding-build"
	cp -r "$(PREFIX)/binutils-install" "$@.partial"
	cd "$(PREFIX)/rust-freestanding-build" && \
	export PATH="$(ROOT)/$@.partial/bin:$$PATH" && \
	"$</configure" \
		--prefix="/" \
		--disable-docs \
		--enable-cargo-native-static \
		--enable-extended \
		--enable-llvm-static-stdcpp \
		--tools=cargo \
		&& \
	make -j `$(NPROC)` && \
	make -j `$(NPROC)` install DESTDIR="$(ROOT)/$@.partial"
	rm -rf "$(PREFIX)/rust-freestanding-build"
	mkdir -p "$@.partial/lib/rustlib/x86_64-unknown-linux-gnu/bin"
	cd "$@.partial" && $(PREFIX_STRIP)
	touch "$@.partial"
	mv "$@.partial" "$@"

$(PREFIX)/relibc-freestanding-install: $(ROOT)/relibc | $(PREFIX_BASE_INSTALL) $(PREFIX_FREESTANDING_INSTALL)
	rm -rf "$@.partial" "$@"
	mkdir -p "$@.partial"
	cd "$<" && \
	export PATH="$(PREFIX_BASE_INSTALL):$(PREFIX_FREESTANDING_PATH):$$PATH" && \
	export CARGO="env -u CARGO xargo" && \
	make -j `$(NPROC)` headers && \
	make -j `$(NPROC)` install-headers DESTDIR="$(ROOT)/$@.partial/$(TARGET)" && \
	export CC_$(subst -,_,$(TARGET))="$(TARGET)-gcc -isystem $(ROOT)/$@.partial/$(TARGET)/include" && \
	make -j `$(NPROC)` all && \
	make -j `$(NPROC)` install DESTDIR="$(ROOT)/$@.partial/$(TARGET)"
	cd "$@.partial" && $(PREFIX_STRIP)
	touch "$@.partial"
	mv "$@.partial" "$@"

$(PREFIX)/gcc-install: $(PREFIX)/gcc | $(PREFIX)/relibc-freestanding-install
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
	make -j `$(NPROC)` all-gcc all-target-libgcc all-target-libstdc++-v3 && \
	make -j `$(NPROC)` install-gcc install-target-libgcc install-target-libstdc++-v3 DESTDIR="$(ROOT)/$@.partial" && \
	rm $(ROOT)/$@.partial/$(TARGET)/lib/*.la
	rm -rf "$<-build"
	cd "$@.partial" && $(PREFIX_STRIP)
	touch "$@.partial"
	mv "$@.partial" "$@"

$(PREFIX)/gcc-install.tar.gz: $(PREFIX)/gcc-install
	tar \
		--create \
		--gzip \
		--file "$@" \
		--directory="$<" \
		.
endif
