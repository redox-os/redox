export PATH := build/prefix/bin:$(PATH)

test.bin: test.c program.ld
	i386-elf-redox-gcc -Os -static -T program.ld -o $@ $<

test.list: test.bin
	i386-elf-redox-objdump -C -M intel -d $< > $@

sdl.bin: sdl.c program.ld SDL-1.2.15/build/.libs/libSDL.a
	i386-elf-redox-gcc -Os -static -T program.ld -I SDL-1.2.15/include/ -o $@ $< SDL-1.2.15/build/.libs/libSDL.a

libc:
	./setup.sh

lua:
	./lua.sh

SDL-1.2.15/build/.libs/libSDL.a:
	./libsdl.sh

clean:
	rm -f *.bin *.list *.o
