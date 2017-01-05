ion: \
	filesystem/bin/ion \
	filesystem/bin/sh

filesystem/test/ion: programs/ion/Cargo.toml programs/ion/src/** $(BUILD)/libstd.rlib $(BUILD)/libtest.rlib
	mkdir -p filesystem/test
	$(CARGO) test --no-run --manifest-path $< $(CARGOFLAGS)
	cp programs/ion/target/$(TARGET)/release/deps/ion-* $@

filesystem/bin/sh: filesystem/bin/ion
	cp $< $@
