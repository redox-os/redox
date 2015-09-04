RUSTC=rustc
RUSTCFLAGS=-C target-feature=-mmx,-sse,-sse2,-sse3,-ssse3,-sse4.1,-sse4.2,-3dnow,-3dnowa,-avx,-avx2 \
	-C no-vectorize-loops -C no-vectorize-slp -C relocation-model=static -C code-model=kernel -C no-stack-check -C opt-level=2 \
	-Z no-landing-pads \
	-A dead-code -W trivial-casts -W trivial-numeric-casts \
	-L .
AS=nasm
AWK=awk
CUT=cut
FIND=find
LD=ld
LDARGS=-m elf_i386
QEMU=qemu-system-i386
QEMU_FLAGS=-serial mon:stdio -net nic,model=rtl8139 -usb -device usb-ehci,id=ehci -device usb-tablet,bus=ehci.0 -drive if=none,id=usb_drive,file=harddrive.bin -device usb-storage,bus=ehci.0,drive=usb_drive
#-usb -device nec-usb-xhci,id=xhci -device usb-tablet,bus=xhci.0
RM=rm -f
SED=sed
SORT=sort

ifeq ($(OS),Windows_NT)
	SHELL=windows\sh
	LD=windows/i386-elf-ld
	AS=windows/nasm
	AWK=windows/awk
	CUT=windows/cut
	FIND=windows/find
	QEMU=windows/qemu/qemu-system-i386w
	QEMU_FLAGS=-L windows/qemu/Bios -net nic,model=rtl8139 -usb -device usb-ehci,id=ehci -device usb-tablet,bus=ehci.0
	RM=windows/rm -f
	SED=windows/sed
	SORT=windows/sort
else
	UNAME := $(shell uname)
	ifeq ($(UNAME),Darwin)
	    LD=i386-elf-ld
	endif
endif

all: harddrive.bin

kernel.list: kernel.bin
	objdump -C -M intel -d $< > $@

terminal.list: filesystem/terminal.bin
	objdump -C -M intel -d $< > $@

doc: src/kernel.rs libcore.rlib liballoc.rlib
	rustdoc --target i686-unknown-linux-gnu $< --extern core=libcore.rlib --extern alloc=liballoc.rlib

liballoc.rlib: src/liballoc/lib.rs libcore.rlib
	$(RUSTC) $(RUSTCFLAGS) --target i686-unknown-linux-gnu --crate-type rlib -o $@ $< --extern core=libcore.rlib

libcore.rlib: src/libcore/lib.rs
	$(RUSTC) $(RUSTCFLAGS) --cfg stage0 --target i686-unknown-linux-gnu --crate-type rlib -o $@ $<

#libcollections.rlib: src/libcollections/lib.rs liballoc.rlib
#	$(RUSTC) $(RUSTCFLAGS) --target i686-unknown-linux-gnu --crate-type rlib -o $@ $< --extern alloc=liballoc.rlib

#libmopa.rlib: src/libmopa/lib.rs
#	$(RUSTC) $(RUSTCFLAGS) --target i686-unknown-linux-gnu --crate-type rlib -o $@ $< --cfg 'feature = "no_std"'

kernel.rlib: src/kernel.rs libcore.rlib liballoc.rlib
	$(RUSTC) $(RUSTCFLAGS) --target i686-unknown-linux-gnu --crate-type rlib -o $@ $< --extern core=libcore.rlib --extern alloc=liballoc.rlib

kernel.bin: kernel.rlib libcore.rlib liballoc.rlib
	$(LD) $(LDARGS) -o $@ -T src/kernel.ld $< libcore.rlib liballoc.rlib

httpd.rlib: src/program.rs filesystem/httpd.rs libcore.rlib liballoc.rlib
	$(SED) "s|APPLICATION_PATH|../filesystem/httpd.rs|" src/program.rs > src/program.gen
	$(RUSTC) $(RUSTCFLAGS) --target i686-unknown-linux-gnu --crate-type rlib -o $@ src/program.gen --extern core=libcore.rlib --extern alloc=liballoc.rlib

filesystem/httpd.bin: httpd.rlib libcore.rlib liballoc.rlib
	$(LD) $(LDARGS) -o $@ -T src/program.ld $< libcore.rlib liballoc.rlib

terminal.rlib: src/program.rs filesystem/terminal.rs libcore.rlib liballoc.rlib
	$(SED) "s|APPLICATION_PATH|../filesystem/terminal.rs|" src/program.rs > src/program.gen
	$(RUSTC) $(RUSTCFLAGS) --target i686-unknown-linux-gnu --crate-type rlib -o $@ src/program.gen --extern core=libcore.rlib --extern alloc=liballoc.rlib

filesystem/terminal.bin: terminal.rlib libcore.rlib liballoc.rlib
	$(LD) $(LDARGS) -o $@ -T src/program.ld $< libcore.rlib liballoc.rlib

src/filesystem.gen: filesystem/httpd.bin filesystem/terminal.bin
	$(FIND) filesystem -type f -o -type l | $(CUT) -d '/' -f2- | $(SORT) | $(AWK) '{printf("file %d,\"%s\"\n", NR, $$0)}' > $@

harddrive.bin: src/loader.asm kernel.bin src/filesystem.gen
	$(AS) -f bin -o $@ -isrc/ -ifilesystem/ $<

run: harddrive.bin
	$(QEMU) $(QEMU_FLAGS) -enable-kvm -net user -hda $<

run_gdb: harddrive.bin
	$(QEMU) $(QEMU_FLAGS) -enable-kvm -s -S -net user -hda $<

run_no_kvm: harddrive.bin
	$(QEMU) $(QEMU_FLAGS) -net user -hda $<

run_tap: harddrive.bin
	sudo tunctl -t tap_qemu -u "${USER}"
	sudo ifconfig tap_qemu 10.85.85.1 up
	$(QEMU) $(QEMU_FLAGS) -enable-kvm -net tap,ifname=tap_qemu,script=no,downscript=no -hda $<
	sudo ifconfig tap_qemu down
	sudo tunctl -d tap_qemu

run_tap_dump: harddrive.bin
	sudo tunctl -t tap_qemu -u "${USER}"
	sudo ifconfig tap_qemu 10.85.85.1 up
	$(QEMU) $(QEMU_FLAGS) -enable-kvm -net dump,file=network.pcap -net tap,ifname=tap_qemu,script=no,downscript=no -hda $<
	sudo ifconfig tap_qemu down
	sudo tunctl -d tap_qemu

clean:
	$(RM) *.bin *.list *.rlib filesystem/*.bin src/*.gen
