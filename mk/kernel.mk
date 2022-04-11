build/libkernel.a: kernel/Cargo.lock kernel/Cargo.toml kernel/src/* kernel/src/*/* kernel/src/*/*/* kernel/src/*/*/*/*
	export PATH="$(PREFIX_PATH):$$PATH" && \
	cd kernel && \
	cargo rustc --lib --target=$(ROOT)/kernel/targets/$(KTARGET).json --release -- -C soft-float -C debuginfo=2 -C lto --emit link=../$@

build/kernel: kernel/linkers/$(ARCH).ld mk/kernel_ld.sh build/libkernel.a
	export PATH="$(PREFIX_PATH):$$PATH" && \
	$(ROOT)/mk/kernel_ld.sh $(LD) --gc-sections -z max-page-size=0x1000 -T $< -o $@.all build/libkernel.a && \
	$(OBJCOPY) --only-keep-debug $@.all $@.sym && \
	$(OBJCOPY) --strip-debug $@.all $@
