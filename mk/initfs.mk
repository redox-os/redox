$(BUILD)/initfs.rs: \
		initfs/bin/init \
		initfs/bin/ahcid \
		initfs/bin/bgad \
		initfs/bin/pcid \
		initfs/bin/ps2d \
		initfs/bin/redoxfs \
		initfs/bin/vesad \
		initfs/etc/**

initfs/bin/%: programs/%/Cargo.toml programs/%/src/** $(BUILD)/libstd.rlib
	mkdir -p initfs/bin
	$(CARGO) rustc --manifest-path $< $(CARGOFLAGS) -o $@
	$(STRIP) $@
