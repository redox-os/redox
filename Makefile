RUSTC=rustc
RUSTCFLAGS=-C relocation-model=dynamic-no-pic -C no-stack-check \
	-O -Z no-landing-pads \
	-A dead-code \
	-W trivial-casts -W trivial-numeric-casts
LD=ld
AS=nasm
QEMU=qemu-system-i386

all: harddrive.bin

kernel.bin: src/kernel.rs
	$(RUSTC) $(RUSTCFLAGS) --target i686-unknown-linux-gnu --crate-type lib -o $<.o --emit obj $<
	$(LD) -m elf_i386 -o $@ -T src/kernel.ld $<.o
	rm $<.o

filesystem/%.bin: src/%.rs
	$(RUSTC) $(RUSTCFLAGS) --target i686-unknown-linux-gnu --crate-type lib -o $<.o --emit obj $<
	$(LD) -m elf_i386 -o $@ -T src/program.ld $<.o
	rm $<.o

filesystem/filesystem.asm: filesystem/test.bin
	ls filesystem |  awk '{printf("file %d,\"%s\"\n", NR, $$0)}' > $@

harddrive.bin: src/loader.asm filesystem/filesystem.asm kernel.bin
	$(AS) -f bin -o $@ -ifilesystem/ -isrc/ $<

run: harddrive.bin
	$(QEMU) -enable-kvm -serial mon:stdio -sdl -hda $<

clean:
	rm -f *.bin filesystem/*.bin filesystem/filesystem.asm
