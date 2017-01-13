pkgutils: \
	filesystem/bin/pkg

filesystem/bin/%: programs/pkgutils/Cargo.toml programs/pkgutils/src/bin/%.rs $(BUILD)/libstd.rlib
	mkdir -p filesystem/bin
	$(CARGO) rustc --manifest-path $< --bin $* $(CARGOFLAGS) -o $@
	$(STRIP) $@
