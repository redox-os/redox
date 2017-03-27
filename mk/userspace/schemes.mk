schemes: \
	filesystem/sbin/ethernetd \
	filesystem/sbin/orbital \
	filesystem/sbin/ptyd \
	filesystem/sbin/randd \
	filesystem/sbin/redoxfs \
	filesystem/sbin/redoxfs-mkfs

initfs/bin/%: schemes/%/Cargo.toml schemes/%/src/** $(BUILD)/libstd.rlib
	mkdir -p initfs/bin
	$(CARGO) rustc --manifest-path $< --bin $* $(CARGOFLAGS) -o $@
	$(STRIP) $@

filesystem/sbin/%: schemes/%/Cargo.toml schemes/%/src/** $(BUILD)/libstd.rlib
	mkdir -p filesystem/sbin
	$(CARGO) rustc --manifest-path $< --bin $* $(CARGOFLAGS) -o $@
	$(STRIP) $@

filesystem/sbin/redoxfs-mkfs: schemes/redoxfs/Cargo.toml schemes/redoxfs/src/** $(BUILD)/libstd.rlib
	mkdir -p filesystem/bin
	$(CARGO) rustc --manifest-path $< --bin redoxfs-mkfs $(CARGOFLAGS) -o $@
	$(STRIP) $@
