ARCH?=x86_64

QEMU=qemu-system-$(ARCH)
QEMUFLAGS=-serial mon:stdio -d guest_errors

RUSTCFLAGS=--target $(ARCH)-unknown-redox.json -O -C soft-float --cfg redox
CARGOFLAGS=--target $(ARCH)-unknown-redox.json -- -O -C soft-float --cfg redox

ifeq ($(ARCH),arm)
	LD=$(ARCH)-none-eabi-ld
	QEMUFLAGS+=-cpu arm1176 -machine integratorcp
	QEMUFLAGS+=-nographic
else
	LD=ld
	QEMUFLAGS+=-enable-kvm -cpu host -machine q35 -smp 4
	QEMUFLAGS+=-nographic -vga none
	#,int,pcall
	#-device intel-iommu

	UNAME := $(shell uname)
	ifeq ($(UNAME),Darwin)
		LD=$(ARCH)-elf-ld
		QEMUFLAGS=
	endif
endif

all: build/kernel.bin

list: build/kernel.list

run: bochs

bochs: build/harddrive.bin
	bochs -f bochs.$(ARCH)

FORCE:

build/libcore.rlib: rust/src/libcore/lib.rs
	mkdir -p build
	./rustc.sh $(RUSTCFLAGS) -o $@ $<

build/librand.rlib: rust/src/librand/lib.rs build/libcore.rlib
	./rustc.sh $(RUSTCFLAGS) -o $@ $<

build/liballoc.rlib: rust/src/liballoc/lib.rs build/libcore.rlib
	./rustc.sh $(RUSTCFLAGS) -o $@ $<

build/librustc_unicode.rlib: rust/src/librustc_unicode/lib.rs build/libcore.rlib
	./rustc.sh $(RUSTCFLAGS) -o $@ $<

build/libcollections.rlib: rust/src/libcollections/lib.rs build/libcore.rlib build/liballoc.rlib build/librustc_unicode.rlib
	./rustc.sh $(RUSTCFLAGS) -o $@ $<

build/libstd.rlib: libstd/Cargo.toml libstd/src/** build/libcore.rlib build/liballoc.rlib build/librustc_unicode.rlib build/libcollections.rlib build/librand.rlib
	RUSTC="./rustc.sh" cargo rustc --manifest-path $< $(CARGOFLAGS) -o $@
	cp libstd/target/$(ARCH)-unknown-redox/debug/deps/*.rlib build

build/init: init/Cargo.toml init/src/*.rs build/libstd.rlib
	RUSTC="./rustc.sh" cargo rustc --manifest-path $< $(CARGOFLAGS) -o $@
	strip $@

build/libkernel.a: build/libcore.rlib build/liballoc.rlib build/libcollections.rlib build/init kernel/**
	RUSTC="./rustc.sh" cargo rustc $(CARGOFLAGS) -o $@

build/kernel.bin: build/libkernel.a
	$(LD) --gc-sections -z max-page-size=0x1000 -T arch/$(ARCH)/src/linker.ld -o $@ $<

ifeq ($(ARCH),arm)
build/kernel.list: build/kernel.bin
	$(ARCH)-none-eabi-objdump -C -D $< > $@

qemu: build/kernel.bin
	$(QEMU) $(QEMUFLAGS) -kernel $<
else
build/kernel.list: build/kernel.bin
	objdump -C -M intel -D $< > $@

build/harddrive.bin: build/kernel.bin
	nasm -f bin -o $@ -D ARCH_$(ARCH) -ibootloader/$(ARCH)/ -ibuild/ bootloader/$(ARCH)/harddrive.asm

qemu: build/harddrive.bin
	$(QEMU) $(QEMUFLAGS) -drive file=$<,format=raw,index=0,media=disk
endif

clean:
	cargo clean
	cargo clean --manifest-path libstd/Cargo.toml
	cargo clean --manifest-path init/Cargo.toml
	rm -rf build/*
