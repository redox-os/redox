userutils: \
	filesystem/bin/getty \
	filesystem/bin/id \
	filesystem/bin/login \
	filesystem/bin/passwd \
	filesystem/bin/su \
	filesystem/bin/sudo \
	filesystem/bin/whoami

filesystem/bin/%: programs/userutils/Cargo.toml programs/userutils/src/bin/%.rs $(BUILD)/libstd.rlib
	mkdir -p filesystem/bin
	$(CARGO) rustc --manifest-path $< --bin $* $(CARGOFLAGS) -o $@
	$(STRIP) $@
