ARCH?=x86_64

# Kernel variables
KTARGET=$(ARCH)-unknown-none
KBUILD=build/kernel
KRUSTC=./krustc.sh
KRUSTCFLAGS=--target $(KTARGET).json -C opt-level=2 -C debuginfo=0 -C soft-float
KRUSTDOC=./krustdoc.sh
KCARGO=RUSTC="$(KRUSTC)" RUSTDOC="$(KRUSTDOC)" cargo
KCARGOFLAGS=--target $(KTARGET).json --release -- -C soft-float

# Userspace variables
TARGET=$(ARCH)-unknown-redox
BUILD=build/userspace
RUSTC=./rustc.sh
RUSTCFLAGS=--target $(TARGET).json -C opt-level=2 -C debuginfo=0 --cfg redox
RUSTDOC=./rustdoc.sh
CARGO=RUSTC="$(RUSTC)" RUSTDOC="$(RUSTDOC)" cargo
CARGOFLAGS=--target $(TARGET).json --release -- --cfg redox

# Default targets
.PHONY: all clean doc update qemu bochs drivers schemes coreutils extrautils netutils userutils wireshark FORCE

all: $(KBUILD)/harddrive.bin

FORCE:

clean:
	cargo clean
	cargo clean --manifest-path libstd/Cargo.toml
	cargo clean --manifest-path libstd_real/Cargo.toml
	cargo clean --manifest-path drivers/ahcid/Cargo.toml
	cargo clean --manifest-path drivers/e1000d/Cargo.toml
	cargo clean --manifest-path drivers/ps2d/Cargo.toml
	cargo clean --manifest-path drivers/pcid/Cargo.toml
	cargo clean --manifest-path drivers/rtl8168d/Cargo.toml
	cargo clean --manifest-path drivers/vesad/Cargo.toml
	cargo clean --manifest-path programs/acid/Cargo.toml
	cargo clean --manifest-path programs/init/Cargo.toml
	cargo clean --manifest-path programs/ion/Cargo.toml
	cargo clean --manifest-path programs/coreutils/Cargo.toml
	cargo clean --manifest-path programs/extrautils/Cargo.toml
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
	-$(FUMOUNT) $(BUILD)/filesystem/
	rm -rf initfs/bin
	rm -rf filesystem/bin
	rm -rf build

doc: \
	doc-kernel \
	doc-std

#FORCE to let cargo decide if docs need updating
doc-kernel: $(KBUILD)/libkernel.a FORCE
	$(KCARGO) doc --target $(KTARGET).json

doc-std: $(BUILD)/libstd.rlib FORCE
	$(CARGO) doc --target $(TARGET).json --manifest-path libstd/Cargo.toml

update:
	cargo update
	cargo update --manifest-path libstd/Cargo.toml
	cargo update --manifest-path libstd_real/Cargo.toml
	cargo update --manifest-path drivers/ahcid/Cargo.toml
	cargo update --manifest-path drivers/e1000d/Cargo.toml
	cargo update --manifest-path drivers/ps2d/Cargo.toml
	cargo update --manifest-path drivers/pcid/Cargo.toml
	cargo update --manifest-path drivers/rtl8168d/Cargo.toml
	cargo update --manifest-path drivers/vesad/Cargo.toml
	cargo update --manifest-path programs/acid/Cargo.toml
	cargo update --manifest-path programs/init/Cargo.toml
	cargo update --manifest-path programs/ion/Cargo.toml
	cargo update --manifest-path programs/coreutils/Cargo.toml
	cargo update --manifest-path programs/extrautils/Cargo.toml
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

# Emulation
QEMU=SDL_VIDEO_X11_DGAMOUSE=0 qemu-system-$(ARCH)
QEMUFLAGS=-serial mon:stdio -d cpu_reset -d guest_errors
ifeq ($(ARCH),arm)
	LD=$(ARCH)-none-eabi-ld
	QEMUFLAGS+=-cpu arm1176 -machine integratorcp
	QEMUFLAGS+=-nographic

%.list: %
	$(ARCH)-none-eabi-objdump -C -D $< > $@

$(KBUILD)/harddrive.bin: $(KBUILD)/kernel
	cp $< $@

qemu: $(KBUILD)/harddrive.bin
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
		QEMUFLAGS+=-net nic,model=e1000 -net user -net dump,file=$(KBUILD)/network.pcap
		ifeq ($(net),redir)
			QEMUFLAGS+=-redir tcp:8080::8080
		endif
	endif
	ifeq ($(storage),usb)
		QEMUFLAGS+=-device usb-ehci,id=flash_bus -drive id=flash_drive,file=$(KBUILD)/harddrive.bin,format=raw,if=none -device usb-storage,drive=flash_drive,bus=flash_bus.0
	else
		QEMUFLAGS+=-drive file=$(KBUILD)/harddrive.bin,format=raw
	endif
	ifeq ($(vga),no)
		QEMUFLAGS+=-nographic -vga none
	endif
	#,int,pcall
	#-device intel-iommu

	UNAME := $(shell uname)
	ifeq ($(UNAME),Darwin)
		CC=$(ARCH)-elf-gcc
		CXX=$(ARCH)-elf-g++
		ECHO=/bin/echo
		FUMOUNT=sudo umount
		LD=$(ARCH)-elf-ld
		LDFLAGS=--gc-sections
		KRUSTCFLAGS+=-C linker=$(CC)
		KCARGOFLAGS+=-C linker=$(CC)
		RUSTCFLAGS+=-C linker=$(CC)
		CARGOFLAGS+=-C linker=$(CC)
		VB_AUDIO=coreaudio
		VBM="/Applications/VirtualBox.app/Contents/MacOS/VBoxManage"
	else
		CC=gcc
		CXX=g++
		ECHO=echo
		FUMOUNT=fusermount -u
		LD=ld
		LDFLAGS=--gc-sections
		ifneq ($(kvm),no)
			QEMUFLAGS+=-enable-kvm -cpu host
		endif
		VB_AUDIO="pulse"
		VBM=VBoxManage
	endif

%.list: %
	objdump -C -M intel -D $< > $@

$(KBUILD)/harddrive.bin: $(KBUILD)/kernel $(BUILD)/filesystem.bin bootloader/$(ARCH)/**
	nasm -f bin -o $@ -D ARCH_$(ARCH) -ibootloader/$(ARCH)/ bootloader/$(ARCH)/harddrive.asm

qemu: $(KBUILD)/harddrive.bin
	$(QEMU) $(QEMUFLAGS)

qemu_no_build:
	$(QEMU) $(QEMUFLAGS)
endif

bochs: $(KBUILD)/harddrive.bin
	bochs -f bochs.$(ARCH)

virtualbox: $(KBUILD)/harddrive.bin
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
	$(VBM) modifyvm Redox --nic1 nat
	$(VBM) modifyvm Redox --nictype1 82540EM
	$(VBM) modifyvm Redox --cableconnected1 on
	$(VBM) modifyvm Redox --nictrace1 on
	$(VBM) modifyvm Redox --nictracefile1 $(KBUILD)/network.pcap
	$(VBM) modifyvm Redox --uart1 0x3F8 4
	$(VBM) modifyvm Redox --uartmode1 file $(KBUILD)/serial.log
	$(VBM) modifyvm Redox --usb off # on
	$(VBM) modifyvm Redox --keyboard ps2
	$(VBM) modifyvm Redox --mouse ps2
	$(VBM) modifyvm Redox --audio $(VB_AUDIO)
	$(VBM) modifyvm Redox --audiocontroller ac97
	echo "Create Disk"
	$(VBM) convertfromraw $< $(KBUILD)/harddrive.vdi
	echo "Attach Disk"
	$(VBM) storagectl Redox --name ATA --add sata --controller IntelAHCI --bootable on --portcount 1
	$(VBM) storageattach Redox --storagectl ATA --port 0 --device 0 --type hdd --medium $(KBUILD)/harddrive.vdi
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

$(KBUILD)/librustc_unicode.rlib: rust/src/librustc_unicode/lib.rs $(KBUILD)/libcore.rlib
	$(KRUSTC) $(KRUSTCFLAGS) -o $@ $<

$(KBUILD)/libcollections.rlib: rust/src/libcollections/lib.rs $(KBUILD)/libcore.rlib $(KBUILD)/liballoc.rlib $(KBUILD)/librustc_unicode.rlib
	$(KRUSTC) $(KRUSTCFLAGS) -o $@ $<

$(KBUILD)/libkernel.a: kernel/** $(KBUILD)/libcore.rlib $(KBUILD)/liballoc.rlib $(KBUILD)/libcollections.rlib $(BUILD)/initfs.rs
	$(KCARGO) rustc $(KCARGOFLAGS) -C lto -o $@

$(KBUILD)/kernel: $(KBUILD)/libkernel.a
	$(LD) $(LDFLAGS) -z max-page-size=0x1000 -T arch/$(ARCH)/src/linker.ld -o $@ $<

# Userspace recipes
$(BUILD)/libcore.rlib: rust/src/libcore/lib.rs
	mkdir -p $(BUILD)
	$(RUSTC) $(RUSTCFLAGS) -o $@ $<

$(BUILD)/liballoc.rlib: rust/src/liballoc/lib.rs $(BUILD)/libcore.rlib
	$(RUSTC) $(RUSTCFLAGS) -o $@ $<

$(BUILD)/libcollections.rlib: rust/src/libcollections/lib.rs $(BUILD)/libcore.rlib $(BUILD)/liballoc.rlib $(BUILD)/librustc_unicode.rlib
	$(RUSTC) $(RUSTCFLAGS) -o $@ $<

$(BUILD)/librand.rlib: rust/src/librand/lib.rs $(BUILD)/libcore.rlib
	$(RUSTC) $(RUSTCFLAGS) -o $@ $<

$(BUILD)/librustc_unicode.rlib: rust/src/librustc_unicode/lib.rs $(BUILD)/libcore.rlib
	$(RUSTC) $(RUSTCFLAGS) -o $@ $<

libstd/openlibm/libopenlibm.a:
	CROSSCC=$(CC) CFLAGS=-fno-stack-protector make -C libstd/openlibm libopenlibm.a

$(BUILD)/libopenlibm.a: libstd/openlibm/libopenlibm.a
	mkdir -p $(BUILD)
	cp $< $@

$(BUILD)/libstd.rlib: libstd/Cargo.toml libstd/src/** $(BUILD)/libcore.rlib $(BUILD)/liballoc.rlib $(BUILD)/librustc_unicode.rlib $(BUILD)/libcollections.rlib $(BUILD)/librand.rlib $(BUILD)/libopenlibm.a
	$(CARGO) rustc --verbose --manifest-path $< $(CARGOFLAGS) -o $@
	cp libstd/target/$(TARGET)/release/deps/*.rlib $(BUILD)

initfs/bin/%: drivers/%/Cargo.toml drivers/%/src/** $(BUILD)/libstd.rlib
	mkdir -p initfs/bin
	$(CARGO) rustc --manifest-path $< $(CARGOFLAGS) -o $@
	strip $@
	rm $@.d

initfs/bin/%: programs/%/Cargo.toml programs/%/src/** $(BUILD)/libstd.rlib
	mkdir -p initfs/bin
	$(CARGO) rustc --manifest-path $< $(CARGOFLAGS) -o $@
	strip $@
	rm $@.d

initfs/bin/%: schemes/%/Cargo.toml schemes/%/src/** $(BUILD)/libstd.rlib
	mkdir -p initfs/bin
	$(CARGO) rustc --manifest-path $< --bin $* $(CARGOFLAGS) -o $@
	strip $@
	rm $@.d

$(BUILD)/initfs.rs: \
		initfs/bin/init \
		initfs/bin/ahcid \
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

filesystem/bin/%: drivers/%/Cargo.toml drivers/%/src/** $(BUILD)/libstd.rlib
	mkdir -p filesystem/bin
	$(CARGO) rustc --manifest-path $< $(CARGOFLAGS) -o $@
	strip $@
	rm $@.d

filesystem/bin/%: programs/%/Cargo.toml programs/%/src/** $(BUILD)/libstd.rlib
	mkdir -p filesystem/bin
	$(CARGO) rustc --manifest-path $< --bin $* $(CARGOFLAGS) -o $@
	strip $@
	rm $@.d

filesystem/bin/sh: filesystem/bin/ion
	cp $< $@

filesystem/bin/%: programs/coreutils/Cargo.toml programs/coreutils/src/bin/%.rs $(BUILD)/libstd.rlib
	mkdir -p filesystem/bin
	$(CARGO) rustc --manifest-path $< --bin $* $(CARGOFLAGS) -o $@
	strip $@
	rm $@.d

filesystem/bin/%: programs/extrautils/Cargo.toml programs/extrautils/src/bin/%.rs $(BUILD)/libstd.rlib
	mkdir -p filesystem/bin
	$(CARGO) rustc --manifest-path $< --bin $* $(CARGOFLAGS) -o $@
	strip $@
	rm $@.d

filesystem/bin/%: programs/netutils/Cargo.toml programs/netutils/src/%/**.rs $(BUILD)/libstd.rlib
	mkdir -p filesystem/bin
	$(CARGO) rustc --manifest-path $< --bin $* $(CARGOFLAGS) -o $@
	strip $@
	rm $@.d

filesystem/bin/%: programs/orbutils/Cargo.toml programs/orbutils/src/%/**.rs $(BUILD)/libstd.rlib
	mkdir -p filesystem/bin
	$(CARGO) rustc --manifest-path $< --bin $* $(CARGOFLAGS) -o $@
	strip $@
	rm $@.d

filesystem/bin/%: programs/pkgutils/Cargo.toml programs/pkgutils/src/%/**.rs $(BUILD)/libstd.rlib
	mkdir -p filesystem/bin
	$(CARGO) rustc --manifest-path $< --bin $* $(CARGOFLAGS) -o $@
	strip $@
	rm $@.d


filesystem/bin/%: programs/userutils/Cargo.toml programs/userutils/src/bin/%.rs $(BUILD)/libstd.rlib
	mkdir -p filesystem/bin
	$(CARGO) rustc --manifest-path $< --bin $* $(CARGOFLAGS) -o $@
	strip $@
	rm $@.d

filesystem/bin/%: schemes/%/Cargo.toml schemes/%/src/** $(BUILD)/libstd.rlib
	mkdir -p filesystem/bin
	$(CARGO) rustc --manifest-path $< --bin $* $(CARGOFLAGS) -o $@
	strip $@
	rm $@.d

drivers: \
	filesystem/bin/e1000d \
	filesystem/bin/rtl8168d

coreutils: \
	filesystem/bin/basename \
	filesystem/bin/cat \
	filesystem/bin/clear \
	filesystem/bin/cp \
	filesystem/bin/cut \
	filesystem/bin/date \
	filesystem/bin/dd \
	filesystem/bin/du \
	filesystem/bin/echo \
	filesystem/bin/env \
	filesystem/bin/false \
	filesystem/bin/head \
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
	#filesystem/bin/free  filesystem/bin/shutdown filesystem/bin/test

extrautils: \
	filesystem/bin/calc \
	filesystem/bin/cksum \
	filesystem/bin/cur \
	filesystem/bin/grep \
	filesystem/bin/less \
	filesystem/bin/mdless \
	filesystem/bin/mtxt \
	filesystem/bin/rem \
	#filesystem/bin/dmesg filesystem/bin/info filesystem/bin/man filesystem/bin/watch

netutils: \
	filesystem/bin/dhcpd \
	filesystem/bin/dns \
	filesystem/bin/httpd \
	filesystem/bin/irc \
	filesystem/bin/nc \
	filesystem/bin/wget

orbutils: \
	filesystem/bin/calculator \
	filesystem/bin/character_map \
	filesystem/bin/editor \
	filesystem/bin/file_manager \
	filesystem/bin/launcher \
	filesystem/bin/orblogin \
	filesystem/bin/terminal \
	filesystem/bin/viewer

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
	filesystem/bin/ethernetd \
	filesystem/bin/example \
	filesystem/bin/ipd \
	filesystem/bin/orbital \
	filesystem/bin/ptyd \
	filesystem/bin/randd \
	filesystem/bin/tcpd \
	filesystem/bin/udpd

$(BUILD)/filesystem.bin: \
		drivers \
		coreutils \
		extrautils \
		netutils \
		orbutils \
		pkgutils \
		userutils \
		schemes \
		filesystem/bin/acid \
		filesystem/bin/ion \
		filesystem/bin/sh \
		filesystem/bin/smith \
		filesystem/bin/tar
	-$(FUMOUNT) $(BUILD)/filesystem/
	rm -rf $@ $(BUILD)/filesystem/
	echo exit | cargo run --manifest-path schemes/redoxfs/Cargo.toml --bin redoxfs-utility $@ 256
	mkdir -p $(BUILD)/filesystem/
	cargo build --manifest-path schemes/redoxfs/Cargo.toml --bin redoxfs-fuse --release
	schemes/redoxfs/target/release/redoxfs-fuse $@ $(BUILD)/filesystem/ &
	sleep 2
	pgrep redoxfs-fuse
	cp -RL filesystem/* $(BUILD)/filesystem/
	chmod -R uog+rX $(BUILD)/filesystem
	chmod -R uog-w $(BUILD)/filesystem
	chmod -R 555 $(BUILD)/filesystem/bin/
	chmod -R u+rwX $(BUILD)/filesystem/root
	chmod -R og-rwx $(BUILD)/filesystem/root
	chmod -R u+rwX $(BUILD)/filesystem/home/user
	chmod -R og-rwx $(BUILD)/filesystem/home/user
	chmod +s $(BUILD)/filesystem/bin/su
	chmod +s $(BUILD)/filesystem/bin/sudo
	mkdir $(BUILD)/filesystem/tmp
	chmod 1777 $(BUILD)/filesystem/tmp
	chown -R 0:0 $(BUILD)/filesystem
	chown -R 1000:1000 $(BUILD)/filesystem/home/user
	sync
	-$(FUMOUNT) $(BUILD)/filesystem/
	rm -rf $(BUILD)/filesystem/

mount: FORCE
	mkdir -p $(BUILD)/filesystem/
	cargo build --manifest-path schemes/redoxfs/Cargo.toml --bin redoxfs-fuse --release
	schemes/redoxfs/target/release/redoxfs-fuse $(KBUILD)/harddrive.bin $(BUILD)/filesystem/ &
	sleep 2
	pgrep redoxfs-fuse

unmount: FORCE
	sync
	-$(FUMOUNT) $(BUILD)/filesystem/
	rm -rf $(BUILD)/filesystem/

wireshark: FORCE
	wireshark $(KBUILD)/network.pcap
