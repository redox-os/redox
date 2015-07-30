RUSTC=rustc
RUSTCFLAGS=-C relocation-model=dynamic-no-pic -C no-stack-check \
	-O -Z no-landing-pads \
	-A dead-code \
	-W trivial-casts -W trivial-numeric-casts --cfg debug_network
LD=ld
AS=nasm
QEMU=qemu-system-i386
QEMU_FLAGS=-serial mon:stdio -net nic,model=rtl8139 -net user
#-usb -device nec-usb-xhci,id=xhci -device usb-tablet,bus=xhci.0

all: harddrive.bin

kernel.bin: src/kernel.rs
	$(RUSTC) $(RUSTCFLAGS) --target i686-unknown-linux-gnu --crate-type lib -o kernel.o --emit obj $<
	$(LD) -m elf_i386 -o $@ -T src/kernel.ld kernel.o

filesystem/example.bin: filesystem/example.rs
	$(RUSTC) $(RUSTCFLAGS) --target i686-unknown-linux-gnu --crate-type lib -o example.o --emit obj $<
	$(LD) -m elf_i386 -o $@ -T src/program.ld example.o

filesystem/filesystem.asm: filesystem/example.bin
	find filesystem -type f -o -type l | cut -d '/' -f2- | grep -v filesystem.asm | sort | awk '{printf("file %d,\"%s\"\n", NR, $$0)}' > $@

harddrive.bin: src/loader.asm filesystem/filesystem.asm kernel.bin
	$(AS) -f bin -o $@ -ifilesystem/ -isrc/ $<

run: harddrive.bin
	$(QEMU) $(QEMU_FLAGS) -enable-kvm -sdl -hda $<

run_no_kvm: harddrive.bin
	$(QEMU) $(QEMU_FLAGS) -sdl -hda $<

run_tap: harddrive.bin
	sudo tunctl -t tap_qemu -u "${USER}"
	sudo ifconfig tap_qemu 10.85.85.1 up
	$(QEMU) $(QEMU_FLAGS) -enable-kvm -sdl  -net tap,ifname=tap_qemu,script=no,downscript=no -hda $<
	sudo ifconfig tap_qemu down
	sudo tunctl -d tap_qemu

run_tap_dump: harddrive.bin
	sudo tunctl -t tap_qemu -u "${USER}"
	sudo ifconfig tap_qemu 10.85.85.1 up
	$(QEMU) $(QEMU_FLAGS) -enable-kvm -sdl -net dump,file=network.pcap -net tap,ifname=tap_qemu,script=no,downscript=no -hda $<
	sudo ifconfig tap_qemu down
	sudo tunctl -d tap_qemu

clean:
	rm -f *.bin *.o filesystem/*.bin filesystem/filesystem.asm
