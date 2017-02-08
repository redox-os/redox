$(KBUILD)/libcollections.rlib: rust/src/libcollections/Cargo.toml rust/src/libcollections/**
	mkdir -p $(KBUILD)
	$(KCARGO) rustc --manifest-path $< $(KCARGOFLAGS) -o $@
	cp rust/src/target/$(KTARGET)/release/deps/*.rlib $(KBUILD)

$(KBUILD)/libkernel.a: kernel/Cargo.toml kernel/arch/** kernel/src/** $(KBUILD)/libcollections.rlib $(BUILD)/initfs.rs
	$(KCARGO) rustc --manifest-path $< --lib $(KCARGOFLAGS) -C lto -o $@

$(KBUILD)/libkernel_live.a: kernel/Cargo.toml kernel/arch/** kernel/src/** $(KBUILD)/libcollections.rlib $(BUILD)/initfs.rs build/filesystem.bin
	$(KCARGO) rustc --manifest-path $< --lib --features live $(KCARGOFLAGS) -C lto -o $@

$(KBUILD)/kernel: $(KBUILD)/libkernel.a
	$(LD) $(LDFLAGS) -z max-page-size=0x1000 -T kernel/arch/$(ARCH)/src/linker.ld -o $@ $<

$(KBUILD)/kernel_live: $(KBUILD)/libkernel_live.a
	$(LD) $(LDFLAGS) -z max-page-size=0x1000 -T kernel/arch/$(ARCH)/src/linker.ld -o $@ $<
