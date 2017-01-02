ARCH?=x86_64

ROOT=$(PWD)
export RUST_TARGET_PATH=$(ROOT)/targets

#TODO: Use libssp
export CFLAGS=-nostartfiles -nostdlib -nodefaultlibs \
			-undef -imacros $(ROOT)/libc-artifacts/define.h \
			-isystem $(ROOT)/libc-artifacts/usr/include \
			-fno-stack-protector -U_FORTIFY_SOURCE
export CXXFLAGS=$(CFLAGS) -nostdinc++ \
	-isystem $(ROOT)/libc-artifacts/usr/include/c++/7.0.0 \
	-isystem $(ROOT)/libc-artifacts/usr/include/c++/7.0.0/x86_64-elf-redox

# Kernel variables
KTARGET=$(ARCH)-unknown-none
KBUILD=build/kernel
KRUSTC=./krustc.sh
KRUSTCFLAGS=--target $(KTARGET) -C opt-level=2 -C debuginfo=0 -C soft-float
KRUSTDOC=./krustdoc.sh
KCARGO=RUSTC="$(KRUSTC)" RUSTDOC="$(KRUSTDOC)" cargo
KCARGOFLAGS=--target $(KTARGET) --release -- -C soft-float

# Userspace variables
TARGET=$(ARCH)-unknown-redox
BUILD=build/userspace
RUSTC=./rustc.sh
RUSTCFLAGS=--target $(TARGET).json -C opt-level=2 -C debuginfo=0
RUSTDOC=./rustdoc.sh
CARGO=RUSTC="$(RUSTC)" RUSTDOC="$(RUSTDOC)" cargo
CARGOFLAGS=--target $(TARGET).json --release --

# Default targets
.PHONY: all live iso clean doc ref test update pull qemu bochs drivers schemes binutils coreutils extrautils netutils userutils wireshark FORCE

all: build/harddrive.bin

live: build/livedisk.bin

iso: build/livedisk.iso

FORCE:

clean:
	cargo clean
	cargo clean --manifest-path rust/src/libstd/Cargo.toml
	cargo clean --manifest-path drivers/ahcid/Cargo.toml
	cargo clean --manifest-path drivers/e1000d/Cargo.toml
	cargo clean --manifest-path drivers/ps2d/Cargo.toml
	cargo clean --manifest-path drivers/pcid/Cargo.toml
	cargo clean --manifest-path drivers/rtl8168d/Cargo.toml
	cargo clean --manifest-path drivers/vesad/Cargo.toml
	cargo clean --manifest-path programs/acid/Cargo.toml
	cargo clean --manifest-path programs/contain/Cargo.toml
	cargo clean --manifest-path programs/init/Cargo.toml
	cargo clean --manifest-path programs/ion/Cargo.toml
	cargo clean --manifest-path programs/binutils/Cargo.toml
	cargo clean --manifest-path programs/coreutils/Cargo.toml
	cargo clean --manifest-path programs/extrautils/Cargo.toml
	cargo clean --manifest-path programs/games/Cargo.toml
	cargo clean --manifest-path programs/netutils/Cargo.toml
	cargo clean --manifest-path programs/orbutils/Cargo.toml
	cargo clean --manifest-path programs/pkgutils/Cargo.toml
	cargo clean --manifest-path programs/userutils/Cargo.toml
	cargo clean --manifest-path programs/smith/Cargo.toml
	cargo clean --manifest-path programs/tar/Cargo.toml
	cargo clean --manifest-path schemes/ethernetd/Cargo.toml
	cargo clean --manifest-path schemes/example/Cargo.toml
	cargo clean --manifest-path schemes/ipd/Cargo.toml
	cargo clean --manifest-path schemes/orbital/Cargo.toml
	cargo clean --manifest-path schemes/ptyd/Cargo.toml
	cargo clean --manifest-path schemes/randd/Cargo.toml
	cargo clean --manifest-path schemes/redoxfs/Cargo.toml
	cargo clean --manifest-path schemes/tcpd/Cargo.toml
	cargo clean --manifest-path schemes/udpd/Cargo.toml
	-$(FUMOUNT) build/filesystem/
	rm -rf initfs/bin
	rm -rf filesystem/bin filesystem/sbin filesystem/ui/bin
	rm -rf build

doc: \
	doc-kernel \
	doc-std

#FORCE to let cargo decide if docs need updating
doc-kernel: $(KBUILD)/libkernel.a FORCE
	$(KCARGO) doc --target $(KTARGET).json

doc-std: $(BUILD)/libstd.rlib FORCE
	$(CARGO) doc --target $(TARGET).json --manifest-path rust/src/libstd/Cargo.toml

ref: FORCE
	rm -rf filesystem/ref/
	mkdir -p filesystem/ref/
	cargo run --manifest-path crates/docgen/Cargo.toml -- programs/binutils/src/bin/ filesystem/ref/
	cargo run --manifest-path crates/docgen/Cargo.toml -- programs/coreutils/src/bin/ filesystem/ref/
	cargo run --manifest-path crates/docgen/Cargo.toml -- programs/extrautils/src/bin/ filesystem/ref/
	cargo run --manifest-path crates/docgen/Cargo.toml -- programs/netutils/src/ filesystem/ref/

test:
	cargo test
	cargo test --manifest-path rust/src/libstd/Cargo.toml
	cargo test --manifest-path drivers/ahcid/Cargo.toml
	cargo test --manifest-path drivers/e1000d/Cargo.toml
	cargo test --manifest-path drivers/ps2d/Cargo.toml
	cargo test --manifest-path drivers/pcid/Cargo.toml
	cargo test --manifest-path drivers/rtl8168d/Cargo.toml
	cargo test --manifest-path drivers/vesad/Cargo.toml
	cargo test --manifest-path programs/acid/Cargo.toml
	cargo test --manifest-path programs/contain/Cargo.toml
	cargo test --manifest-path programs/init/Cargo.toml
	cargo test --manifest-path programs/ion/Cargo.toml
	cargo test --manifest-path programs/binutils/Cargo.toml
	cargo test --manifest-path programs/coreutils/Cargo.toml
	cargo test --manifest-path programs/extrautils/Cargo.toml
	cargo test --manifest-path programs/games/Cargo.toml
	cargo test --manifest-path programs/netutils/Cargo.toml
	cargo test --manifest-path programs/orbutils/Cargo.toml
	cargo test --manifest-path programs/pkgutils/Cargo.toml
	cargo test --manifest-path programs/userutils/Cargo.toml
	cargo test --manifest-path programs/smith/Cargo.toml
	cargo test --manifest-path programs/tar/Cargo.toml
	cargo test --manifest-path schemes/ethernetd/Cargo.toml
	cargo test --manifest-path schemes/example/Cargo.toml
	cargo test --manifest-path schemes/ipd/Cargo.toml
	cargo test --manifest-path schemes/orbital/Cargo.toml
	cargo test --manifest-path schemes/ptyd/Cargo.toml
	cargo test --manifest-path schemes/randd/Cargo.toml
	cargo test --manifest-path schemes/redoxfs/Cargo.toml
	cargo test --manifest-path schemes/tcpd/Cargo.toml
	cargo test --manifest-path schemes/udpd/Cargo.toml

update:
	cargo update
	cargo update --manifest-path rust/src/libstd/Cargo.toml
	cargo update --manifest-path drivers/ahcid/Cargo.toml
	cargo update --manifest-path drivers/e1000d/Cargo.toml
	cargo update --manifest-path drivers/ps2d/Cargo.toml
	cargo update --manifest-path drivers/pcid/Cargo.toml
	cargo update --manifest-path drivers/rtl8168d/Cargo.toml
	cargo update --manifest-path drivers/vesad/Cargo.toml
	cargo update --manifest-path programs/acid/Cargo.toml
	cargo update --manifest-path programs/contain/Cargo.toml
	cargo update --manifest-path programs/init/Cargo.toml
	cargo update --manifest-path programs/ion/Cargo.toml
	cargo update --manifest-path programs/binutils/Cargo.toml
	cargo update --manifest-path programs/coreutils/Cargo.toml
	cargo update --manifest-path programs/extrautils/Cargo.toml
	cargo update --manifest-path programs/games/Cargo.toml
	cargo update --manifest-path programs/netutils/Cargo.toml
	cargo update --manifest-path programs/orbutils/Cargo.toml
	cargo update --manifest-path programs/pkgutils/Cargo.toml
	cargo update --manifest-path programs/userutils/Cargo.toml
	cargo update --manifest-path programs/smith/Cargo.toml
	cargo update --manifest-path programs/tar/Cargo.toml
	cargo update --manifest-path schemes/ethernetd/Cargo.toml
	cargo update --manifest-path schemes/example/Cargo.toml
	cargo update --manifest-path schemes/ipd/Cargo.toml
	cargo update --manifest-path schemes/orbital/Cargo.toml
	cargo update --manifest-path schemes/ptyd/Cargo.toml
	cargo update --manifest-path schemes/randd/Cargo.toml
	cargo update --manifest-path schemes/redoxfs/Cargo.toml
	cargo update --manifest-path schemes/tcpd/Cargo.toml
	cargo update --manifest-path schemes/udpd/Cargo.toml

pull:
	git pull --rebase --recurse-submodules
	git submodule sync
	git submodule update --recursive --init
	git clean -X -f -d
	make clean
	make update

# Emulation
QEMU=SDL_VIDEO_X11_DGAMOUSE=0 qemu-system-$(ARCH)
QEMUFLAGS=-serial mon:stdio -d cpu_reset -d guest_errors
ifeq ($(ARCH),arm)
	QEMUFLAGS+=-cpu arm1176 -machine integratorcp
	QEMUFLAGS+=-nographic

	export CC=$(ARCH)-none-eabi-gcc
	export CXX=$(ARCH)-none-eabi-g++
	export LD=$(ARCH)-none-eabi-ld

	KRUSTCFLAGS+=-C linker=$(CC)
	KCARGOFLAGS+=-C linker=$(CC)
	RUSTCFLAGS+=-C linker=$(CC)
	CARGOFLAGS+=-C linker=$(CC)

%.list: %
	$(ARCH)-none-eabi-objdump -C -D $< > $@

build/harddrive.bin: $(KBUILD)/kernel
	cp $< $@

qemu: build/harddrive.bin
	$(QEMU) $(QEMUFLAGS) -kernel $<
else
	QEMUFLAGS+=-smp 4 -m 1024
	ifeq ($(iommu),yes)
		QEMUFLAGS+=-machine q35,iommu=on
	else
		QEMUFLAGS+=-machine q35
	endif
	ifeq ($(net),no)
		QEMUFLAGS+=-net none
	else
		QEMUFLAGS+=-net nic,model=e1000 -net user -net dump,file=build/network.pcap
		ifeq ($(net),redir)
			QEMUFLAGS+=-redir tcp:8080::8080
		endif
	endif
	ifeq ($(vga),no)
		QEMUFLAGS+=-nographic -vga none
	endif
	#,int,pcall
	#-device intel-iommu

	UNAME := $(shell uname)
	ifeq ($(UNAME),Darwin)
		export CC=$(ARCH)-elf-gcc
		export CXX=$(ARCH)-elf-g++
		ECHO=/bin/echo
		FUMOUNT=sudo umount
		export LD=$(ARCH)-elf-ld
		export LDFLAGS=--gc-sections
		VB_AUDIO=coreaudio
		VBM="/Applications/VirtualBox.app/Contents/MacOS/VBoxManage"
	else
		export CC=gcc
		export CXX=g++
		ECHO=echo
		FUMOUNT=fusermount -u
		export LD=ld
		export LDFLAGS=--gc-sections
		ifneq ($(kvm),no)
			QEMUFLAGS+=-enable-kvm -cpu host
		endif
		VB_AUDIO="pulse"
		VBM=VBoxManage
	endif

	KRUSTCFLAGS+=-C linker=$(CC) -C link-args="$(CFLAGS)"
	KCARGOFLAGS+=-C linker=$(CC) -C link-args="$(CFLAGS)"
	RUSTCFLAGS+=-C linker=$(CC) -C link-args="$(CFLAGS)"
	CARGOFLAGS+=-C linker=$(CC) -C link-args="$(CFLAGS)"

%.list: %
	objdump -C -M intel -D $< > $@

build/harddrive.bin: $(KBUILD)/kernel bootloader/$(ARCH)/** build/filesystem.bin
	nasm -f bin -o $@ -D ARCH_$(ARCH) -ibootloader/$(ARCH)/ bootloader/$(ARCH)/harddrive.asm

build/livedisk.bin: $(KBUILD)/kernel_live bootloader/$(ARCH)/**
	nasm -f bin -o $@ -D ARCH_$(ARCH) -ibootloader/$(ARCH)/ bootloader/$(ARCH)/livedisk.asm

build/%.bin.gz: build/%.bin
	gzip -k -f $<

build/livedisk.iso: build/livedisk.bin.gz
	rm -rf build/iso/
	mkdir -p build/iso/
	cp -RL isolinux build/iso/
	cp $< build/iso/livedisk.gz
	genisoimage -o $@ -b isolinux/isolinux.bin -c isolinux/boot.cat \
					-no-emul-boot -boot-load-size 4 -boot-info-table \
					build/iso/
	isohybrid $@

qemu: build/harddrive.bin
	$(QEMU) $(QEMUFLAGS) -drive file=$<,format=raw

qemu_extra: build/harddrive.bin
	if [ ! -e build/extra.bin ]; then dd if=/dev/zero of=build/extra.bin bs=1M count=1024; fi
	$(QEMU) $(QEMUFLAGS) -drive file=$<,format=raw -drive file=build/extra.bin,format=raw

qemu_no_build:
	$(QEMU) $(QEMUFLAGS) -drive file=build/harddrive.bin,format=raw

qemu_live: build/livedisk.bin
	$(QEMU) $(QEMUFLAGS) -device usb-ehci,id=flash_bus -drive id=flash_drive,file=$<,format=raw,if=none -device usb-storage,drive=flash_drive,bus=flash_bus.0

qemu_live_no_build:
	$(QEMU) $(QEMUFLAGS) -device usb-ehci,id=flash_bus -drive id=flash_drive,file=build/livedisk.bin,format=raw,if=none -device usb-storage,drive=flash_drive,bus=flash_bus.0

qemu_iso: build/livedisk.iso
	$(QEMU) $(QEMUFLAGS) -boot d -cdrom $<

qemu_iso_no_build:
		$(QEMU) $(QEMUFLAGS) -boot d -cdrom build/livedisk.iso

endif

bochs: build/harddrive.bin
	bochs -f bochs.$(ARCH)

virtualbox: build/harddrive.bin
	echo "Delete VM"
	-$(VBM) unregistervm Redox --delete; \
	if [ $$? -ne 0 ]; \
	then \
		if [ -d "$$HOME/VirtualBox VMs/Redox" ]; \
		then \
			echo "Redox directory exists, deleting..."; \
			$(RM) -rf "$$HOME/VirtualBox VMs/Redox"; \
		fi \
	fi
	echo "Delete Disk"
	-$(RM) harddrive.vdi
	echo "Create VM"
	$(VBM) createvm --name Redox --register
	echo "Set Configuration"
	$(VBM) modifyvm Redox --memory 1024
	$(VBM) modifyvm Redox --vram 16
	if [ "$(net)" != "no" ]; \
	then \
		$(VBM) modifyvm Redox --nic1 nat; \
		$(VBM) modifyvm Redox --nictype1 82540EM; \
		$(VBM) modifyvm Redox --cableconnected1 on; \
		$(VBM) modifyvm Redox --nictrace1 on; \
		$(VBM) modifyvm Redox --nictracefile1 build/network.pcap; \
	fi
	$(VBM) modifyvm Redox --uart1 0x3F8 4
	$(VBM) modifyvm Redox --uartmode1 file build/serial.log
	$(VBM) modifyvm Redox --usb off # on
	$(VBM) modifyvm Redox --keyboard ps2
	$(VBM) modifyvm Redox --mouse ps2
	$(VBM) modifyvm Redox --audio $(VB_AUDIO)
	$(VBM) modifyvm Redox --audiocontroller ac97
	$(VBM) modifyvm Redox --nestedpaging off
	echo "Create Disk"
	$(VBM) convertfromraw $< build/harddrive.vdi
	echo "Attach Disk"
	$(VBM) storagectl Redox --name ATA --add sata --controller IntelAHCI --bootable on --portcount 1
	$(VBM) storageattach Redox --storagectl ATA --port 0 --device 0 --type hdd --medium build/harddrive.vdi
	echo "Run VM"
	$(VBM) startvm Redox

# Kernel recipes
$(KBUILD)/libcore.rlib: rust/src/libcore/lib.rs
	mkdir -p $(KBUILD)
	$(KRUSTC) $(KRUSTCFLAGS) -o $@ $<

$(KBUILD)/librand.rlib: rust/src/librand/lib.rs $(KBUILD)/libcore.rlib
	$(KRUSTC) $(KRUSTCFLAGS) -o $@ $<

$(KBUILD)/liballoc.rlib: rust/src/liballoc/lib.rs $(KBUILD)/libcore.rlib
	$(KRUSTC) $(KRUSTCFLAGS) -o $@ $<

$(KBUILD)/libstd_unicode.rlib: rust/src/libstd_unicode/lib.rs $(KBUILD)/libcore.rlib
	$(KRUSTC) $(KRUSTCFLAGS) -o $@ $<

$(KBUILD)/libcollections.rlib: rust/src/libcollections/lib.rs $(KBUILD)/libcore.rlib $(KBUILD)/liballoc.rlib $(KBUILD)/libstd_unicode.rlib
	$(KRUSTC) $(KRUSTCFLAGS) -o $@ $<

$(KBUILD)/libkernel.a: kernel/** $(KBUILD)/libcore.rlib $(KBUILD)/liballoc.rlib $(KBUILD)/libcollections.rlib $(BUILD)/initfs.rs
	$(KCARGO) rustc $(KCARGOFLAGS) -C lto -o $@

$(KBUILD)/libkernel_live.a: kernel/** $(KBUILD)/libcore.rlib $(KBUILD)/liballoc.rlib $(KBUILD)/libcollections.rlib $(BUILD)/initfs.rs build/filesystem.bin
	$(KCARGO) rustc --lib $(KCARGOFLAGS) --cfg 'feature="live"' -C lto --emit obj=$@

$(KBUILD)/kernel: $(KBUILD)/libkernel.a
	$(LD) $(LDFLAGS) -z max-page-size=0x1000 -T arch/$(ARCH)/src/linker.ld -o $@ $<

$(KBUILD)/kernel_live: $(KBUILD)/libkernel_live.a
	$(LD) $(LDFLAGS) -z max-page-size=0x1000 -T arch/$(ARCH)/src/linker.ld -o $@ $<

# Userspace recipes
$(BUILD)/libstd.rlib: rust/src/libstd/Cargo.toml rust/src/libstd/**
	mkdir -p $(BUILD)
	$(CARGO) rustc --verbose --manifest-path $< $(CARGOFLAGS) -L native=libc-artifacts/usr/lib -o $@
	cp rust/src/target/$(TARGET)/release/deps/*.rlib $(BUILD)

initfs/bin/%: drivers/%/Cargo.toml drivers/%/src/** $(BUILD)/libstd.rlib
	mkdir -p initfs/bin
	$(CARGO) rustc --manifest-path $< $(CARGOFLAGS) -o $@
	strip $@

initfs/bin/%: programs/%/Cargo.toml programs/%/src/** $(BUILD)/libstd.rlib
	mkdir -p initfs/bin
	$(CARGO) rustc --manifest-path $< $(CARGOFLAGS) -o $@
	strip $@

initfs/bin/%: schemes/%/Cargo.toml schemes/%/src/** $(BUILD)/libstd.rlib
	mkdir -p initfs/bin
	$(CARGO) rustc --manifest-path $< --bin $* $(CARGOFLAGS) -o $@
	strip $@

$(BUILD)/initfs.rs: \
		initfs/bin/init \
		initfs/bin/ahcid \
		initfs/bin/bgad \
		initfs/bin/pcid \
		initfs/bin/ps2d \
		initfs/bin/redoxfs \
		initfs/bin/vesad \
		initfs/etc/**
	echo 'use collections::BTreeMap;' > $@
	echo 'pub fn gen() -> BTreeMap<&'"'"'static [u8], (&'"'"'static [u8], bool)> {' >> $@
	echo '    let mut files: BTreeMap<&'"'"'static [u8], (&'"'"'static [u8], bool)> = BTreeMap::new();' >> $@
	for folder in `find initfs -type d | sort`; do \
		name=$$(echo $$folder | sed 's/initfs//' | cut -d '/' -f2-) ; \
		$(ECHO) -n '    files.insert(b"'$$name'", (b"' >> $@ ; \
		ls -1 $$folder | sort | awk 'NR > 1 {printf("\\n")} {printf("%s", $$0)}' >> $@ ; \
		echo '", true));' >> $@ ; \
	done
	find initfs -type f -o -type l | cut -d '/' -f2- | sort | awk '{printf("    files.insert(b\"%s\", (include_bytes!(\"../../initfs/%s\"), false));\n", $$0, $$0)}' >> $@
	echo '    files' >> $@
	echo '}' >> $@

filesystem/sbin/%: drivers/%/Cargo.toml drivers/%/src/** $(BUILD)/libstd.rlib
	mkdir -p filesystem/sbin
	$(CARGO) rustc --manifest-path $< $(CARGOFLAGS) -o $@
	strip $@

filesystem/bin/%: programs/%/Cargo.toml programs/%/src/** $(BUILD)/libstd.rlib
	mkdir -p filesystem/bin
	$(CARGO) rustc --manifest-path $< --bin $* $(CARGOFLAGS) -o $@
	strip $@

filesystem/bin/sh: filesystem/bin/ion
	cp $< $@

filesystem/bin/%: programs/binutils/Cargo.toml programs/binutils/src/bin/%.rs $(BUILD)/libstd.rlib
	mkdir -p filesystem/bin
	$(CARGO) rustc --manifest-path $< --bin $* $(CARGOFLAGS) -o $@
	strip $@

filesystem/bin/%: programs/coreutils/Cargo.toml programs/coreutils/src/bin/%.rs $(BUILD)/libstd.rlib
	mkdir -p filesystem/bin
	$(CARGO) rustc --manifest-path $< --bin $* $(CARGOFLAGS) -o $@
	strip $@

filesystem/bin/%: programs/extrautils/Cargo.toml programs/extrautils/src/bin/%.rs $(BUILD)/libstd.rlib
	mkdir -p filesystem/bin
	$(CARGO) rustc --manifest-path $< --bin $* $(CARGOFLAGS) -o $@
	strip $@

filesystem/bin/%: programs/games/Cargo.toml programs/games/src/%/**.rs $(BUILD)/libstd.rlib
	mkdir -p filesystem/bin
	$(CARGO) rustc --manifest-path $< --bin $* $(CARGOFLAGS) -o $@
	strip $@

filesystem/bin/%: programs/netutils/Cargo.toml programs/netutils/src/%/**.rs $(BUILD)/libstd.rlib
	mkdir -p filesystem/bin
	$(CARGO) rustc --manifest-path $< --bin $* $(CARGOFLAGS) -o $@
	strip $@

filesystem/ui/bin/%: programs/orbutils/Cargo.toml programs/orbutils/src/%/**.rs $(BUILD)/libstd.rlib
	mkdir -p filesystem/ui/bin
	$(CARGO) rustc --manifest-path $< --bin $* $(CARGOFLAGS) -o $@
	strip $@

filesystem/bin/%: programs/pkgutils/Cargo.toml programs/pkgutils/src/%/**.rs $(BUILD)/libstd.rlib
	mkdir -p filesystem/bin
	$(CARGO) rustc --manifest-path $< --bin $* $(CARGOFLAGS) -o $@
	strip $@

filesystem/bin/%: programs/userutils/Cargo.toml programs/userutils/src/bin/%.rs $(BUILD)/libstd.rlib
	mkdir -p filesystem/bin
	$(CARGO) rustc --manifest-path $< --bin $* $(CARGOFLAGS) -o $@
	strip $@

filesystem/sbin/%: schemes/%/Cargo.toml schemes/%/src/** $(BUILD)/libstd.rlib
	mkdir -p filesystem/sbin
	$(CARGO) rustc --manifest-path $< --bin $* $(CARGOFLAGS) -o $@
	strip $@

filesystem/sbin/redoxfs-mkfs: schemes/redoxfs/Cargo.toml schemes/redoxfs/src/** $(BUILD)/libstd.rlib
	mkdir -p filesystem/bin
	$(CARGO) rustc --manifest-path $< --bin redoxfs-mkfs $(CARGOFLAGS) -o $@
	strip $@

drivers: \
	filesystem/sbin/pcid \
	filesystem/sbin/e1000d \
	filesystem/sbin/rtl8168d

binutils: \
	filesystem/bin/hex \
	filesystem/bin/hexdump \
	filesystem/bin/strings

coreutils: \
	filesystem/bin/basename \
	filesystem/bin/cat \
	filesystem/bin/chmod \
	filesystem/bin/clear \
	filesystem/bin/cp \
	filesystem/bin/cut \
	filesystem/bin/date \
	filesystem/bin/dd \
	filesystem/bin/df \
	filesystem/bin/du \
	filesystem/bin/echo \
	filesystem/bin/env \
	filesystem/bin/false \
	filesystem/bin/free \
	filesystem/bin/head \
	filesystem/bin/kill \
	filesystem/bin/ls \
	filesystem/bin/mkdir \
	filesystem/bin/mv \
	filesystem/bin/printenv \
	filesystem/bin/ps \
	filesystem/bin/pwd \
	filesystem/bin/realpath \
	filesystem/bin/reset \
	filesystem/bin/rmdir \
	filesystem/bin/rm \
	filesystem/bin/seq \
	filesystem/bin/sleep \
	filesystem/bin/sort \
	filesystem/bin/tail \
	filesystem/bin/tee \
	filesystem/bin/time \
	filesystem/bin/touch \
	filesystem/bin/true \
	filesystem/bin/wc \
	filesystem/bin/yes
	#filesystem/bin/shutdown filesystem/bin/test

extrautils: \
	filesystem/bin/calc \
	filesystem/bin/cksum \
	filesystem/bin/cur \
	filesystem/bin/grep \
	filesystem/bin/less \
	filesystem/bin/man \
	filesystem/bin/mdless \
	filesystem/bin/mtxt \
	filesystem/bin/rem \
	#filesystem/bin/dmesg filesystem/bin/info  filesystem/bin/watch

games: \
	filesystem/bin/ice \
	filesystem/bin/minesweeper \
	filesystem/bin/reblox \
	filesystem/bin/rusthello \
	filesystem/bin/snake

netutils: \
	filesystem/bin/dhcpd \
	filesystem/bin/dns \
	filesystem/bin/httpd \
	filesystem/bin/irc \
	filesystem/bin/nc \
	filesystem/bin/ntp \
	filesystem/bin/wget

orbutils: \
	filesystem/ui/bin/browser \
	filesystem/ui/bin/calculator \
	filesystem/ui/bin/character_map \
	filesystem/ui/bin/editor \
	filesystem/ui/bin/file_manager \
	filesystem/ui/bin/launcher \
	filesystem/ui/bin/orblogin \
	filesystem/ui/bin/terminal \
	filesystem/ui/bin/viewer

pkgutils: \
	filesystem/bin/pkg

userutils: \
	filesystem/bin/getty \
	filesystem/bin/id \
	filesystem/bin/login \
	filesystem/bin/passwd \
	filesystem/bin/su \
	filesystem/bin/sudo

schemes: \
	filesystem/sbin/ethernetd \
	filesystem/sbin/ipd \
	filesystem/sbin/orbital \
	filesystem/sbin/ptyd \
	filesystem/sbin/randd \
	filesystem/sbin/redoxfs \
	filesystem/sbin/redoxfs-mkfs \
	filesystem/sbin/tcpd \
	filesystem/sbin/udpd

build/filesystem.bin: \
		drivers \
		coreutils \
		extrautils \
		games \
		netutils \
		orbutils \
		pkgutils \
		userutils \
		schemes \
		filesystem/bin/acid \
		filesystem/bin/contain \
		filesystem/bin/ion \
		filesystem/bin/sh \
		filesystem/bin/smith \
		filesystem/bin/tar
	-$(FUMOUNT) build/filesystem/
	rm -rf $@ build/filesystem/
	dd if=/dev/zero of=$@ bs=1M count=64
	cargo run --manifest-path schemes/redoxfs/Cargo.toml --release --bin redoxfs-mkfs $@
	mkdir -p build/filesystem/
	cargo build --manifest-path schemes/redoxfs/Cargo.toml --release --bin redoxfs
	schemes/redoxfs/target/release/redoxfs $@ build/filesystem/
	sleep 2
	pgrep redoxfs
	cp -RL filesystem/* build/filesystem/
	chown -R 0:0 build/filesystem
	chown -R 1000:1000 build/filesystem/home/user
	chmod -R uog+rX build/filesystem
	chmod -R u+w build/filesystem
	chmod -R og-w build/filesystem
	chmod -R 755 build/filesystem/bin
	chmod -R u+rwX build/filesystem/root
	chmod -R og-rwx build/filesystem/root
	chmod -R u+rwX build/filesystem/home/user
	chmod -R og-rwx build/filesystem/home/user
	chmod +s build/filesystem/bin/passwd
	chmod +s build/filesystem/bin/su
	chmod +s build/filesystem/bin/sudo
	mkdir build/filesystem/tmp
	chmod 1777 build/filesystem/tmp
	sync
	-$(FUMOUNT) build/filesystem/
	rm -rf build/filesystem/

mount: FORCE
	mkdir -p build/filesystem/
	cargo build --manifest-path schemes/redoxfs/Cargo.toml --release --bin redoxfs
	schemes/redoxfs/target/release/redoxfs build/harddrive.bin build/filesystem/
	sleep 2
	pgrep redoxfs

unmount: FORCE
	sync
	-$(FUMOUNT) build/filesystem/
	rm -rf build/filesystem/

wireshark: FORCE
	wireshark build/network.pcap
