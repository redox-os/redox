build/libkernel.a: kernel/Cargo.lock kernel/Cargo.toml kernel/src/* kernel/src/*/* kernel/src/*/*/* kernel/src/*/*/*/* build/initfs.tag
	export PATH="$(PREFIX_PATH):$$PATH" && \
	export INITFS_FOLDER=$(ROOT)/build/initfs && \
	cd kernel && \
	cargo rustc --lib --target=$(ROOT)/kernel/targets/$(KTARGET).json --release -- -C soft-float -C debuginfo=2 -C lto --emit link=../$@

build/libkernel_coreboot.a: kernel/Cargo.toml kernel/src/* kernel/src/*/* kernel/src/*/*/* kernel/src/*/*/*/* build/initfs_coreboot.tag
	export PATH="$(PREFIX_PATH):$$PATH" && \
	export INITFS_FOLDER=$(ROOT)/build/initfs_coreboot && \
	cd kernel && \
	cargo rustc --lib --target=$(ROOT)/kernel/targets/$(KTARGET).json --release --features live -- -C soft-float -C debuginfo=2 -C lto --emit link=../$@

build/libkernel_live.a: kernel/Cargo.toml kernel/src/* kernel/src/*/* kernel/src/*/*/* kernel/src/*/*/*/* build/initfs_live.tag
	export PATH="$(PREFIX_PATH):$$PATH" && \
	export INITFS_FOLDER=$(ROOT)/build/initfs_live && \
	cd kernel && \
	cargo rustc --lib --target=$(ROOT)/kernel/targets/$(KTARGET).json --release --features live -- -C soft-float -C debuginfo=2 -C lto --emit link=../$@

build/kernel: kernel/linkers/$(ARCH).ld mk/kernel_ld.sh build/libkernel.a
	export PATH="$(PREFIX_PATH):$$PATH" && \
	$(ROOT)/mk/kernel_ld.sh $(LD) --gc-sections -z max-page-size=0x1000 -T $< -o $@ build/libkernel.a && \
	$(OBJCOPY) --only-keep-debug $@ $@.sym && \
	$(OBJCOPY) --strip-debug $@

build/kernel_coreboot: kernel/linkers/$(ARCH).ld mk/kernel_ld.sh build/libkernel_coreboot.a build/live.o
	export PATH="$(PREFIX_PATH):$$PATH" && \
	$(ROOT)/mk/kernel_ld.sh $(LD) --gc-sections -z max-page-size=0x1000 -T $< -o $@ build/libkernel_coreboot.a build/live.o && \
	$(OBJCOPY) --only-keep-debug $@ $@.sym && \
	$(OBJCOPY) --strip-debug $@

build/kernel_live: kernel/linkers/$(ARCH).ld mk/kernel_ld.sh build/libkernel_live.a build/live.o
	export PATH="$(PREFIX_PATH):$$PATH" && \
	$(ROOT)/mk/kernel_ld.sh $(LD) --gc-sections -z max-page-size=0x1000 -T $< -o $@ build/libkernel_live.a build/live.o && \
	$(OBJCOPY) --only-keep-debug $@ $@.sym && \
	$(OBJCOPY) --strip-debug $@

build/live.o: build/filesystem.bin
	#TODO: More general use of $(ARCH)
	export PATH="$(PREFIX_PATH):$$PATH" && \
	$(OBJCOPY) -I binary -O elf64-x86-64 -B i386:x86-64 $< $@ \
		--redefine-sym _binary_build_filesystem_bin_start=__live_start \
		--redefine-sym _binary_build_filesystem_bin_end=__live_end \
		--redefine-sym _binary_build_filesystem_bin_size=__live_size
