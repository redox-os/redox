extrautils: \
	filesystem/bin/calc \
	filesystem/bin/cksum \
	filesystem/bin/cur \
	filesystem/bin/grep \
	filesystem/bin/gunzip \
	filesystem/bin/gzip \
	filesystem/bin/less \
	filesystem/bin/man \
	filesystem/bin/mdless \
	filesystem/bin/mtxt \
	filesystem/bin/rem \
	filesystem/bin/resize \
	filesystem/bin/screenfetch \
	filesystem/bin/tar
	#filesystem/bin/dmesg filesystem/bin/info  filesystem/bin/watch

filesystem/bin/%: programs/extrautils/Cargo.toml programs/extrautils/src/bin/%.rs $(BUILD)/libstd.rlib
	mkdir -p filesystem/bin
	$(CARGO) rustc --manifest-path $< --bin $* $(CARGOFLAGS) -o $@
	$(STRIP) $@
