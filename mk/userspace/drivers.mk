drivers: \
	filesystem/sbin/pcid \
	filesystem/sbin/e1000d \
	filesystem/sbin/rtl8168d

initfs/bin/%: drivers/%/Cargo.toml drivers/%/src/** $(BUILD)/libstd.rlib
	mkdir -p initfs/bin
	$(CARGO) rustc --manifest-path $< $(CARGOFLAGS) -o $@
	$(STRIP) $@

filesystem/sbin/%: drivers/%/Cargo.toml drivers/%/src/** $(BUILD)/libstd.rlib
	mkdir -p filesystem/sbin
	$(CARGO) rustc --manifest-path $< $(CARGOFLAGS) -o $@
	$(STRIP) $@
