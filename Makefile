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
LD=ld
LDARGS=-m elf_$(ARCH)
MAKE=make
MKDIR=mkdir
OBJDUMP=objdump
RM=rm
SED=sed
SORT=sort
VB=virtualbox
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

ifeq ($(OS),Windows_NT)
	SHELL=windows\sh
	LD=windows/$(ARCH)-elf-ld
	CARGOFLAGS += -C ar=windows/$(ARCH)-elf-ar -C linker=windows/$(ARCH)-elf-gcc -C link-args="-v -fno-use-linker-plugin"
	RUSTCFLAGS += -C ar=windows/$(ARCH)-elf-ar -C linker=windows/$(ARCH)-elf-gcc -C link-args="-v -fno-use-linker-plugin"
	AS=windows/nasm
	AWK=windows/awk
	BASENAME=windows/basename
	CUT=windows/cut
	DATE=windows/date
	FIND=windows/find
	MAKE=windows/make
	MKDIR=windows/mkdir
	OBJDUMP=windows/i386-elf-objdump
	RM=windows/rm
	SED=windows/sed
	SORT=windows/sort
	VB="C:/Program Files/Oracle/VirtualBox/VirtualBox"
	VB_AUDIO="dsound"
	VBM="C:/Program Files/Oracle/VirtualBox/VBoxManage"
	VBM_CLEANUP=\
		if [ $$? -ne 0 ]; \
		then \
			if [ -d "$$USERPROFILE/VirtualBox VMs/Redox" ]; \
			then \
				echo "Redox directory exists, deleting..."; \
				$(RM) -rf "$$USERPROFILE/VirtualBox VMs/Redox"; \
			fi \
		fi
else
	UNAME := $(shell uname)
	ifeq ($(UNAME),Darwin)
		LD=$(ARCH)-elf-ld
		OBJDUMP=$(ARCH)-elf-objdump
		CARGOFLAGS += -C ar=$(ARCH)-elf-ar -C linker=$(ARCH)-elf-gcc
		RUSTCFLAGS += -C ar=$(ARCH)-elf-ar -C linker=$(ARCH)-elf-gcc
		VB="/Applications/VirtualBox.app/Contents/MacOS/VirtualBox"
		VB_AUDIO="coreaudio"
		VBM="/Applications/VirtualBox.app/Contents/MacOS/VBoxManage"
	endif
endif

.PHONY: help all doc apps bins clean \
	bochs \
	qemu qemu_bare qemu_tap \
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

filesystem/apps/rusthello/main.bin: filesystem/apps/rusthello/src/main.rs filesystem/apps/rusthello/src/*.rs filesystem/apps/rusthello/src/*/*.rs $(BUILD)/crt0.o $(BUILD)/libstd.rlib
	$(RUSTC) $(RUSTCFLAGS) --crate-type bin -o $@ $<

filesystem/apps/sodium/main.bin: filesystem/apps/sodium/src/main.rs filesystem/apps/sodium/src/*.rs $(BUILD)/libstd.rlib $(BUILD)/liborbclient.rlib
	$(RUSTC) $(RUSTCFLAGS) --crate-type bin -o $@ $< --cfg 'feature="orbital"'

filesystem/apps/example/main.bin: filesystem/apps/example/main.rs filesystem/apps/example/*.rs $(BUILD)/crt0.o $(BUILD)/libstd.rlib
	$(RUSTC) $(RUSTCFLAGS) --crate-type bin -o $@ $<

filesystem/apps/%/main.bin: filesystem/apps/%/main.rs filesystem/apps/%/*.rs $(BUILD)/crt0.o $(BUILD)/libstd.rlib $(BUILD)/liborbclient.rlib $(BUILD)/liborbtk.rlib
	$(RUSTC) $(RUSTCFLAGS) --crate-type bin -o $@ $<

filesystem/apps/%/main.bin: crates/orbutils/src/%/main.rs crates/orbutils/src/%/*.rs $(BUILD)/crt0.o $(BUILD)/libstd.rlib $(BUILD)/liborbclient.rlib $(BUILD)/liborbtk.rlib
	$(RUSTC) $(RUSTCFLAGS) --crate-type bin -o $@ $<

apps: filesystem/apps/editor/main.bin \
	  filesystem/apps/file_manager/main.bin \
	  filesystem/apps/orbtk/main.bin \
	  filesystem/apps/player/main.bin \
	  filesystem/apps/rusthello/main.bin \
	  filesystem/apps/sodium/main.bin \
	  filesystem/apps/terminal/main.bin \
	  filesystem/apps/viewer/main.bin

$(BUILD)/libcoreutils.rlib: crates/coreutils/src/lib.rs crates/coreutils/src/*.rs $(BUILD)/libstd.rlib
	$(RUSTC) $(RUSTCFLAGS) --crate-name coreutils --crate-type lib -o $@ $<

filesystem/bin/%: crates/coreutils/src/bin/%.rs $(BUILD)/crt0.o $(BUILD)/libcoreutils.rlib
	mkdir -p filesystem/bin
	$(RUSTC) $(RUSTCFLAGS) --crate-type bin -o $@ $<

coreutils: \
	filesystem/bin/basename \
	filesystem/bin/cat \
	filesystem/bin/cp \
	filesystem/bin/du \
	filesystem/bin/echo \
	filesystem/bin/false \
	filesystem/bin/free \
	filesystem/bin/head \
	filesystem/bin/ls \
	filesystem/bin/mkdir \
	filesystem/bin/ps \
	filesystem/bin/pwd \
	filesystem/bin/realpath \
	filesystem/bin/rm \
	filesystem/bin/rmdir \
	filesystem/bin/seq \
	filesystem/bin/shutdown \
	filesystem/bin/sleep \
	filesystem/bin/tail \
	filesystem/bin/touch \
	filesystem/bin/true \
	filesystem/bin/wc \
	filesystem/bin/yes
	#TODO: filesystem/bin/env filesystem/bin/test


$(BUILD)/libbinutils.rlib: crates/binutils/src/lib.rs crates/binutils/src/*.rs $(BUILD)/libcoreutils.rlib
	$(RUSTC) $(RUSTCFLAGS) --crate-name binutils --crate-type lib -o $@ $<

filesystem/bin/%: crates/binutils/src/bin/%.rs $(BUILD)/crt0.o $(BUILD)/libbinutils.rlib
	mkdir -p filesystem/bin
	$(RUSTC) $(RUSTCFLAGS) --crate-type bin -o $@ $<

binutils: \
	filesystem/bin/hex \
	filesystem/bin/hexdump \
	filesystem/bin/strings

$(BUILD)/libtermion.rlib: crates/termion/src/lib.rs crates/termion/src/*.rs $(BUILD)/libstd.rlib
	$(RUSTC) $(RUSTCFLAGS) --crate-name termion --crate-type lib -o $@ $< --cfg 'feature="nightly"'

filesystem/bin/%: crates/extrautils/src/bin/%.rs $(BUILD)/crt0.o $(BUILD)/libcoreutils.rlib $(BUILD)/libtermion.rlib
	mkdir -p filesystem/bin
	$(RUSTC) $(RUSTCFLAGS) --crate-type bin -o $@ $<

extrautils: \
	filesystem/bin/calc \
	filesystem/bin/cksum \
	filesystem/bin/cur \
	filesystem/bin/grep \
	filesystem/bin/less \
	filesystem/bin/rem
	#TODO: filesystem/bin/mtxt

filesystem/bin/%: crates/%/main.rs crates/%/*.rs $(BUILD)/crt0.o $(BUILD)/libstd.rlib
	mkdir -p filesystem/bin
	$(RUSTC) $(RUSTCFLAGS) --crate-type bin -o $@ $<

filesystem/bin/%: libc/bin/%
	mkdir -p filesystem/bin
	cp $< $@

$(BUILD)/ion-shell.bin: FORCE $(BUILD)/libstd.rlib
	$(CARGO) --manifest-path crates/ion/Cargo.toml --bin ion-shell $(CARGOFLAGS)

filesystem/bin/ion: $(BUILD)/ion-shell.bin
	mkdir -p filesystem/bin
	cp $< $@

filesystem/bin/sh: $(BUILD)/ion-shell.bin
	mkdir -p filesystem/bin
	cp $< $@

filesystem/bin/launcher: crates/orbutils/src/launcher/main.rs crates/orbutils/src/launcher/*.rs $(BUILD)/crt0.o $(BUILD)/libstd.rlib $(BUILD)/liborbclient.rlib $(BUILD)/liborbtk.rlib
	mkdir -p filesystem/bin
	$(RUSTC) $(RUSTCFLAGS) --crate-type bin -o $@ $<


bins: \
	coreutils \
	extrautils \
	filesystem/bin/ansi-test \
	filesystem/bin/c-test \
	filesystem/bin/dosbox \
	filesystem/bin/ed \
	filesystem/bin/example \
	filesystem/bin/init \
  	filesystem/bin/ion \
	filesystem/bin/launcher \
  	filesystem/bin/lua \
  	filesystem/bin/login \
  	filesystem/bin/orbital \
	filesystem/bin/std-test \
  	filesystem/bin/sdl-test \
  	filesystem/bin/sdl-ttf-test \
  	filesystem/bin/sh \
	filesystem/bin/tar \
	filesystem/bin/zfs
	#TODO: binutils

initfs/redoxfsd: crates/redoxfs/scheme/main.rs crates/redoxfs/scheme/*.rs $(BUILD)/crt0.o $(BUILD)/libstd.rlib $(BUILD)/libredoxfs.rlib
	mkdir -p initfs/
	$(RUSTC) $(RUSTCFLAGS) --crate-type bin -o $@ $<

initfs/build-arch: FORCE
	mkdir -p initfs/
	echo $(ARCH) > $@

initfs/build-branch: FORCE
	mkdir -p initfs/
	git rev-parse --abbrev-ref HEAD > $@

initfs/build-cargo: FORCE
	mkdir -p initfs/
	cargo -V > $@

initfs/build-date: FORCE
	mkdir -p initfs/
	date > $@

initfs/build-host: FORCE
	mkdir -p initfs/
	uname -a > $@

initfs/build-rustc: FORCE
	mkdir -p initfs/
	$(RUSTC) -V > $@

initfs/build-rev: FORCE
	mkdir -p initfs/
	git rev-parse HEAD > $@

build/initfs.gen: \
		initfs/redoxfsd \
		initfs/build-arch \
		initfs/build-branch \
		initfs/build-cargo \
		initfs/build-date \
		initfs/build-host \
		initfs/build-rustc \
		initfs/build-rev
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
	$(RM) -rf build doc filesystem/bin/ initfs/bin/ filesystem/apps/*/*.bin filesystem/apps/*/*.list

FORCE:

doc/core: rust/src/libcore/lib.rs $(BUILD)/libcore.rlib
	$(RUSTDOC) --cfg disable_float $<

doc/alloc_system: liballoc_system/lib.rs $(BUILD)/liballoc_system.rlib doc/core
	$(RUSTDOC) $<

doc/alloc: rust/src/liballoc/lib.rs $(BUILD)/liballoc.rlib doc/alloc_system
	$(RUSTDOC) $<

doc/rustc_unicode: rust/src/librustc_unicode/lib.rs $(BUILD)/librustc_unicode.rlib doc/core
	$(RUSTDOC) $<

doc/collections: rust/src/libcollections/lib.rs $(BUILD)/libcollections.rlib doc/alloc doc/rustc_unicode
	$(RUSTDOC) $<

doc/rand: rust/src/librand/lib.rs $(BUILD)/librand.rlib doc/collections
	$(RUSTDOC) --cfg disable_float $<

doc/io: crates/io/lib.rs crates/io/*.rs $(BUILD)/libio.rlib doc/core
	$(RUSTDOC) $<

doc/system: crates/system/lib.rs crates/system/*.rs crates/system/*/*.rs $(BUILD)/libsystem.rlib doc/core
	$(RUSTDOC) $<

doc/redoxfs: crates/redoxfs/src/lib.rs crates/redoxfs/src/*.rs doc/system doc/alloc doc/collections
	$(RUSTDOC) $<

doc/kernel: kernel/main.rs kernel/*.rs kernel/*/*.rs kernel/*/*/*.rs $(BUILD)/kernel.rlib doc/io doc/redoxfs
	$(RUSTDOC) $<

doc/std: libstd/src/lib.rs libstd/src/*.rs libstd/src/*/*.rs libstd/src/*/*/*.rs $(BUILD)/libstd.rlib doc/rand doc/system
	$(RUSTDOC) --cfg disable_float --crate-name=std $<

doc: doc/kernel doc/std

man: filesystem/man

filesystem/man:
	mkdir man \
	rm -rf filesystem/man |& true \
	cd crates/docgen \
	cargo build --release \
	cd ../../ \
	./crates/docgen/target/release/docgen \
	mv man filesystem

$(BUILD)/libcore.rlib: rust/src/libcore/lib.rs
	$(MKDIR) -p $(BUILD)
	$(RUSTC) $(RUSTCFLAGS) -o $@ $<

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

$(BUILD)/liblibc.rlib: rust/src/liblibc/src/lib.rs $(BUILD)/libcore.rlib
	$(RUSTC) $(RUSTCFLAGS) -o $@ $<

$(BUILD)/librealstd.rlib: rust/src/libstd/lib.rs $(BUILD)/libcore.rlib $(BUILD)/liblibc.rlib $(BUILD)/liballoc.rlib $(BUILD)/librustc_unicode.rlib $(BUILD)/libcollections.rlib $(BUILD)/librand.rlib
	$(RUSTC) $(RUSTCFLAGS) --cfg unix --crate-type rlib -o $@ $<

$(BUILD)/libstd.rlib: libstd/src/lib.rs libstd/src/*.rs libstd/src/*/*.rs libstd/src/*/*/*.rs $(BUILD)/libcore.rlib $(BUILD)/liballoc.rlib $(BUILD)/libcollections.rlib $(BUILD)/librand.rlib $(BUILD)/libsystem.rlib
	$(RUSTC) $(RUSTCFLAGS) -o $@ $<

$(BUILD)/liborbclient.rlib: crates/orbclient/src/lib.rs crates/orbclient/src/*.rs crates/orbclient/src/*/*.rs $(BUILD)/libstd.rlib
	$(RUSTC) $(RUSTCFLAGS) -o $@ $<

$(BUILD)/liborbtk.rlib: crates/orbtk/src/lib.rs crates/orbtk/src/*.rs $(BUILD)/libstd.rlib $(BUILD)/liborbclient.rlib
	$(RUSTC) $(RUSTCFLAGS) -o $@ $<

#Kernel stuff
$(BUILD)/libio.rlib: crates/io/lib.rs crates/io/*.rs $(BUILD)/libcore.rlib
	$(RUSTC) $(RUSTCFLAGS) -o $@ $<

$(BUILD)/libsystem.rlib: crates/system/lib.rs crates/system/*.rs crates/system/*/*.rs $(BUILD)/libcore.rlib
	$(RUSTC) $(RUSTCFLAGS) -o $@ $<

$(BUILD)/libredoxfs.rlib: crates/redoxfs/src/lib.rs crates/redoxfs/src/*.rs $(BUILD)/libsystem.rlib $(BUILD)/liballoc.rlib $(BUILD)/libcollections.rlib
	$(RUSTC) $(RUSTCFLAGS) -o $@ $<

$(BUILD)/kernel.rlib: kernel/main.rs kernel/*.rs kernel/*/*.rs kernel/*/*/*.rs  $(BUILD)/libio.rlib build/initfs.gen
	$(RUSTC) $(RUSTCFLAGS) -C lto -o $@ $<

$(BUILD)/kernel.bin: $(BUILD)/kernel.rlib kernel/kernel.ld
	$(LD) $(LDARGS) -o $@ -T kernel/kernel.ld -z max-page-size=0x1000 $<

$(BUILD)/kernel.list: $(BUILD)/kernel.bin
	$(OBJDUMP) -C -M intel -D $< > $@

$(BUILD)/kernel.asm: kernel/main.rs $(BUILD)/libcore.rlib $(BUILD)/liballoc.rlib $(BUILD)/libcollections.rlib
	$(RUSTC) $(RUSTCFLAGS) -C lto -o $@ --emit asm $<

$(BUILD)/kernel.ir: kernel/main.rs $(BUILD)/libcore.rlib $(BUILD)/liballoc.rlib $(BUILD)/libcollections.rlib
	$(RUSTC) $(RUSTCFLAGS) -C lto -o $@ --emit llvm-ir $<

$(BUILD)/crt0.o: kernel/program-$(ARCH).asm
	$(MKDIR) -p $(BUILD)
ifeq ($(ARCH),x86_64)
	$(AS) -f elf64 -o $@ $<
else
	$(AS) -f elf -o $@ $<
endif

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

$(BUILD)/filesystem.gen: apps bins
	$(FIND) filesystem -type f -o -type l | $(CUT) -d '/' -f2- | $(SORT) | $(AWK) '{printf("file %d,\"%s\"\n", NR, $$0)}' > $@

$(BUILD)/harddrive.bin: kernel/harddrive.asm $(BUILD)/kernel.bin $(BUILD)/filesystem.gen
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
	$(VBM) modifyvm Redox --nic1 nat
	$(VBM) modifyvm Redox --nictype1 82540EM
	$(VBM) modifyvm Redox --nictrace1 on
	$(VBM) modifyvm Redox --nictracefile1 $(BUILD)/network.pcap
	$(VBM) modifyvm Redox --uart1 0x3F8 4
	$(VBM) modifyvm Redox --uartmode1 file $(BUILD)/serial.log
	$(VBM) modifyvm Redox --usb on
	$(VBM) modifyvm Redox --mouse usbtablet
	#$(VBM) modifyvm Redox --audio $(VB_AUDIO)
	#$(VBM) modifyvm Redox --audiocontroller ac97
	echo "Create Disk"
	$(VBM) convertfromraw $< $(BUILD)/harddrive.vdi
	echo "Attach Disk"
	#PATA
	#$(VBM) storagectl Redox --name ATA --add ide --controller PIIX4 --bootable on
	#SATA
	$(VBM) storagectl Redox --name ATA --add sata --controller IntelAHCI --bootable on --portcount 1
	$(VBM) storageattach Redox --storagectl ATA --port 0 --device 0 --type hdd --medium $(BUILD)/harddrive.vdi
	echo "Run VM"
	$(VB) --startvm Redox --dbg

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

ifeq ($(storage),ahci)
	QFLAGS += -device ahci,id=ahci -drive id=disk,file=$(BUILD)/harddrive.bin,format=raw,if=none -device ide-hd,drive=disk,bus=ahci.0
else ifeq ($(storage),usb)
	QFLAGS += -device usb-ehci,id=flash_bus -drive id=flash_drive,file=$(BUILD)/harddrive.bin,format=raw,if=none -device usb-storage,drive=flash_drive,bus=flash_bus.0
else
	QFLAGS += -drive file=$(BUILD)/harddrive.bin,format=raw,index=0,media=disk
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
		sudo tunctl -t tap_redox -u "${USER}"; \
		sudo ifconfig tap_redox 10.85.85.1 up; \
	fi
	-$(QEMU) $(QFLAGS)
	@if [ "$(net)" = "tap" ]; \
	then \
		sudo ifconfig tap_redox down; \
		sudo tunctl -d tap_redox; \
	fi

arping:
	arping -I tap_redox 10.85.85.2

ping:
	ping 10.85.85.2

wireshark:
	wireshark $(BUILD)/network.pcap
