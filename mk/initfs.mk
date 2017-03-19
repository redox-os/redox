$(KBUILD)/initfs.tag: initfs/bin/init \
	initfs/bin/ahcid \
	initfs/bin/bgad \
	initfs/bin/nvmed \
	initfs/bin/pcid \
	initfs/bin/ps2d \
	initfs/bin/redoxfs \
	initfs/bin/vboxd \
	initfs/bin/vesad \
	initfs/etc/**
		$(KCARGO) clean --manifest-path kernel/Cargo.toml
		touch $@

initfs/bin/%: programs/%/Cargo.toml programs/%/src/** $(BUILD)/libstd.rlib
	mkdir -p initfs/bin
	$(CARGO) rustc --manifest-path $< $(CARGOFLAGS) -o $@
	$(STRIP) $@
