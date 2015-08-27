RUSTC=rustc
RUSTCFLAGS=-C relocation-model=static -C code-model=kernel -C no-stack-check -C opt-level=2 \
	-Z no-landing-pads \
	-W dead-code -W trivial-casts -W trivial-numeric-casts \
	-L .
#--cfg debug_network
LD=ld
AS=nasm
QEMU=qemu-system-i386
QEMU_FLAGS=-serial mon:stdio -net nic,model=rtl8139 -usb -device usb-ehci,id=ehci -device usb-tablet,bus=ehci.0 -drive if=none,id=usb_drive,file=harddrive.bin -device usb-storage,bus=ehci.0,drive=usb_drive -d cpu_reset
#-usb -device nec-usb-xhci,id=xhci -device usb-tablet,bus=xhci.0

all: harddrive.bin

kernel.list: kernel.bin
	objdump -C -M intel -d $< > $@

doc: src/kernel.rs
	rustdoc --target i686-unknown-linux-gnu $< --extern alloc=liballoc.rlib --extern mopa=libmopa.rlib

liballoc.rlib: src/liballoc/lib.rs
	$(RUSTC) $(RUSTCFLAGS) --target i686-unknown-linux-gnu --crate-type rlib -o $@ $<

#libcollections.rlib: src/libcollections/lib.rs liballoc.rlib
#	$(RUSTC) $(RUSTCFLAGS) --target i686-unknown-linux-gnu --crate-type rlib -o $@ $< --extern alloc=liballoc.rlib

libmopa.rlib: src/libmopa/lib.rs
	$(RUSTC) $(RUSTCFLAGS) --target i686-unknown-linux-gnu --crate-type rlib -o $@ $< --cfg 'feature = "no_std"'

kernel.rlib: src/kernel.rs liballoc.rlib libmopa.rlib
	$(RUSTC) $(RUSTCFLAGS) --target i686-unknown-linux-gnu --crate-type rlib -o $@ $< --extern alloc=liballoc.rlib --extern mopa=libmopa.rlib

kernel.bin: kernel.rlib liballoc.rlib libmopa.rlib
	$(LD) -m elf_i386 -o $@ -T src/kernel.ld $< liballoc.rlib libmopa.rlib

terminal.rlib: src/program.rs filesystem/terminal.rs liballoc.rlib libmopa.rlib
	sed 's|APPLICATION_PATH|../filesystem/terminal.rs|' src/program.rs > src/program.gen
	$(RUSTC) $(RUSTCFLAGS) --target i686-unknown-linux-gnu --crate-type rlib -o $@ src/program.gen --extern alloc=liballoc.rlib --extern mopa=libmopa.rlib

filesystem/terminal.bin: terminal.rlib liballoc.rlib libmopa.rlib
	$(LD) -m elf_i386 -o $@ -T src/program.ld $< liballoc.rlib libmopa.rlib

src/filesystem.gen: filesystem/terminal.bin
	find filesystem -type f -o -type l | cut -d '/' -f2- | sort | awk '{printf("file %d,\"%s\"\n", NR, $$0)}' > $@

harddrive.bin: src/loader.asm kernel.bin src/filesystem.gen
	$(AS) -f bin -o $@ -isrc/ -ifilesystem/ $<

run: harddrive.bin
	$(QEMU) $(QEMU_FLAGS) -enable-kvm -sdl -net user -hda $<

run_gdb: harddrive.bin
	$(QEMU) $(QEMU_FLAGS) -enable-kvm -s -S -sdl -net user -hda $<

run_no_kvm: harddrive.bin
	$(QEMU) $(QEMU_FLAGS) -sdl -net user -hda $<

run_tap: harddrive.bin
	sudo tunctl -t tap_qemu -u "${USER}"
	sudo ifconfig tap_qemu 10.85.85.1 up
	$(QEMU) $(QEMU_FLAGS) -enable-kvm -sdl -net tap,ifname=tap_qemu,script=no,downscript=no -hda $<
	sudo ifconfig tap_qemu down
	sudo tunctl -d tap_qemu

run_tap_dump: harddrive.bin
	sudo tunctl -t tap_qemu -u "${USER}"
	sudo ifconfig tap_qemu 10.85.85.1 up
	$(QEMU) $(QEMU_FLAGS) -enable-kvm -sdl -net dump,file=network.pcap -net tap,ifname=tap_qemu,script=no,downscript=no -hda $<
	sudo ifconfig tap_qemu down
	sudo tunctl -d tap_qemu

clean:
	rm -f *.bin *.list *.rlib filesystem/*.bin src/*.gen
