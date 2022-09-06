build/harddrive.bin: $(FILESYSTEM_CONFIG)
	$(HOST_CARGO) build --manifest-path installer/Cargo.toml --release
	mkdir -p build
	rm -rf $@  $@.partial
	fallocate --posix --length "$(FILESYSTEM_SIZE)MiB" $@.partial
	$(INSTALLER) -c $(FILESYSTEM_CONFIG) $@.partial
	mv $@.partial $@

build/livedisk.bin: $(FILESYSTEM_CONFIG)
	$(HOST_CARGO) build --manifest-path installer/Cargo.toml --release
	mkdir -p build
	rm -rf $@  $@.partial
	fallocate --posix --length "$(FILESYSTEM_SIZE)MiB" $@.partial
	$(INSTALLER) -c $(FILESYSTEM_CONFIG) --live $@.partial
	mv $@.partial $@

build/livedisk.iso: build/livedisk.bin
	rm -rf build/iso/ $@.partial
	mkdir -p build/iso/
	cp -v $< build/iso/livedisk.bin
	genisoimage -o $@.partial \
		-hard-disk-boot \
		-b livedisk.bin \
		build/iso/
	mv $@.partial $@

mount: FORCE
	mkdir -p build/filesystem/
	$(HOST_CARGO) build --manifest-path redoxfs/Cargo.toml --release --bin redoxfs
	redoxfs/target/release/redoxfs build/harddrive.bin build/filesystem/
	sleep 2
	pgrep redoxfs

mount_extra: FORCE
	mkdir -p build/filesystem/
	$(HOST_CARGO) build --manifest-path redoxfs/Cargo.toml --release --bin redoxfs
	redoxfs/target/release/redoxfs build/extra.bin build/filesystem/
	sleep 2
	pgrep redoxfs

unmount: FORCE
	sync
	-$(FUMOUNT) build/filesystem/ || true
	rm -rf build/filesystem/
