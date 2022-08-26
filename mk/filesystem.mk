build/filesystem.bin: prefix $(FILESYSTEM_CONFIG)
	$(HOST_CARGO) build --manifest-path cookbook/Cargo.toml --release
	$(HOST_CARGO) build --manifest-path installer/Cargo.toml --release
	$(HOST_CARGO) build --manifest-path redoxfs/Cargo.toml --release
	-$(FUMOUNT) build/filesystem/ || true
	rm -rf $@  $@.partial build/filesystem/
	fallocate --posix --length "$(FILESYSTEM_SIZE)MiB" $@.partial
	$(HOST_CARGO) run --release \
		--manifest-path redoxfs/Cargo.toml \
		--bin redoxfs-mkfs \
		-- $(REDOXFS_MKFS_FLAGS) $@.partial
	mkdir -p build/filesystem/
	redoxfs/target/release/redoxfs $@.partial build/filesystem/
	sleep 2
	pgrep redoxfs
	$(INSTALLER) -c $(FILESYSTEM_CONFIG) build/filesystem/
	cp -v $(FILESYSTEM_CONFIG) build/filesystem/filesystem.toml
	mkdir -pv build/filesystem/pkg
	cp -v cookbook/build/id_ed25519.pub.toml build/filesystem/pkg/id_ed25519.pub.toml
	sync
	-$(FUMOUNT) build/filesystem/ || true
	rm -rf build/filesystem/
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
