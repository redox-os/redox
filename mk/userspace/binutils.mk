binutils: \
	filesystem/bin/hex \
	filesystem/bin/hexdump \
	filesystem/bin/strings

filesystem/bin/%: programs/binutils/Cargo.toml programs/binutils/src/bin/%.rs $(BUILD)/libstd.rlib
	mkdir -p filesystem/bin
	$(CARGO) rustc --manifest-path $< --bin $* $(CARGOFLAGS) -o $@
	$(STRIP) $@
