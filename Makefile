ARCH?=x86_64

LD=ld
QEMU=qemu-system-$(ARCH)
QEMUFLAGS=-enable-kvm -cpu host

UNAME := $(shell uname)
ifeq ($(UNAME),Darwin)
	LD=$(ARCH)-elf-ld
	QEMUFLAGS=
endif

QEMUFLAGS+=-smp 4 -machine q35
QEMUFLAGS+=-serial mon:stdio
QEMUFLAGS+=-d guest_errors
#,int,pcall
#-nographic
#-device intel-iommu


all: build/harddrive.bin

list: build/kernel.list

run: bochs

bochs: build/harddrive.bin
	bochs -f bochs.$(ARCH)

qemu: build/harddrive.bin
	$(QEMU) $(QEMUFLAGS) -drive file=$<,format=raw,index=0,media=disk

FORCE:

build/libcore.rlib: rust/src/libcore/lib.rs
	mkdir -p build
	./rustc.sh --target $(ARCH)-unknown-none.json -C soft-float -o $@ $<

build/liballoc.rlib: rust/src/liballoc/lib.rs build/libcore.rlib
	mkdir -p build
	./rustc.sh --target $(ARCH)-unknown-none.json -C soft-float -o $@ $<

build/librustc_unicode.rlib: rust/src/librustc_unicode/lib.rs build/libcore.rlib
	mkdir -p build
	./rustc.sh --target $(ARCH)-unknown-none.json -C soft-float -o $@ $<

build/libcollections.rlib: rust/src/libcollections/lib.rs build/libcore.rlib build/liballoc.rlib build/librustc_unicode.rlib
	mkdir -p build
	./rustc.sh --target $(ARCH)-unknown-none.json -C soft-float -o $@ $<

build/libkernel.a: build/libcore.rlib build/liballoc.rlib build/libcollections.rlib FORCE
	mkdir -p build
	RUSTC="./rustc.sh" cargo rustc --target $(ARCH)-unknown-none.json -- -C soft-float -o $@

build/kernel.bin: build/libkernel.a
	$(LD) --gc-sections -z max-page-size=0x1000 -T bootloader/x86/kernel.ld -o $@ $<

build/kernel.list: build/kernel.bin
	objdump -C -M intel -D $< > $@

build/harddrive.bin: build/kernel.bin
	nasm -f bin -o $@ -D ARCH_$(ARCH) -ibootloader/x86/ -ibuild/ bootloader/x86/harddrive.asm

clean:
	rm -rf build/* target/*
