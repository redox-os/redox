orbutils: \
	filesystem/ui/bin/browser \
	filesystem/ui/bin/calculator \
	filesystem/ui/bin/character_map \
	filesystem/ui/bin/editor \
	filesystem/ui/bin/file_manager \
	filesystem/ui/bin/launcher \
	filesystem/ui/bin/orblogin \
	filesystem/ui/bin/orbterm \
	filesystem/ui/bin/viewer

filesystem/ui/bin/%: programs/orbutils/Cargo.toml programs/orbutils/src/%/**.rs $(BUILD)/libstd.rlib
	mkdir -p filesystem/ui/bin
	$(CARGO) rustc --manifest-path $< --bin $* $(CARGOFLAGS) -o $@
	$(STRIP) $@
