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

build/livedisk.iso: build/livedisk.bin.gz
	rm -rf build/iso/
	mkdir -p build/iso/
	cp -RL isolinux build/iso/
	cp $< build/iso/livedisk.gz
	genisoimage -o $@ -b isolinux/isolinux.bin -c isolinux/boot.cat \
					-no-emul-boot -boot-load-size 4 -boot-info-table \
					build/iso/
	isohybrid $@

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
