RUSTC=rustc
RUSTCFLAGS=--target i686-unknown-linux-gnu \
	-C target-feature=-mmx,-sse,-sse2,-sse3,-ssse3,-sse4.1,-sse4.2,-3dnow,-3dnowa,-avx,-avx2 \
	-C no-vectorize-loops -C no-vectorize-slp -C relocation-model=static -C code-model=kernel -C no-stack-check -C opt-level=2 \
	-Z no-landing-pads \
	-A dead-code -A deprecated \
	-L .
AS=nasm
AWK=awk
CUT=cut
FIND=find
LD=ld
LDARGS=-m elf_i386
RM=rm -f
SED=sed
SORT=sort
VB=virtualbox
VBM=VBoxManage

ifeq ($(OS),Windows_NT)
	SHELL=windows\sh
	LD=windows/i386-elf-ld
	AS=windows/nasm
	AWK=windows/awk
	CUT=windows/cut
	FIND=windows/find
	RM=windows/rm -f
	SED=windows/sed
	SORT=windows/sort
	VB="C:/Program Files/Oracle/VirtualBox/VirtualBox"
	VBM="C:/Program Files/Oracle/VirtualBox/VBoxManage"
else
	UNAME := $(shell uname)
	ifeq ($(UNAME),Darwin)
	    LD=i386-elf-ld
			VB="/Applications/VirtualBox.app/Contents/MacOS/VirtualBox"
			VBM="/Applications/VirtualBox.app/Contents/MacOS/VBoxManage"
	endif
endif

all: harddrive.bin

doc: src/kernel.rs libcore.rlib liballoc.rlib
	rustdoc --target i686-unknown-linux-gnu $< --extern core=libcore.rlib --extern alloc=liballoc.rlib

liballoc.rlib: rust/liballoc/lib.rs libcore.rlib
	$(RUSTC) $(RUSTCFLAGS) --crate-type rlib -o $@ $< --extern core=libcore.rlib

libcore.rlib: rust/libcore/lib.rs
	$(RUSTC) $(RUSTCFLAGS) --cfg stage0 --crate-type rlib -o $@ $<

#libcollections.rlib: src/libcollections/lib.rs liballoc.rlib
#	$(RUSTC) $(RUSTCFLAGS) --crate-type rlib -o $@ $< --extern alloc=liballoc.rlib

kernel.rlib: src/kernel.rs libcore.rlib liballoc.rlib
	$(RUSTC) $(RUSTCFLAGS) --crate-type rlib -o $@ $< --extern core=libcore.rlib --extern alloc=liballoc.rlib

kernel.bin: kernel.rlib libcore.rlib liballoc.rlib
	$(LD) $(LDARGS) -o $@ -T src/kernel.ld $< libcore.rlib liballoc.rlib

kernel.list: kernel.bin
	objdump -C -M intel -d $< > $@

filesystem/%.bin: filesystem/%.rs src/program.rs src/program.ld libcore.rlib liballoc.rlib
	$(SED) "s|APPLICATION_PATH|$<|" src/program.rs > $*.gen
	$(RUSTC) $(RUSTCFLAGS) --crate-type rlib -o $*.rlib $*.gen --extern core=libcore.rlib --extern alloc=liballoc.rlib
	$(LD) $(LDARGS) -o $@ -T src/program.ld $*.rlib libcore.rlib liballoc.rlib

filesystem.gen: filesystem/httpd.bin filesystem/terminal.bin
	$(FIND) filesystem -not -path '*/\.*' -type f -o -type l | $(CUT) -d '/' -f2- | $(SORT) | $(AWK) '{printf("file %d,\"%s\"\n", NR, $$0)}' > $@

harddrive.bin: src/loader.asm kernel.bin filesystem.gen
	$(AS) -f bin -o $@ -isrc/ -ifilesystem/ $<

virtualbox: harddrive.bin
	echo "Delete VM"
	-$(VBM) unregistervm Redox --delete
	echo "Create VM"
	$(VBM) createvm --name Redox --register
	echo "Set Configuration"
	$(VBM) modifyvm Redox --memory 512
	$(VBM) modifyvm Redox --vram 64
	$(VBM) modifyvm Redox --nic1 nat
	$(VBM) modifyvm Redox --nictype1 82540EM
	$(VBM) modifyvm Redox --nictrace1 on
	$(VBM) modifyvm Redox --nictracefile1 network.pcap
	$(VBM) modifyvm Redox --uart1 0x3F8 4
	$(VBM) modifyvm Redox --uartmode1 file serial.log
	$(VBM) modifyvm Redox --usb on
	echo "Create Disk"
	$(VBM) convertfromraw $< harddrive.vdi
	echo "Attach Disk"
	$(VBM) storagectl Redox --name IDE --add ide --controller PIIX4 --bootable on
	$(VBM) storageattach Redox --storagectl IDE --port 0 --device 0 --type hdd --medium harddrive.vdi
	echo "Run VM"
	$(VB) --startvm Redox --dbg

qemu: harddrive.bin
	-qemu-system-i386 -net nic,model=rtl8139 -net user -net dump,file=network.pcap \
			-usb -device usb-ehci,id=ehci -device usb-tablet,bus=ehci.0 \
			-serial mon:stdio -enable-kvm -hda $<
			#-device nec-usb-xhci,id=xhci -device usb-tablet,bus=xhci.0

qemu_tap: harddrive.bin
	sudo tunctl -t tap_redox -u "${USER}"
	sudo ifconfig tap_redox 10.85.85.1 up
	-qemu-system-i386 -net nic,model=rtl8139 -net tap,ifname=tap_redox,script=no,downscript=no -net dump,file=network.pcap \
			-usb -device usb-ehci,id=ehci -device usb-tablet,bus=ehci.0 \
			-serial mon:stdio -enable-kvm -hda $<
			#-device nec-usb-xhci,id=xhci -device usb-tablet,bus=xhci.0
	sudo ifconfig tap_redox down
	sudo tunctl -d tap_redox

qemu_tap_8254x: harddrive.bin
	sudo tunctl -t tap_redox -u "${USER}"
	sudo ifconfig tap_redox 10.85.85.1 up
	-qemu-system-i386 -net nic,model=e1000 -net tap,ifname=tap_redox,script=no,downscript=no -net dump,file=network.pcap \
			-usb -device usb-ehci,id=ehci -device usb-tablet,bus=ehci.0 \
			-serial mon:stdio -enable-kvm -hda $<
			#-device nec-usb-xhci,id=xhci -device usb-tablet,bus=xhci.0
	sudo ifconfig tap_redox down
	sudo tunctl -d tap_redox

virtualbox_tap: harddrive.bin
	echo "Delete VM"
	-$(VBM) unregistervm Redox --delete
	echo "Create VM"
	$(VBM) createvm --name Redox --register
	echo "Create Bridge"
	sudo tunctl -t tap_redox -u "${USER}"
	sudo ifconfig tap_redox 10.85.85.1 up
	echo "Set Configuration"
	$(VBM) modifyvm Redox --memory 512
	$(VBM) modifyvm Redox --vram 64
	$(VBM) modifyvm Redox --nic1 bridged
	$(VBM) modifyvm Redox --nictype1 82540EM
	$(VBM) modifyvm Redox --nictrace1 on
	$(VBM) modifyvm Redox --nictracefile1 network.pcap
	$(VBM) modifyvm Redox --bridgeadapter1 tap_redox
	$(VBM) modifyvm Redox --uart1 0x3F8 4
	$(VBM) modifyvm Redox --uartmode1 file serial.log
	$(VBM) modifyvm Redox --usb on
	echo "Create Disk"
	$(VBM) convertfromraw $< harddrive.vdi
	echo "Attach Disk"
	$(VBM) storagectl Redox --name IDE --add ide --controller PIIX4 --bootable on
	$(VBM) storageattach Redox --storagectl IDE --port 0 --device 0 --type hdd --medium harddrive.vdi
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

clean:
	$(RM) *.bin *.gen *.list *.log *.pcap *.rlib *.vdi filesystem/*.bin
