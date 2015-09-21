NEWLIB=newlib-2.2.0.20150824

test.bin: test.c ../src/program.ld
	gcc-4.6 -m32 -Os -nostartfiles -nostdlib -static \
		-T ../src/program.ld -Wl,--build-id=none \
		-o $@ "build-$(NEWLIB)/i386-elf-redox/newlib/crt0.o" $< \
		-I "$(NEWLIB)/newlib/include" -L "build-$(NEWLIB)/i386-elf-redox/newlib" -lc -lm

test.list: test.bin
	objdump -C -M intel -d $< > $@

libc:
	./setup.sh

clean:
	rm -f *.o *.bin
