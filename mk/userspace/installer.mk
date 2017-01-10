installer: \
	filesystem/bin/redox_installer

filesystem/bin/redox_installer: installer/Cargo.toml installer/src/** $(BUILD)/libstd.rlib
	mkdir -p filesystem/bin
	$(CARGO) rustc --manifest-path $< --bin redox_installer $(CARGOFLAGS) -o $@
	$(STRIP) $@
