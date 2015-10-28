export PATH := build/prefix/bin:$(PATH)

all: sdl.bin test.bin

test.bin: test.c program.ld
	i386-elf-redox-gcc -Os -static -T program.ld -o $@ $<

test.list: test.bin
	i386-elf-redox-objdump -C -M intel -d $< > $@

sdl.bin: sdl.c program.ld
	i386-elf-redox-gcc -Os -static -T program.ld -o $@ $< -lSDL_ttf -lSDL_image -lSDL -lfreetype -lpng -lz -lm

libc:
	./libc.sh

clean:
	rm -f *.bin *.list *.o
