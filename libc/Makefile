export PATH := build/prefix/bin:$(PATH)

test.bin: test.c program.ld
	i386-elf-redox-gcc -Os -static -T program.ld -o $@ $<

test.list: test.bin
	i386-elf-redox-objdump -C -M intel -d $< > $@

sdl.bin: sdl.c program.ld SDL-1.2.15/build/.libs/libSDL.a SDL_image-1.2.12/.libs/libSDL_image.a
	i386-elf-redox-gcc -Os -static -T program.ld -I SDL-1.2.15/include/ -o $@ $< SDL-1.2.15/build/.libs/libSDL.a SDL_image-1.2.12/.libs/libSDL_image.a

libc:
	./libc.sh

lua:
	./lua.sh

SDL-1.2.15/build/.libs/libSDL.a:
	./sdl.sh

SDL_image-1.2.12/.libs/libSDL_image.a:
	./sdl_image.sh

clean:
	rm -f *.bin *.list *.o
