#Modify for different target support
ARCH?=i386
#ARCH?=x86_64

BUILD=build/$(ARCH)-unknown-redox/debug

QEMU?=qemu-system-$(ARCH)

CARGO=CARGO_TARGET_DIR=build RUSTC="./rustc-$(ARCH).sh" cargo rustc
CARGOFLAGS=--verbose --target=$(ARCH)-unknown-redox.json -- -L $(BUILD) \
	-C no-prepopulate-passes -C no-stack-check -C opt-level=2 \
	-Z no-landing-pads \
	-A dead_code
RUSTC=RUST_BACKTRACE=1 rustc
RUSTDOC=rustdoc --target=$(ARCH)-unknown-redox.json -L $(BUILD) \
	--no-defaults --passes collapse-docs --passes unindent-comments
RUSTCFLAGS=--target=$(ARCH)-unknown-redox.json -L $(BUILD) \
	-C no-prepopulate-passes -C no-stack-check -C opt-level=2 \
	-Z no-landing-pads \
	-A dead_code
AS=nasm
AWK=awk
BASENAME=basename
CUT=cut
DATE=date
FIND=find
FUMOUNT=fusermount -u
LD=ld
LDARGS=-m elf_$(ARCH)
MAKE=make
MKDIR=mkdir
OBJDUMP=objdump
RM=rm
SED=sed
SORT=sort
VB_AUDIO="pulse"
VBM=VBoxManage
VBM_CLEANUP=\
	if [ $$? -ne 0 ]; \
	then \
		if [ -d "$$HOME/VirtualBox VMs/Redox" ]; \
		then \
			echo "Redox directory exists, deleting..."; \
			$(RM) -rf "$$HOME/VirtualBox VMs/Redox"; \
		fi \
	fi

UNAME := $(shell uname)
ifeq ($(UNAME),Darwin)
	FUMOUNT=umount
	LD=$(ARCH)-elf-ld
	OBJDUMP=$(ARCH)-elf-objdump
	CARGOFLAGS += -C ar=$(ARCH)-elf-ar -C linker=$(ARCH)-elf-gcc
	RUSTCFLAGS += -C ar=$(ARCH)-elf-ar -C linker=$(ARCH)-elf-gcc
	VB="/Applications/VirtualBox.app/Contents/MacOS/VirtualBox"
	VB_AUDIO="coreaudio"
	VBM="/Applications/VirtualBox.app/Contents/MacOS/VBoxManage"
endif

.PHONY: help all doc apps bins clean FORCE \
	drivers binutils coreutils extrautils games \
	qemu qemu_bare qemu_tap bochs \
	virtualbox virtualbox_tap \
	arping ping wireshark

help:
	@echo ".########..########.########...#######..##.....##"
	@echo ".##.....##.##.......##.....##.##.....##..##...##."
	@echo ".##.....##.##.......##.....##.##.....##...##.##.."
	@echo ".########..######...##.....##.##.....##....###..."
	@echo ".##...##...##.......##.....##.##.....##...##.##.."
	@echo ".##....##..##.......##.....##.##.....##..##...##."
	@echo ".##.....##.########.########...#######..##.....##"
	@echo
	@echo "-------- Redox: A Rust Operating System ---------"
	@echo
	@echo "Commands:"
	@echo
	@echo "    make all"
	@echo "        Build raw image of filesystem used by Redox."
	@echo "        It create $(BUILD)/harddrive.bin which can be used to build"
	@echo "        images for Your virtual machine."
	@echo
	@echo "    make virtualbox"
	@echo "        Build Redox and run it inside VirtualBox machine."
	@echo
	@echo "    make qemu"
	@echo "        Build Redox and run it inside KVM machine."
	@echo
	@echo "    make qemu kvm=no"
	@echo "        Build Redox and run it inside Qemu machine without KVM support."
	@echo
	@echo "    make apps"
	@echo "        Build apps for Redox."
	@echo
	@echo "    make clean"
	@echo "        Clean build directory."
	@echo
	@echo
	@echo " .........................................................."
	@echo " For more information check out 'github.com/redox-os/redox'"
	@echo " or 'redox-os.org'"

all: $(BUILD)/harddrive.bin

filesystem/apps/pixelcannon/main.bin: crates/pixelcannon/src/main.rs crates/pixelcannon/src/*.rs crates/pixelcannon/assets/* $(BUILD)/libstd.rlib $(BUILD)/liborbclient.rlib $(BUILD)/liborbimage.rlib
	mkdir -p filesystem/apps/pixelcannon/assets/
	cp crates/pixelcannon/assets/* filesystem/apps/pixelcannon/assets/
	$(RUSTC) $(RUSTCFLAGS) -C lto --crate-type bin -o $@ $<

filesystem/apps/sodium/main.bin: filesystem/apps/sodium/src/main.rs filesystem/apps/sodium/src/*.rs $(BUILD)/libstd.rlib $(BUILD)/liborbclient.rlib
	$(RUSTC) $(RUSTCFLAGS) -C lto --crate-type bin -o $@ $< --cfg 'feature="orbital"'

filesystem/apps/%/main.bin: filesystem/apps/%/main.rs filesystem/apps/%/*.rs $(BUILD)/libstd.rlib $(BUILD)/liborbclient.rlib $(BUILD)/liborbfont.rlib $(BUILD)/liborbimage.rlib $(BUILD)/liborbtk.rlib
	$(RUSTC) $(RUSTCFLAGS) -C lto --crate-type bin -o $@ $< -L $(BUILD)/deps

filesystem/apps/%/main.bin: crates/orbutils/src/%/main.rs crates/orbutils/src/%/*.rs $(BUILD)/libstd.rlib $(BUILD)/liborbclient.rlib $(BUILD)/liborbfont.rlib $(BUILD)/liborbimage.rlib $(BUILD)/liborbtk.rlib
	$(RUSTC) $(RUSTCFLAGS) -C lto --crate-type bin -o $@ $< -L $(BUILD)/deps

apps: filesystem/apps/calculator/main.bin \
	  filesystem/apps/character_map/main.bin \
	  filesystem/apps/editor/main.bin \
	  filesystem/apps/file_manager/main.bin \
	  filesystem/apps/orbtk/main.bin \
	  filesystem/apps/pixelcannon/main.bin \
	  filesystem/apps/player/main.bin \
	  filesystem/apps/sodium/main.bin \
	  filesystem/apps/terminal/main.bin \
	  filesystem/apps/viewer/main.bin

$(BUILD)/libbitflags.rlib: crates/bitflags/src/lib.rs crates/bitflags/src/*.rs $(BUILD)/libcore.rlib
	$(RUSTC) $(RUSTCFLAGS) --crate-name bitflags --crate-type lib -o $@ $<

$(BUILD)/libextra.rlib: crates/extra/src/lib.rs crates/extra/src/*.rs $(BUILD)/libstd.rlib
	$(RUSTC) $(RUSTCFLAGS) --crate-name extra --crate-type lib -o $@ $<

$(BUILD)/libpng.rlib: crates/rust-png/src/lib.rs crates/rust-png/src/*.rs $(BUILD)/libpng_sys.rlib
	$(RUSTC) $(RUSTCFLAGS) --crate-name png --crate-type lib -o $@ $< -L native=libc/lib/

$(BUILD)/libpng_sys.rlib: crates/rust-png/png-sys/lib.rs crates/rust-png/png-sys/*.rs $(BUILD)/liblibz_sys.rlib
	$(RUSTC) $(RUSTCFLAGS) --crate-name png_sys --crate-type lib -o $@ $<

$(BUILD)/liblibz_sys.rlib: crates/libz-sys/src/lib.rs crates/libz-sys/src/*.rs $(BUILD)/libstd.rlib $(BUILD)/liblibc.rlib
	$(RUSTC) $(RUSTCFLAGS) --crate-name libz_sys --crate-type lib -o $@ $< -L native=libc/lib/

$(BUILD)/libwalkdir.rlib: crates/walkdir/src/lib.rs crates/walkdir/src/*.rs $(BUILD)/libstd.rlib
	$(RUSTC) $(RUSTCFLAGS) --crate-name walkdir --crate-type lib -o $@ $<

$(BUILD)/libralloc.rlib: crates/ralloc/src/lib.rs crates/ralloc/src/*.rs $(BUILD)/libsystem.rlib
	$(RUSTC) $(RUSTCFLAGS) --crate-name ralloc --crate-type lib -o $@ $< --cfg 'feature="allocator"'

filesystem/bin/%: crates/coreutils/src/bin/%.rs $(BUILD)/libextra.rlib $(BUILD)/libwalkdir.rlib
	mkdir -p filesystem/bin
	$(RUSTC) $(RUSTCFLAGS) -C lto --crate-type bin -o $@ $<

coreutils: \
	filesystem/bin/basename \
	filesystem/bin/cat \
	filesystem/bin/clear \
	filesystem/bin/cp \
	filesystem/bin/cut \
	filesystem/bin/date \
	filesystem/bin/dmesg \
	filesystem/bin/du \
	filesystem/bin/echo \
	filesystem/bin/env \
	filesystem/bin/false \
	filesystem/bin/free \
	filesystem/bin/head \
	filesystem/bin/ls \
	filesystem/bin/mkdir \
	filesystem/bin/mv \
	filesystem/bin/ps \
	filesystem/bin/pwd \
	filesystem/bin/realpath \
	filesystem/bin/reset \
	filesystem/bin/rm \
	filesystem/bin/rmdir \
	filesystem/bin/seq \
	filesystem/bin/shutdown \
	filesystem/bin/sleep \
	filesystem/bin/tail \
	filesystem/bin/time \
	filesystem/bin/touch \
	filesystem/bin/true \
	filesystem/bin/wc \
	filesystem/bin/yes
	#TODO: filesystem/bin/test

$(BUILD)/libbinutils.rlib: crates/binutils/src/lib.rs crates/binutils/src/*.rs $(BUILD)/libextra.rlib
	$(RUSTC) $(RUSTCFLAGS) --crate-name binutils --crate-type lib -o $@ $<

filesystem/bin/%: crates/binutils/src/bin/%.rs $(BUILD)/libbinutils.rlib
	mkdir -p filesystem/bin
	$(RUSTC) $(RUSTCFLAGS) -C lto --crate-type bin -o $@ $<

binutils: \
	filesystem/bin/hex \
	filesystem/bin/hexdump \
	filesystem/bin/strings

filesystem/bin/%: drivers/%/main.rs $(BUILD)/libstd.rlib $(BUILD)/libio.rlib
	mkdir -p filesystem/bin
	$(RUSTC) $(RUSTCFLAGS) -C lto --crate-type bin -o $@ $<

drivers: \
	filesystem/bin/seriald

$(BUILD)/libtermion.rlib: crates/termion/src/lib.rs crates/termion/src/*.rs $(BUILD)/libstd.rlib
	$(RUSTC) $(RUSTCFLAGS) --crate-name termion --crate-type lib -o $@ $< --cfg 'feature="nightly"'

filesystem/bin/%: crates/extrautils/src/bin/%.rs $(BUILD)/libextra.rlib $(BUILD)/libtermion.rlib
	mkdir -p filesystem/bin
	$(RUSTC) $(RUSTCFLAGS) -C lto --crate-type bin -o $@ $<

extrautils: \
	filesystem/bin/calc \
	filesystem/bin/cksum \
	filesystem/bin/cur \
	filesystem/bin/grep \
	filesystem/bin/less \
	filesystem/bin/man \
	filesystem/bin/mtxt \
	filesystem/bin/rem \
	filesystem/bin/wget

filesystem/bin/%: crates/games/src/%/main.rs crates/games/src/%/*.rs $(BUILD)/libextra.rlib $(BUILD)/libtermion.rlib
	mkdir -p filesystem/bin
	$(RUSTC) $(RUSTCFLAGS) -C lto --crate-type bin -o $@ $<

games: \
	filesystem/bin/ice \
	filesystem/bin/flappy \
	filesystem/bin/h4xx3r \
	filesystem/bin/minesweeper \
	filesystem/bin/rusthello \
	filesystem/bin/snake

filesystem/bin/%: crates/%/main.rs crates/%/*.rs $(BUILD)/libstd.rlib
	mkdir -p filesystem/bin
	$(RUSTC) $(RUSTCFLAGS) -C lto --crate-type bin -o $@ $<

filesystem/bin/%: libc/bin/%
	mkdir -p filesystem/bin
	cp $< $@

$(BUILD)/librusttype.rlib: crates/rusttype/src/lib.rs crates/rusttype/src/*.rs crates/rusttype/src/*/*.rs $(BUILD)/libstd.rlib
	$(CARGO) --manifest-path crates/rusttype/Cargo.toml --lib $(CARGOFLAGS)

$(BUILD)/ion-shell.bin: FORCE $(BUILD)/libstd.rlib
	$(CARGO) --manifest-path crates/ion/Cargo.toml --bin ion-shell $(CARGOFLAGS) -C lto

filesystem/bin/ion: $(BUILD)/ion-shell.bin
	mkdir -p filesystem/bin
	cp $< $@

filesystem/bin/sh: $(BUILD)/ion-shell.bin
	mkdir -p filesystem/bin
	cp $< $@

filesystem/bin/launcher: crates/orbutils/src/launcher/main.rs crates/orbutils/src/launcher/*.rs $(BUILD)/libstd.rlib $(BUILD)/liborbclient.rlib $(BUILD)/liborbtk.rlib
	mkdir -p filesystem/bin
	$(RUSTC) $(RUSTCFLAGS) -C lto --crate-type bin -o $@ $< -L $(BUILD)/deps

filesystem/bin/orbital: crates/orbital/main.rs crates/orbital/*.rs $(BUILD)/libstd.rlib $(BUILD)/liborbimage.rlib
	mkdir -p filesystem/bin
	$(RUSTC) $(RUSTCFLAGS) -C lto --crate-type bin -o $@ $<

filesystem/bin/zfs: crates/zfs/src/main.rs crates/zfs/src/*.rs $(BUILD)/libstd.rlib
	mkdir -p filesystem/bin
	$(RUSTC) $(RUSTCFLAGS) -C lto --crate-type bin -o $@ $<

filesystem/bin/%: crates/%/main.rs crates/%/*.rs $(BUILD)/libstd.rlib
	mkdir -p filesystem/bin
	$(RUSTC) $(RUSTCFLAGS) -C lto --crate-type bin -o $@ $<

bins: \
	coreutils \
	extrautils \
	drivers \
	games \
	filesystem/bin/ansi-test \
	filesystem/bin/c-test \
	filesystem/bin/dosbox \
	filesystem/bin/ed \
	filesystem/bin/example \
	filesystem/bin/init \
  	filesystem/bin/ion \
	filesystem/bin/launcher \
  	filesystem/bin/lua \
  	filesystem/bin/luac \
  	filesystem/bin/login \
  	filesystem/bin/minesweeper \
  	filesystem/bin/orbital \
	filesystem/bin/screenfetch \
  	filesystem/bin/sdl-test \
	filesystem/bin/std-test \
  	filesystem/bin/sh \
	filesystem/bin/tar \
	#TODO: binutils	filesystem/bin/zfs

refs: FORCE
	mkdir -p filesystem/ref/
	cargo run --manifest-path crates/docgen/Cargo.toml -- crates/coreutils/src/bin/ filesystem/ref/
	cargo run --manifest-path crates/docgen/Cargo.toml -- crates/extrautils/src/bin/ filesystem/ref/
	cargo run --manifest-path crates/docgen/Cargo.toml -- kernel/ filesystem/ref/

initfs/bin/init: crates/init/main.rs crates/init/*.rs $(BUILD)/libstd.rlib
	mkdir -p initfs/bin/
	$(RUSTC) $(RUSTCFLAGS) -C lto --crate-type bin -o $@ $<

initfs/bin/redoxfsd: crates/redoxfs/scheme/main.rs crates/redoxfs/scheme/*.rs crates/redoxfs/scheme/*/*.rs $(BUILD)/libredoxfs.rlib
	mkdir -p initfs/bin/
	$(RUSTC) $(RUSTCFLAGS) -C lto --crate-type bin -o $@ $<

initfs/build/arch:
	mkdir -p initfs/build/
	echo $(ARCH) > $@

initfs/build/branch:
	mkdir -p initfs/build/
	git rev-parse --abbrev-ref HEAD > $@

initfs/build/cargo:
	mkdir -p initfs/build/
	cargo -V > $@

initfs/build/date:
	mkdir -p initfs/build/
	date > $@

initfs/build/host:
	mkdir -p initfs/build/
	uname -a > $@

initfs/build/rustc:
	mkdir -p initfs/build/
	$(RUSTC) -V > $@

initfs/build/rev:
	mkdir -p initfs/build/
	git rev-parse HEAD > $@

build/initfs.gen: \
		initfs/bin/init \
		initfs/bin/redoxfsd \
		initfs/build/arch \
		initfs/build/branch \
		initfs/build/cargo \
		initfs/build/date \
		initfs/build/host \
		initfs/build/rustc \
		initfs/build/rev \
		initfs/etc/init.rc
	echo 'use collections::BTreeMap;' > $@
	echo 'pub fn gen() -> BTreeMap<&'"'"'static str, &'"'"'static [u8]> {' >> $@
	echo '    let mut files: BTreeMap<&'"'"'static str, &'"'"'static [u8]> = BTreeMap::new();' >> $@
	$(FIND) initfs -type f -o -type l | $(CUT) -d '/' -f2- | $(SORT) \
		| $(AWK) '{printf("    files.insert(\"%s\", include_bytes!(\"../initfs/%s\"));\n", $$0, $$0)}' \
		>> $@
	echo '    files' >> $@
	echo '}' >> $@

test: kernel/main.rs \
	  rust/src/libtest/lib.rs \
	  $(BUILD)/libcore.rlib \
	  $(BUILD)/liballoc.rlib \
	  $(BUILD)/libcollections.rlib \
	  $(BUILD)/libtest.rlib
	$(RUSTC) $(RUSTCFLAGS) --test $<

clean:
	$(RM) -rf build doc filesystem/bin/ filesystem/ref/ initfs/bin/ initfs/build/ filesystem/apps/*/*.bin filesystem/apps/*/*.list

FORCE:

doc/core: rust/src/libcore/lib.rs $(BUILD)/libcore.rlib
	$(RUSTDOC) $<

doc/alloc_malloc: liballoc_malloc/lib.rs $(BUILD)/liballoc_malloc.rlib doc/core
	$(RUSTDOC) $<

doc/alloc_system: liballoc_system/lib.rs $(BUILD)/liballoc_system.rlib doc/core
	$(RUSTDOC) $<

doc/alloc: rust/src/liballoc/lib.rs $(BUILD)/liballoc.rlib doc/alloc_system
	$(RUSTDOC) $<

doc/rustc_unicode: rust/src/librustc_unicode/lib.rs $(BUILD)/librustc_unicode.rlib doc/core
	$(RUSTDOC) $<

doc/collections: rust/src/libcollections/lib.rs $(BUILD)/libcollections.rlib doc/alloc doc/rustc_unicode
	$(RUSTDOC) $<

doc/rand: rust/src/librand/lib.rs $(BUILD)/librand.rlib doc/collections
	$(RUSTDOC) $<

doc/io: crates/io/lib.rs crates/io/*.rs $(BUILD)/libio.rlib doc/core
	$(RUSTDOC) $<

doc/system: crates/system/lib.rs crates/system/*.rs crates/system/*/*.rs $(BUILD)/libsystem.rlib doc/core
	$(RUSTDOC) $<

doc/redoxfs: crates/redoxfs/src/lib.rs crates/redoxfs/src/*.rs doc/system doc/alloc doc/collections
	$(RUSTDOC) $<

doc/kernel: kernel/main.rs kernel/*.rs kernel/*/*.rs kernel/*/*/*.rs $(BUILD)/kernel.rlib doc/io doc/redoxfs
	$(RUSTDOC) $<

doc/extra: crates/extra/src/lib.rs crates/extra/src/*.rs $(BUILD)/libextra.rlib
	$(RUSTDOC) --crate-name=extra $<

doc/ralloc: crates/ralloc/src/lib.rs crates/ralloc/src/*.rs $(BUILD)/libralloc.rlib
	$(RUSTDOC) --crate-name=ralloc $<

doc/binutils: crates/binutils/src/lib.rs crates/binutils/src/*.rs $(BUILD)/libbinutils.rlib
	$(RUSTDOC) --crate-name=binutils $<

doc/zfs: crates/zfs/src/main.rs crates/zfs/src/*.rs filesystem/bin/zfs
	$(RUSTDOC) --crate-name=zfs $<

doc/orbclient: crates/orbclient/src/lib.rs crates/orbclient/src/*.rs $(BUILD)/liborbclient.rlib doc/std
	$(RUSTDOC) $<

doc/orbtk: crates/orbtk/src/lib.rs crates/orbtk/src/*.rs $(BUILD)/liborbtk.rlib doc/orbclient
	$(RUSTDOC) $<

doc/sodium: filesystem/apps/sodium/src/main.rs filesystem/apps/sodium/src/*.rs filesystem/apps/sodium/main.bin
	$(RUSTDOC) --crate-name=sodium --cfg 'feature="orbital"' $<

doc/std: libstd/src/lib.rs libstd/src/*.rs libstd/src/*/*.rs libstd/src/*/*/*.rs $(BUILD)/libstd.rlib doc/rand doc/system doc/alloc_malloc
	$(RUSTDOC) --crate-name=std $<

doc: doc/kernel doc/std doc/extra doc/ralloc doc/orbclient doc/orbtk doc/sodium doc/binutils

$(BUILD)/libcore.rlib: rust/src/libcore/lib.rs
	$(MKDIR) -p $(BUILD)
	$(RUSTC) $(RUSTCFLAGS) -o $@ $<

$(BUILD)/liballoc_malloc.rlib: liballoc_malloc/lib.rs $(BUILD)/libcore.rlib
	$(RUSTC) $(RUSTCFLAGS) -o $@ $< -L native=libc/lib/

$(BUILD)/liballoc_system.rlib: liballoc_system/lib.rs $(BUILD)/libcore.rlib
	$(RUSTC) $(RUSTCFLAGS) -o $@ $<

$(BUILD)/liballoc.rlib: rust/src/liballoc/lib.rs $(BUILD)/libcore.rlib $(BUILD)/liballoc_system.rlib
	$(RUSTC) $(RUSTCFLAGS) -o $@ $<

$(BUILD)/librustc_unicode.rlib: rust/src/librustc_unicode/lib.rs $(BUILD)/libcore.rlib
	$(RUSTC) $(RUSTCFLAGS) -o $@ $<

$(BUILD)/libcollections.rlib: rust/src/libcollections/lib.rs $(BUILD)/libcore.rlib $(BUILD)/liballoc.rlib $(BUILD)/librustc_unicode.rlib
	$(RUSTC) $(RUSTCFLAGS) -o $@ $<

$(BUILD)/libgetopts.rlib: rust/src/libgetopts/lib.rs $(BUILD)/libserialize.rlib $(BUILD)/liblog.rlib
	$(RUSTC) $(RUSTCFLAGS) -o $@ $<

$(BUILD)/librand.rlib: rust/src/librand/lib.rs $(BUILD)/libcore.rlib $(BUILD)/liballoc.rlib $(BUILD)/librustc_unicode.rlib $(BUILD)/libcollections.rlib
	$(RUSTC) $(RUSTCFLAGS) -o $@ $<

$(BUILD)/liblibc.rlib: crates/liblibc/src/lib.rs $(BUILD)/libcore.rlib
	$(RUSTC) $(RUSTCFLAGS) -o $@ $< -L native=libc/lib/

$(BUILD)/librealstd.rlib: rust/src/libstd/lib.rs $(BUILD)/libcore.rlib $(BUILD)/liblibc.rlib $(BUILD)/liballoc.rlib $(BUILD)/librustc_unicode.rlib $(BUILD)/libcollections.rlib $(BUILD)/librand.rlib
	$(RUSTC) $(RUSTCFLAGS) --cfg unix --crate-type rlib -o $@ $<

$(BUILD)/libstd.rlib: libstd/src/lib.rs libstd/src/*.rs libstd/src/*/*.rs libstd/src/*/*/*.rs $(BUILD)/libcore.rlib $(BUILD)/liballoc_malloc.rlib $(BUILD)/liballoc.rlib $(BUILD)/libcollections.rlib $(BUILD)/librand.rlib $(BUILD)/libsystem.rlib
	$(RUSTC) $(RUSTCFLAGS) -o $@ $< -L native=libc/lib/

$(BUILD)/liborbclient.rlib: crates/orbclient/src/lib.rs crates/orbclient/src/*.rs crates/orbclient/src/*/*.rs $(BUILD)/libstd.rlib
	$(RUSTC) $(RUSTCFLAGS) -o $@ $<

$(BUILD)/liborbfont.rlib: crates/orbfont/src/lib.rs crates/orbfont/src/*.rs $(BUILD)/libstd.rlib $(BUILD)/liborbclient.rlib $(BUILD)/librusttype.rlib
	$(RUSTC) $(RUSTCFLAGS) -o $@ $< -L $(BUILD)/deps

$(BUILD)/liborbimage.rlib: crates/orbimage/src/lib.rs crates/orbimage/src/*.rs $(BUILD)/libstd.rlib $(BUILD)/liborbclient.rlib $(BUILD)/libpng.rlib
	$(RUSTC) $(RUSTCFLAGS) -o $@ $<

$(BUILD)/liborbtk.rlib: crates/orbtk/src/lib.rs crates/orbtk/src/*.rs $(BUILD)/libstd.rlib $(BUILD)/liborbclient.rlib $(BUILD)/liborbfont.rlib
	$(RUSTC) $(RUSTCFLAGS) -o $@ $< -L $(BUILD)/deps

#Kernel stuff
$(BUILD)/libio.rlib: crates/io/lib.rs crates/io/*.rs $(BUILD)/libcore.rlib
	$(RUSTC) $(RUSTCFLAGS) -o $@ $<

$(BUILD)/libsystem.rlib: crates/system/lib.rs crates/system/*.rs crates/system/*/*.rs $(BUILD)/libcore.rlib
	$(RUSTC) $(RUSTCFLAGS) -o $@ $<

$(BUILD)/libredoxfs.rlib: crates/redoxfs/src/lib.rs crates/redoxfs/src/*.rs $(BUILD)/libstd.rlib
	$(RUSTC) $(RUSTCFLAGS) -o $@ $<

$(BUILD)/kernel.rlib: kernel/main.rs kernel/*.rs kernel/*/*.rs kernel/*/*/*.rs $(BUILD)/libbitflags.rlib $(BUILD)/libio.rlib build/initfs.gen
	$(RUSTC) $(RUSTCFLAGS) -C lto -o $@ $<

$(BUILD)/kernel.bin: $(BUILD)/kernel.rlib kernel/kernel.ld
	$(LD) $(LDARGS) -o $@ -T kernel/kernel.ld -z max-page-size=0x1000 $<

$(BUILD)/kernel.list: $(BUILD)/kernel.bin
	$(OBJDUMP) -C -M intel -D $< > $@

$(BUILD)/kernel.asm: kernel/main.rs $(BUILD)/libcore.rlib $(BUILD)/liballoc.rlib $(BUILD)/libcollections.rlib
	$(RUSTC) $(RUSTCFLAGS) -C lto -o $@ --emit asm $<

$(BUILD)/kernel.ir: kernel/main.rs $(BUILD)/libcore.rlib $(BUILD)/liballoc.rlib $(BUILD)/libcollections.rlib
	$(RUSTC) $(RUSTCFLAGS) -C lto -o $@ --emit llvm-ir $<

#Rustc
$(BUILD)/liblog.rlib: rust/src/liblog/lib.rs $(BUILD)/libstd.rlib
	$(RUSTC) $(RUSTCFLAGS) -o $@ $<

$(BUILD)/librustc_%.rlib: rust/src/librustc_%/lib.rs $(BUILD)/libsyntax.rlib
	$(RUSTC) $(RUSTCFLAGS) -o $@ $<

$(BUILD)/libserialize.rlib: rust/src/libserialize/lib.rs $(BUILD)/liblog.rlib
	$(RUSTC) $(RUSTCFLAGS) -o $@ $<

$(BUILD)/libsyntax.rlib: rust/src/libsyntax/lib.rs $(BUILD)/libserialize.rlib
	$(RUSTC) $(RUSTCFLAGS) -o $@ $<

$(BUILD)/libtest.rlib: rust/src/libtest/lib.rs $(BUILD)/libstd.rlib $(BUILD)/libgetopts.rlib
	$(RUSTC) $(RUSTCFLAGS) -o $@ $<

rustc: $(BUILD)/librustc_back.rlib \
	$(BUILD)/librustc_bitflags.rlib \
	$(BUILD)/librustc_borrowck.rlib \
	$(BUILD)/librustc_data_structures.rlib \
	$(BUILD)/librustc_driver.rlib \
	$(BUILD)/librustc_front.rlib \
	$(BUILD)/librustc_lint.rlib \
	$(BUILD)/librustc_llvm.rlib \
	$(BUILD)/librustc_metadata.rlib \
	$(BUILD)/librustc_mir.rlib \
	$(BUILD)/librustc_passes.rlib \
	$(BUILD)/librustc_platform_intrinsics.rlib \
	$(BUILD)/librustc_plugin.rlib \
	$(BUILD)/librustc_privacy.rlib \
	$(BUILD)/librustc_resolve.rlib \
	$(BUILD)/librustc_trans.rlib \
	$(BUILD)/librustc_typeck.rlib \
	$(BUILD)/librustc_unicode.rlib

filesystem/%.list: filesystem/%
	$(OBJDUMP) -C -M intel -D $< > $@

filesystem/apps/zfs/zfs.img:
	dd if=/dev/zero of=$@ bs=64M count=1
	sudo losetup /dev/loop0 $@
	-sudo zpool create redox_zfs /dev/loop0
	-sudo mkdir /redox_zfs/home/
	-sudo mkdir /redox_zfs/home/test/
	-sudo cp LICENSE.md README.md /redox_zfs/home/
	-sudo sync
	-sleep 1
	-sudo zfs unmount redox_zfs
	-sleep 1
	-sudo zpool destroy redox_zfs
	sudo losetup -d /dev/loop0

$(BUILD)/filesystem.bin: apps bins
	rm -rf $@ $(BUILD)/filesystem/
	echo exit | cargo run --manifest-path crates/redoxfs/Cargo.toml --bin redoxfs-utility $@
	mkdir -p $(BUILD)/filesystem/
	cargo run --manifest-path crates/redoxfs/Cargo.toml --bin redoxfs-fuse $@ $(BUILD)/filesystem/ &
	sleep 2
	-cp -RL filesystem/* $(BUILD)/filesystem/
	sync
	-$(FUMOUNT) $(BUILD)/filesystem/
	rm -rf $(BUILD)/filesystem/

$(BUILD)/harddrive.bin: kernel/harddrive.asm $(BUILD)/kernel.bin $(BUILD)/filesystem.bin
	$(AS) -f bin -o $@ -l $(BUILD)/harddrive.list -D ARCH_$(ARCH) -D TIME="`$(DATE) "+%F %T"`" -i$(BUILD)/ -ikernel/ -ifilesystem/ $<

virtualbox: $(BUILD)/harddrive.bin
	echo "Delete VM"
	-$(VBM) unregistervm Redox --delete; $(VBM_CLEANUP)
	echo "Delete Disk"
	-$(RM) harddrive.vdi
	echo "Create VM"
	$(VBM) createvm --name Redox --register
	echo "Set Configuration"
	$(VBM) modifyvm Redox --memory 1024
	$(VBM) modifyvm Redox --vram 16
	# $(VBM) modifyvm Redox --nic1 nat
	# $(VBM) modifyvm Redox --nictype1 82540EM
	# $(VBM) modifyvm Redox --nictrace1 on
	# $(VBM) modifyvm Redox --nictracefile1 $(BUILD)/network.pcap
	$(VBM) modifyvm Redox --uart1 0x3F8 4
	$(VBM) modifyvm Redox --uartmode1 file $(BUILD)/serial.log
	$(VBM) modifyvm Redox --usb off # on
	$(VBM) modifyvm Redox --keyboard ps2
	$(VBM) modifyvm Redox --mouse ps2
	$(VBM) modifyvm Redox --audio $(VB_AUDIO)
	$(VBM) modifyvm Redox --audiocontroller ac97
	echo "Create Disk"
	$(VBM) convertfromraw $< $(BUILD)/harddrive.vdi
	echo "Attach Disk"
	#PATA
	# $(VBM) storagectl Redox --name ATA --add ide --controller PIIX4 --bootable on
	#SATA
	$(VBM) storagectl Redox --name ATA --add sata --controller IntelAHCI --bootable on --portcount 1
	$(VBM) storageattach Redox --storagectl ATA --port 0 --device 0 --type hdd --medium $(BUILD)/harddrive.vdi
	echo "Run VM"
	$(VBM) startvm Redox

bochs: $(BUILD)/harddrive.bin
	-bochs -f bochs.$(ARCH)

QFLAGS := -serial mon:stdio -m 1024 -d guest_errors

ifeq ($(machine),q35)
	QFLAGS += -machine q35
endif

ifneq ($(kvm),no)
	QFLAGS += -enable-kvm
endif

ifeq ($(vga),no)
	QFLAGS += -vga none -nographic
else
	QFLAGS += -vga std
endif

ifneq ($(audio),no)
	QFLAGS += -soundhw ac97
endif

ifneq ($(usb),no)
	QFLAGS += -usb

	ifeq ($(usb),ohci)
		QFLAGS += -device pci-ohci,id=ohci
   		QFLAGS += -device usb-tablet,bus=ohci.0
	else ifeq ($(usb),ehci)
		QFLAGS += -device usb-ehci,id=ehci
   		QFLAGS += -device usb-tablet,bus=ehci.0
	else ifeq ($(usb),xhci)
		QFLAGS += -device nec-usb-xhci,id=xhci
		QFLAGS += -device usb-tablet,bus=xhci.0
	else
		QFLAGS += -device usb-tablet
	endif
endif

ifeq ($(storage),ide)
	QFLAGS += -drive file=$(BUILD)/harddrive.bin,format=raw,index=0,media=disk
else ifeq ($(storage),usb)
	QFLAGS += -device usb-ehci,id=flash_bus -drive id=flash_drive,file=$(BUILD)/harddrive.bin,format=raw,if=none -device usb-storage,drive=flash_drive,bus=flash_bus.0
else
	QFLAGS += -device ahci,id=ahci -drive id=disk,file=$(BUILD)/harddrive.bin,format=raw,if=none -device ide-hd,drive=disk,bus=ahci.0
endif

ifeq ($(net),no)
	QFLAGS += -net none
else ifeq ($(net),tap)
	QFLAGS += -net nic,model=rtl8139 -net tap,ifname=tap_redox,script=no,downscript=no -net dump,file=$(BUILD)/network.pcap
else
	QFLAGS += -net nic,model=rtl8139 -net user -net dump,file=$(BUILD)/network.pcap
endif

qemu: $(BUILD)/harddrive.bin
	@if [ "$(net)" = "tap" ]; \
	then \
		sudo ip tuntap add dev tap_redox mode tap user "${USER}"; \
		sudo ifconfig tap_redox 10.85.85.1 up; \
	fi
	-$(QEMU) $(QFLAGS)
	@if [ "$(net)" = "tap" ]; \
	then \
		sudo ifconfig tap_redox down; \
		sudo ip tuntap del dev tap_redox mode tap; \
	fi

arping:
	arping -I tap_redox 10.85.85.2

ping:
	ping 10.85.85.2

wireshark:
	wireshark $(BUILD)/network.pcap

%:
	@echo "ERROR: Unknown target. Maybe you forgot to get the submodules (git submodule update --init --recursive)"
	exit 100

