.PHONY: help all docs apps schemes tests clean \
	qemu qemu_bare qemu_no_kvm qemu_tap \
	virtualbox virtualbox_tap \
	arping ping wireshark

#Modify fo different target support
ARCH?=i386
#ARCH?=x86_64

BUILD=build/$(ARCH)

RUSTC=RUST_BACKTRACE=1 rustc
RUSTCFLAGS=--target=$(ARCH)-unknown-redox.json \
	-C no-prepopulate-passes -C no-vectorize-loops -C no-vectorize-slp -C no-stack-check -C opt-level=2 \
	-Z no-landing-pads \
	-A dead_code -A deprecated \
	-L $(BUILD)
AS=nasm
AWK=awk
BASENAME=basename
CUT=cut
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
	AS=windows/nasm
	AWK=windows/awk
	BASENAME=windows/basename
	CUT=windows/cut
	FIND=windows/find
	MAKE=windows/make
	MKDIR=windows/mkdir
	OBJDUMP=windows/objdump
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
        RUSTCFLAGS += -C ar=$(ARCH)-elf-ar -C linker=$(ARCH)-elf-linker
		VB="/Applications/VirtualBox.app/Contents/MacOS/VirtualBox"
		VB_AUDIO="coreaudio"
		VBM="/Applications/VirtualBox.app/Contents/MacOS/VBoxManage"
	endif
endif

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
	@echo "    make qemu_no_kvm"
	@echo "        Build Redox and run it inside Qemu machine without KVM support."
	@echo
	@echo "    make apps"
	@echo "        Build apps for Redox."
	@echo
	@echo "    make schemes"
	@echo "        Build schemes for Redox."
	@echo
	@echo "    make tests"
	@echo "        Run tests on Redox."
	@echo
	@echo "    make clean"
	@echo "        Clean build directory."
	@echo
	@echo
	@echo " .........................................................."
	@echo " For more information check out 'github.com/redox-os/redox'"
	@echo " or 'redox-os.org'"

all: $(BUILD)/harddrive.bin

docs: kernel/kernel.rs $(BUILD)/libcore.rlib $(BUILD)/liballoc.rlib
	rustdoc --target=$(ARCH)-unknown-redox.json -L$(BUILD) $<

apps: apps/editor apps/file_manager apps/orbital apps/player apps/sodium apps/terminal apps/test apps/viewer apps/zfs

schemes: schemes/console schemes/tcp schemes/udp schemes/window schemes/zfs

tests: tests/success tests/failure

clean:
	$(RM) -rf build filesystem/*.bin filesystem/*.list filesystem/apps/*/*.bin filesystem/apps/*/*.list filesystem/schemes/*/*.bin filesystem/schemes/*/*.list

apps/%:
	@$(MAKE) --no-print-directory filesystem/apps/$*/$*.bin

schemes/%:
	@$(MAKE) --no-print-directory filesystem/schemes/$*/$*.bin

FORCE:

tests/%: FORCE
	@$(SHELL) $@ && echo "$*: PASSED" || echo "$*: FAILED"

$(BUILD)/libcore.rlib: rust/libcore/lib.rs
	$(MKDIR) -p $(BUILD)
	$(RUSTC) $(RUSTCFLAGS) -o $@ $<

$(BUILD)/liballoc_system.rlib: rust/liballoc_system/lib.rs $(BUILD)/libcore.rlib
	$(RUSTC) $(RUSTCFLAGS) -o $@ $<

$(BUILD)/liballoc.rlib: rust/liballoc/lib.rs $(BUILD)/libcore.rlib $(BUILD)/liballoc_system.rlib
	$(RUSTC) $(RUSTCFLAGS) -o $@ $<

$(BUILD)/librustc_unicode.rlib: rust/librustc_unicode/lib.rs $(BUILD)/libcore.rlib
	$(RUSTC) $(RUSTCFLAGS) -o $@ $<

$(BUILD)/libcollections.rlib: rust/libcollections/lib.rs $(BUILD)/libcore.rlib $(BUILD)/liballoc.rlib $(BUILD)/librustc_unicode.rlib
	$(RUSTC) $(RUSTCFLAGS) -o $@ $<

$(BUILD)/librand.rlib: rust/librand/lib.rs $(BUILD)/libcore.rlib $(BUILD)/liballoc.rlib $(BUILD)/librustc_unicode.rlib $(BUILD)/libcollections.rlib
	$(RUSTC) $(RUSTCFLAGS) -o $@ $<

$(BUILD)/libredox.rlib: libredox/src/lib.rs $(BUILD)/libcore.rlib $(BUILD)/liballoc.rlib $(BUILD)/libcollections.rlib $(BUILD)/librand.rlib
	$(RUSTC) $(RUSTCFLAGS) --crate-name redox -o $@ $<

$(BUILD)/kernel.rlib: kernel/kernel.rs $(BUILD)/libcore.rlib $(BUILD)/liballoc.rlib $(BUILD)/libcollections.rlib
	$(RUSTC) $(RUSTCFLAGS) -C lto -o $@ $<

$(BUILD)/kernel.ir: kernel/kernel.rs $(BUILD)/libcore.rlib $(BUILD)/liballoc.rlib $(BUILD)/libcollections.rlib
	$(RUSTC) $(RUSTCFLAGS) -C lto -o $@ --emit llvm-ir $<

$(BUILD)/kernel.bin: $(BUILD)/kernel.rlib kernel/kernel.ld
	$(LD) $(LDARGS) -o $@ -T kernel/kernel.ld $<

$(BUILD)/kernel.list: $(BUILD)/kernel.bin
	$(OBJDUMP) -C -M intel -D $< > $@

$(BUILD)/crt0.o: kernel/program-$(ARCH).asm
ifeq ($(ARCH),x86_64)
	$(AS) -f elf64 $< -o $@
else
	$(AS) -f elf $< -o $@
endif

filesystem/apps/%.bin: filesystem/apps/%.rs kernel/program.rs kernel/program.ld $(BUILD)/crt0.o $(BUILD)/libcore.rlib $(BUILD)/liballoc.rlib $(BUILD)/libredox.rlib
	$(SED) "s|APPLICATION_PATH|../../$<|" kernel/program.rs > $(BUILD)/`$(BASENAME) $*`.gen
	$(RUSTC) $(RUSTCFLAGS) -C lto -o $(BUILD)/`$(BASENAME) $*`.rlib $(BUILD)/`$(BASENAME) $*`.gen
	$(LD) $(LDARGS) -o $@ -T kernel/program.ld $(BUILD)/crt0.o $(BUILD)/`$(BASENAME) $*`.rlib

filesystem/schemes/%.bin: filesystem/schemes/%.rs kernel/scheme.rs kernel/scheme.ld $(BUILD)/libredox.rlib
	$(SED) "s|SCHEME_PATH|../../$<|" kernel/scheme.rs > $(BUILD)/`$(BASENAME) $*`.gen
	$(RUSTC) $(RUSTCFLAGS) -C lto -o $(BUILD)/`$(BASENAME) $*`.rlib $(BUILD)/`$(BASENAME) $*`.gen
	$(LD) $(LDARGS) -o $@ -T kernel/scheme.ld $(BUILD)/`$(BASENAME) $*`.rlib $(BUILD)/libredox.rlib

filesystem/%.list: filesystem/%.bin
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

$(BUILD)/filesystem.gen: apps schemes
	$(FIND) filesystem -not -path '*/\.*' -type f -o -type l | $(CUT) -d '/' -f2- | $(SORT) | $(AWK) '{printf("file %d,\"%s\"\n", NR, $$0)}' > $@

$(BUILD)/harddrive.bin: kernel/loader-$(ARCH).asm $(BUILD)/kernel.bin $(BUILD)/filesystem.gen
	$(AS) -f bin -o $@ -i$(BUILD)/ -ikernel/ -ifilesystem/ $<

$(BUILD)/harddrive.list: kernel/loader-$(ARCH).asm $(BUILD)/kernel.bin $(BUILD)/filesystem.gen
	$(AS) -f bin -o $(BUILD)/harddrive.bin -l $@ -i$(BUILD)/ -ikernel/ -ifilesystem/ $<

virtualbox: $(BUILD)/harddrive.bin
	echo "Delete VM"
	-$(VBM) unregistervm Redox --delete; $(VBM_CLEANUP)
	echo "Delete Disk"
	-$(RM) harddrive.vdi
	echo "Create VM"
	$(VBM) createvm --name Redox --register
	echo "Set Configuration"
	$(VBM) modifyvm Redox --memory 1024
	$(VBM) modifyvm Redox --vram 64
	$(VBM) modifyvm Redox --nic1 nat
	$(VBM) modifyvm Redox --nictype1 82540EM
	$(VBM) modifyvm Redox --nictrace1 on
	$(VBM) modifyvm Redox --nictracefile1 $(BUILD)/network.pcap
	$(VBM) modifyvm Redox --uart1 0x3F8 4
	$(VBM) modifyvm Redox --uartmode1 file $(BUILD)/serial.log
	$(VBM) modifyvm Redox --usb on
	$(VBM) modifyvm Redox --audio $(VB_AUDIO)
	$(VBM) modifyvm Redox --audiocontroller ac97
	echo "Create Disk"
	$(VBM) convertfromraw $< $(BUILD)/harddrive.vdi
	echo "Attach Disk"
	$(VBM) storagectl Redox --name IDE --add ide --controller PIIX4 --bootable on
	$(VBM) storageattach Redox --storagectl IDE --port 0 --device 0 --type hdd --medium $(BUILD)/harddrive.vdi
	echo "Run VM"
	$(VB) --startvm Redox --dbg

qemu: $(BUILD)/harddrive.bin
	-qemu-system-$(ARCH) -net nic,model=rtl8139 -net user -net dump,file=$(BUILD)/network.pcap \
			-usb -device usb-tablet \
			-device usb-ehci,id=ehci -device nec-usb-xhci,id=xhci \
			-soundhw ac97 \
			-serial mon:stdio -m 1024 -d guest_errors -enable-kvm -hda $<

qemu_bare: $(BUILD)/harddrive.bin
	-qemu-system-$(ARCH) -net none -serial mon:stdio -m 1024 -d guest_errors -enable-kvm -hda $<

qemu_no_kvm: $(BUILD)/harddrive.bin
	-qemu-system-$(ARCH) -net nic,model=rtl8139 -net user -net dump,file=$(BUILD)/network.pcap \
			-usb -device usb-tablet \
			-device usb-ehci,id=ehci -device nec-usb-xhci,id=xhci \
			-soundhw ac97 \
			-serial mon:stdio -m 1024 -d guest_errors -hda $<

qemu_tap: $(BUILD)/harddrive.bin
	sudo tunctl -t tap_redox -u "${USER}"
	sudo ifconfig tap_redox 10.85.85.1 up
	-qemu-system-$(ARCH) -net nic,model=rtl8139 -net tap,ifname=tap_redox,script=no,downscript=no -net dump,file=$(BUILD)/network.pcap \
			-usb -device usb-tablet \
			-device usb-ehci,id=ehci -device nec-usb-xhci,id=xhci \
			-soundhw ac97 \
			-serial mon:stdio -m 1024 -d guest_errors -enable-kvm -hda $<
	sudo ifconfig tap_redox down
	sudo tunctl -d tap_redox

qemu_tap_8254x: $(BUILD)/harddrive.bin
	sudo tunctl -t tap_redox -u "${USER}"
	sudo ifconfig tap_redox 10.85.85.1 up
	-qemu-system-$(ARCH) -net nic,model=e1000 -net tap,ifname=tap_redox,script=no,downscript=no -net dump,file=$(BUILD)/network.pcap \
			-usb -device usb-tablet \
			-device usb-ehci,id=ehci -device nec-usb-xhci,id=xhci \
			-soundhw ac97 \
			-serial mon:stdio -m 1024 -d guest_errors -enable-kvm -hda $<
	sudo ifconfig tap_redox down
	sudo tunctl -d tap_redox

virtualbox_tap: $(BUILD)/harddrive.bin
	echo "Delete VM"
	-$(VBM) unregistervm Redox --delete; $(VBM_CLEANUP)
	echo "Delete Disk"
	-$(RM) harddrive.vdi
	echo "Create VM"
	$(VBM) createvm --name Redox --register
	echo "Create Bridge"
	sudo tunctl -t tap_redox -u "${USER}"
	sudo ifconfig tap_redox 10.85.85.1 up
	echo "Set Configuration"
	$(VBM) modifyvm Redox --memory 1024
	$(VBM) modifyvm Redox --vram 64
	$(VBM) modifyvm Redox --nic1 bridged
	$(VBM) modifyvm Redox --nictype1 82540EM
	$(VBM) modifyvm Redox --nictrace1 on
	$(VBM) modifyvm Redox --nictracefile1 network.pcap
	$(VBM) modifyvm Redox --bridgeadapter1 tap_redox
	$(VBM) modifyvm Redox --uart1 0x3F8 4
	$(VBM) modifyvm Redox --uartmode1 file $(BUILD)/serial.log
	$(VBM) modifyvm Redox --usb on
	$(VBM) modifyvm Redox --audio $(VB_AUDIO)
	$(VBM) modifyvm Redox --audiocontroller ac97
	echo "Create Disk"
	$(VBM) convertfromraw $< $(BUILD)/harddrive.vdi
	echo "Attach Disk"
	$(VBM) storagectl Redox --name IDE --add ide --controller PIIX4 --bootable on
	$(VBM) storageattach Redox --storagectl IDE --port 0 --device 0 --type hdd --medium $(BUILD)/harddrive.vdi
	echo "Run VM"
	-$(VB) --startvm Redox --dbg
	echo "Delete Bridge"
	sudo ifconfig tap_redox down
	sudo tunctl -d tap_redox

arping:
	arping -I tap_redox 10.85.85.2

ping:
	ping 10.85.85.2

wireshark:
	wireshark network.pcap

