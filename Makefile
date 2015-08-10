RUSTC=rustc
RUSTCFLAGS=-C relocation-model=dynamic-no-pic -C no-stack-check \
	-O -Z no-landing-pads \
	-A dead-code \
	-W trivial-casts -W trivial-numeric-casts \
	-L .
#--cfg debug_network
LD=ld
AS=nasm
QEMU=qemu-system-i386
QEMU_FLAGS=-serial mon:stdio -net nic,model=rtl8139
#-usb -device nec-usb-xhci,id=xhci -device usb-tablet,bus=xhci.0

all: harddrive.bin

libredox_alloc.rlib: src/alloc/lib.rs
	$(RUSTC) $(RUSTCFLAGS) --target i686-unknown-linux-gnu --crate-type rlib -o $@ $<

libmopa.rlib: src/mopa/lib.rs
	$(RUSTC) $(RUSTCFLAGS) --target i686-unknown-linux-gnu --crate-type rlib -o $@ $< --cfg 'feature = "no_std"'

kernel.rlib: src/kernel.rs libredox_alloc.rlib libmopa.rlib
	$(RUSTC) $(RUSTCFLAGS) --target i686-unknown-linux-gnu --crate-type rlib -o $@ $< --extern redox_alloc=libredox_alloc.rlib --extern mopa=libmopa.rlib

kernel.bin: kernel.rlib libredox_alloc.rlib
	$(LD) -m elf_i386 -o $@ -T src/kernel.ld $< libredox_alloc.rlib

example.rlib: src/program.rs filesystem/example.rs libredox_alloc.rlib libmopa.rlib
	sed 's|APPLICATION_PATH|../filesystem/example.rs|' src/program.rs > src/program.gen
	$(RUSTC) $(RUSTCFLAGS) --target i686-unknown-linux-gnu --crate-type rlib -o $@ src/program.gen --extern redox_alloc=libredox_alloc.rlib --extern mopa=libmopa.rlib

filesystem/example.bin: example.rlib libredox_alloc.rlib
	$(LD) -m elf_i386 -o $@ -T src/program.ld $< libredox_alloc.rlib

src/filesystem.gen: filesystem/example.bin
	find filesystem -type f -o -type l | cut -d '/' -f2- | sort | awk '{printf("file %d,\"%s\"\n", NR, $$0)}' > $@

harddrive.bin: src/loader.asm kernel.bin src/filesystem.gen
	$(AS) -f bin -o $@ -isrc/ -ifilesystem/ $<

run: harddrive.bin
	$(QEMU) $(QEMU_FLAGS) -enable-kvm -sdl -net user -hda $<

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
	rm -f *.bin *.o *.rlib filesystem/*.bin src/*.gen
