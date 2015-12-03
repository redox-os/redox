export PATH := build/prefix/bin:$(PATH)

all: sdl.bin sdl_ttf.bin test.bin

sdl.bin: sdl.c program.ld
	i386-elf-redox-gcc -Os -static -o $@ $< -lSDL

sdl_ttf.bin: sdl_ttf.c program.ld
	i386-elf-redox-gcc -Os -static -o $@ $< -lSDL_ttf -lSDL_image -lSDL -lfreetype -lpng -lz -lm

test.bin: test.c program.ld
	i386-elf-redox-gcc -Os -static -o $@ $<

install: all
	mkdir -p ../filesystem/libc/
	cp *.bin ../filesystem/libc/

libc:
	./libc.sh

clean:
	rm -f *.bin *.list *.o
