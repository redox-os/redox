games: \
	filesystem/bin/ice \
	filesystem/bin/minesweeper \
	filesystem/bin/reblox \
	filesystem/bin/rusthello \
	filesystem/bin/snake

filesystem/bin/%: programs/games/Cargo.toml programs/games/src/%/**.rs $(BUILD)/libstd.rlib
	mkdir -p filesystem/bin
	$(CARGO) rustc --manifest-path $< --bin $* $(CARGOFLAGS) -o $@
	$(STRIP) $@
