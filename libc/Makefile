export PATH := build/prefix/bin:$(PATH)

test.bin: test.c program.ld
	i386-elf-redox-gcc -Os -static -T program.ld -o $@ $<

test.list: test.bin
	i386-elf-redox-objdump -C -M intel -d $< > $@

libc:
	./setup.sh

clean:
	rm -f *.bin *.list *.o
