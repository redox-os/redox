ARCH?=x86_64

run: qemu

bochs: build/harddrive.bin
	bochs -f bochs.$(ARCH)

qemu: build/harddrive.bin
	qemu-system-$(ARCH) -drive file=$<,format=raw,index=0,media=disk

FORCE:

build/libkernel.a: FORCE
	rustc --crate-type staticlib src/lib.rs -o $@
	#--target $(ARCH)-unknown-none.json

build/kernel.bin: build/libkernel.a
	ld -m elf_$(ARCH) -o $@ -T bootloader/x86/kernel.ld -z max-page-size=0x1000 $<

build/harddrive.bin: build/kernel.bin
	nasm -f bin -o $@ -D ARCH_$(ARCH) -ibootloader/x86/ -ibuild/ bootloader/x86/harddrive.asm

clean:
	rm -rf build/*
