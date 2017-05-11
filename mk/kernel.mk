build/libkernel.a: kernel/Cargo.toml kernel/src/** build/initfs.tag
	cd kernel && xargo rustc --lib --target $(KTARGET) --release -- -C soft-float --emit link=../$@

build/libkernel_live.a: kernel/Cargo.toml kernel/src/** build/initfs.tag build/filesystem.bin
	cd kernel && xargo rustc --lib --features live --target $(KTARGET) --release -- -C soft-float --emit link=../$@

build/kernel: build/libkernel.a
	$(LD) --gc-sections -z max-page-size=0x1000 -T kernel/linkers/$(ARCH).ld -o $@ $<

build/kernel_live: build/libkernel_live.a
	$(LD) --gc-sections -z max-page-size=0x1000 -T kernel/linkers/$(ARCH).ld -o $@ $<
