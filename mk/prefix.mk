PREFIX=prefix/$(TARGET)

PREFIX_FREESTANDING_INSTALL=$(PREFIX)/gcc-freestanding-install
PREFIX_INSTALL=$(PREFIX)/relibc-install

ifeq ($(PREFIX_RUSTC),1)
	PREFIX_FREESTANDING_INSTALL=$(PREFIX)/rust-freestanding-install
	export RUSTUP_TOOLCHAIN=$(PREFIX)/rust-freestanding-install
endif

PREFIX_FREESTANDING_PATH=$(ROOT)/$(PREFIX_FREESTANDING_INSTALL)/bin
PREFIX_PATH=$(ROOT)/$(PREFIX_INSTALL)/bin

prefix-freestanding: $(PREFIX_FREESTANDING_INSTALL)

prefix: $(PREFIX_INSTALL)

$(PREFIX)/relibc-install: $(ROOT)/relibc | $(PREFIX)/gcc-install
	rm -rf "$@.partial" "$@"
	cp -r "$(PREFIX)/gcc-install" "$@.partial"
	cd "$<" && \
	export PATH="$(ROOT)/$@.partial/bin:$$PATH" && \
	export CARGO=xargo && \
	make -j `nproc` all && \
	make -j `nproc` install DESTDIR="$(ROOT)/$@.partial/$(TARGET)"
	touch "$@.partial"
	mv "$@.partial" "$@"

ifeq ($(PREFIX_BINARY),1)

$(PREFIX)/gcc-install.tar.gz:
	mkdir -p "$(@D)"
	wget -O $@.partial "https://static.redox-os.org/toolchain/$(TARGET)/gcc-install.tar.gz"
	mv $@.partial $@

$(PREFIX)/gcc-install: $(PREFIX)/gcc-install.tar.gz
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
	make -j `nproc` all && \
	make -j `nproc` install DESTDIR="$(ROOT)/$@.partial"
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
	make -j `nproc` all-gcc all-target-libgcc && \
	make -j `nproc` install-gcc install-target-libgcc DESTDIR="$(ROOT)/$@.partial"
	touch "$@.partial"
	mv "$@.partial" "$@"

$(PREFIX)/rust-freestanding-install: $(ROOT)/rust | $(PREFIX)/gcc-freestanding-install
	rm -rf "$(PREFIX)/rust-freestanding-build" "$@.partial" "$@"
	mkdir -p "$(PREFIX)/rust-freestanding-build"
	cp -r "$(PREFIX)/gcc-freestanding-install" "$@.partial"
	cd "$(PREFIX)/rust-freestanding-build" && \
	export PATH="$(ROOT)/$@.partial/bin:$$PATH" && \
	"$</configure" --prefix="" --disable-docs && \
	make -j `nproc` && \
	make -j `nproc` install DESTDIR="$(ROOT)/$@.partial"
	mkdir -p "$@.partial/lib/rustlib/x86_64-unknown-linux-gnu/bin"
	touch "$@.partial"
	mv "$@.partial" "$@"

# TODO: Only make headers for freestanding install
$(PREFIX)/relibc-freestanding-install: $(ROOT)/relibc | $(PREFIX_FREESTANDING_INSTALL)
	rm -rf "$@.partial" "$@"
	mkdir -p "$@.partial"
	cd "$<" && \
	export PATH="$(PREFIX_FREESTANDING_PATH):$$PATH" && \
	export CARGO=xargo && \
	make -j `nproc` all && \
	make -j `nproc` install DESTDIR="$(ROOT)/$@.partial/$(TARGET)"
	touch "$@.partial"
	mv "$@.partial" "$@"

$(PREFIX)/gcc-install: $(PREFIX)/gcc | $(PREFIX)/relibc-freestanding-install
	rm -rf "$<-build" "$@.partial" "$@"
	mkdir -p "$<-build"
	cp -r "$(PREFIX)/binutils-install" "$@.partial"
	cd "$<-build" && \
	export PATH="$(ROOT)/$@.partial/bin:$$PATH" && \
	"$(ROOT)/$</configure" \
		--target="$(TARGET)" \
		--program-prefix="$(TARGET)-" \
		--prefix="" \
		--with-sysroot \
		--with-build-sysroot="$(ROOT)/$(PREFIX)/relibc-freestanding-install/$(TARGET)" \
		--with-native-system-header-dir="/include" \
		--disable-nls \
		--disable-werror \
		--enable-languages=c,c++ \
		--enable-threads=posix \
		&& \
	make -j `nproc` all-gcc all-target-libgcc all-target-libstdc++-v3 && \
	make -j `nproc` install-gcc install-target-libgcc install-target-libstdc++-v3 DESTDIR="$(ROOT)/$@.partial"
	touch "$@.partial"
	mv "$@.partial" "$@"

$(PREFIX)/gcc-install.tar.gz: $(PREFIX)/gcc-install
	tar \
		--create \
		--gzip \
		--file "$@" \
		--directory="$<" \
		.

# Building full rustc may not be required
# $(PREFIX)/rust-install: $(ROOT)/rust | $(PREFIX)/gcc-install
# 	rm -rf "$(PREFIX)/rust-build" "$@"
# 	mkdir -p "$(PREFIX)/rust-build" "$@"
# 	cd "$(PREFIX)/rust-build" && \
# 	export PATH="$(PREFIX_PATH):$$PATH" && \
# 	"$</configure" --target="$(TARGET)" --prefix="$@" --disable-docs && \
# 	make -j `nproc` && \
# 	make install -j `nproc`
# 	touch "$@"

endif
