PREFIX=$(ROOT)/prefix/$(TARGET)

PREFIX_FREESTANDING_INSTALL=$(PREFIX)/gcc-freestanding-install
PREFIX_INSTALL=$(PREFIX)/gcc-install

ifeq ($(PREFIX_RUSTC),1)
	PREFIX_FREESTANDING_INSTALL=$(PREFIX)/rust-freestanding-install
	export RUSTUP_TOOLCHAIN=$(PREFIX)/rust-freestanding-install
endif

PREFIX_FREESTANDING_PATH=$(PREFIX_FREESTANDING_INSTALL)/bin
PREFIX_PATH=$(PREFIX_INSTALL)/bin

prefix-freestanding: $(PREFIX_FREESTANDING_INSTALL)

prefix: $(PREFIX_INSTALL)

$(PREFIX)/binutils.tar.bz2:
	mkdir -p "$(@D)"
	wget -O $@.partial "https://gitlab.redox-os.org/redox-os/binutils-gdb/-/archive/redox/binutils-gdb-redox.tar.bz2"
	mv $@.partial $@

$(PREFIX)/binutils: $(PREFIX)/binutils.tar.bz2
	mkdir -p "$@.partial"
	tar --extract --file "$<" --directory "$@.partial" --strip-components=1
	mv "$@.partial" "$@"
	touch "$@"

$(PREFIX)/binutils-install: $(PREFIX)/binutils
	rm -rf "$<-build" "$@"
	mkdir -p "$<-build" "$@"
	cd "$<-build" && \
	"$</configure" \
		--target="$(TARGET)" \
		--program-prefix="$(TARGET)-" \
		--prefix="$@" \
		--disable-werror \
		&& \
	make all -j `nproc` && \
	make install -j `nproc`
	touch "$@"

$(PREFIX)/gcc.tar.bz2:
	mkdir -p "$(@D)"
	wget -O $@.partial "https://gitlab.redox-os.org/redox-os/gcc/-/archive/redox/gcc-redox.tar.bz2"
	mv "$@.partial" "$@"

$(PREFIX)/gcc: $(PREFIX)/gcc.tar.bz2
	mkdir -p "$@.partial"
	tar --extract --file "$<" --directory "$@.partial" --strip-components=1
	cd "$@.partial" && ./contrib/download_prerequisites
	mv "$@.partial" "$@"
	touch "$@"

$(PREFIX)/gcc-freestanding-install: $(PREFIX)/gcc | $(PREFIX)/binutils-install
	rm -rf "$<-freestanding-build" "$@"
	mkdir -p "$<-freestanding-build"
	cp -r "$(PREFIX)/binutils-install" "$@"
	cd "$<-freestanding-build" && \
	export PATH="$@/bin:$$PATH" && \
	"$</configure" \
		--target="$(TARGET)" \
		--program-prefix="$(TARGET)-" \
		--prefix="$@" \
		--disable-nls \
		--enable-languages=c,c++ \
		--without-headers \
		&& \
	make all-gcc all-target-libgcc -j `nproc` && \
	make install-gcc install-target-libgcc -j `nproc`
	touch "$@"

$(PREFIX)/rust-freestanding-install: $(ROOT)/rust | $(PREFIX)/gcc-freestanding-install
	rm -rf "$(PREFIX)/rust-freestanding-build" "$@"
	mkdir -p "$(PREFIX)/rust-freestanding-build"
	cp -r "$(PREFIX)/gcc-freestanding-install" "$@"
	cd "$(PREFIX)/rust-freestanding-build" && \
	export PATH="$@/bin:$$PATH" && \
	"$</configure" --prefix="$@" --disable-docs && \
	make -j `nproc` && \
	make install -j `nproc`
	mkdir -p "$@/lib/rustlib/x86_64-unknown-linux-gnu/bin"
	touch "$@"

$(PREFIX)/relibc-install: $(ROOT)/relibc | $(PREFIX_FREESTANDING_INSTALL)
	rm -rf "$@"
	mkdir -p "$@"
	cd "$<" && \
	export PATH="$(PREFIX_FREESTANDING_PATH):$$PATH" && \
	make CARGO=xargo all && \
	make CARGO=xargo DESTDIR="$@/usr" install
	touch "$@"

$(PREFIX)/gcc-install: $(PREFIX)/gcc | $(PREFIX)/relibc-install
	rm -rf "$<-build" "$@"
	mkdir -p "$<-build"
	cp -r "$(PREFIX)/binutils-install" "$@"
	cd "$<-build" && \
	export PATH="$@/bin:$$PATH" && \
	"$</configure" \
		--target="$(TARGET)" \
		--program-prefix="$(TARGET)-" \
		--prefix="$@" \
		--with-sysroot="$(PREFIX)/relibc-install" \
		--disable-nls \
		--disable-werror \
		--enable-languages=c,c++ \
		--enable-threads=posix \
		&& \
	make all-gcc all-target-libgcc all-target-libstdc++-v3 -j `nproc` && \
	make install-gcc install-target-libgcc install-target-libstdc++-v3 -j `nproc`
	touch "$@"

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
