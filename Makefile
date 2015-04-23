RUSTC=rustc
RUSTCFLAGS=-C relocation-model=dynamic-no-pic -C no-stack-check \
	-O -Z no-landing-pads \
	-A dead-code \
	-W trivial-casts -W trivial-numeric-casts
LD=ld
AS=nasm
QEMU=qemu-system-i386

all: harddrive.bin

kernel.o: src/kernel.rs
	$(RUSTC) $(RUSTCFLAGS) --target i686-unknown-linux-gnu --crate-type lib -o $@ --emit obj $<

kernel.bin: src/kernel.ld kernel.o
	$(LD) -m elf_i386 -o $@ -T $^

test.o: src/test.rs
	$(RUSTC) $(RUSTCFLAGS) --target i686-unknown-linux-gnu --crate-type lib -o $@ --emit obj $<

filesystem/test.bin: src/program.ld test.o
	$(LD) -m elf_i386 -o $@ -T $^

filesystem/filesystem.asm: filesystem/test.bin
	ls filesystem |  awk '{printf("file %d,\"%s\"\n", NR, $$0)}' > filesystem/filesystem.asm

harddrive.bin: src/loader.asm filesystem/filesystem.asm kernel.bin
	$(AS) -f bin -o $@ -ifilesystem/ -isrc/ $<

run: harddrive.bin
	$(QEMU) -serial mon:stdio -sdl -hda $<

clean:
	rm -f *.bin *.o filesystem/test.bin filesystem/filesystem.asm
