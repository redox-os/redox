build/libkernel.a: kernel/Cargo.toml kernel/src/* kernel/src/*/* kernel/src/*/*/* build/initfs.tag
# Temporary fix for https://github.com/redox-os/redox/issues/963 allowing to build on macOS
ifeq ($(UNAME),Darwin)
	cd kernel && CC=$(ARCH)-elf-gcc AR=$(ARCH)-elf-ar CFLAGS=-ffreestanding xargo rustc --lib --target $(KTARGET) --release -- -C soft-float --emit link=../$@
else
	cd kernel && xargo rustc --lib --target $(KTARGET) --release -- -C soft-float --emit link=../$@
endif

build/libkernel_live.a: kernel/Cargo.toml kernel/src/* kernel/src/*/* kernel/src/*/*/* build/initfs.tag build/filesystem.bin
	cd kernel && FILESYSTEM="$(PWD)/build/filesystem.bin" xargo rustc --lib --features live --target $(KTARGET) --release -- -C soft-float --emit link=../$@

build/kernel: build/libkernel.a kernel/linkers/$(ARCH).ld
	$(LD) --gc-sections -z max-page-size=0x1000 -T kernel/linkers/$(ARCH).ld -o $@ $<

build/kernel_live: build/libkernel_live.a kernel/linkers/$(ARCH).ld
	$(LD) --gc-sections -z max-page-size=0x1000 -T kernel/linkers/$(ARCH).ld -o $@ $<
