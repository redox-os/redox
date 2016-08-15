ARCH?=x86_64

all: build/harddrive.bin

list: build/kernel.list

run: bochs

bochs: build/harddrive.bin
	bochs -f bochs.$(ARCH)

qemu: build/harddrive.bin
	qemu-system-$(ARCH) -serial mon:stdio -drive file=$<,format=raw,index=0,media=disk

FORCE:

build:
	mkdir build

build/libkernel.a: build FORCE
	cargo rustc -- --crate-type staticlib -o $@
	#--target $(ARCH)-unknown-none.json

build/kernel.bin: build/libkernel.a
	ld -m elf_$(ARCH) --gc-sections -z max-page-size=0x1000 -T bootloader/x86/kernel.ld -o $@ $<

build/kernel.list: build/kernel.bin
	objdump -C -M intel -D $< > $@

build/harddrive.bin: build/kernel.bin
	nasm -f bin -o $@ -D ARCH_$(ARCH) -ibootloader/x86/ -ibuild/ bootloader/x86/harddrive.asm

clean:
	rm -rf build/*
