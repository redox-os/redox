build/harddrive.img: $(REPO_TAG)
	mkdir -p build
	rm -rf $@  $@.partial
	fallocate --posix --length "$(FILESYSTEM_SIZE)MiB" $@.partial
	$(INSTALLER) -c $(FILESYSTEM_CONFIG) $@.partial
	mv $@.partial $@

build/livedisk.iso: $(REPO_TAG)
	mkdir -p build
	rm -rf $@  $@.partial
	fallocate --posix --length "$(FILESYSTEM_SIZE)MiB" $@.partial
	$(INSTALLER) -c $(FILESYSTEM_CONFIG) --live $@.partial
	mv $@.partial $@

build/filesystem.img: $(REPO_TAG)
	mkdir -p build
	$(HOST_CARGO) build --manifest-path redoxfs/Cargo.toml --release
	-$(FUMOUNT) build/filesystem/ || true
	rm -rf $@  $@.partial build/filesystem/
	fallocate --posix --length "$(FILESYSTEM_SIZE)MiB" $@.partial
	redoxfs/target/release/redoxfs-mkfs $(REDOXFS_MKFS_FLAGS) $@.partial
	mkdir -p build/filesystem/
	redoxfs/target/release/redoxfs $@.partial build/filesystem/
	sleep 1
	pgrep redoxfs
	$(INSTALLER) -c $(FILESYSTEM_CONFIG) build/filesystem/
	sync
	-$(FUMOUNT) build/filesystem/ || true
	rm -rf build/filesystem/
	mv $@.partial $@

mount: FORCE
	mkdir -p build/filesystem/
	$(HOST_CARGO) build --manifest-path redoxfs/Cargo.toml --release --bin redoxfs
	redoxfs/target/release/redoxfs build/harddrive.img build/filesystem/
	sleep 2
	pgrep redoxfs

mount_extra: FORCE
	mkdir -p build/filesystem/
	$(HOST_CARGO) build --manifest-path redoxfs/Cargo.toml --release --bin redoxfs
	redoxfs/target/release/redoxfs build/extra.img build/filesystem/
	sleep 2
	pgrep redoxfs

unmount: FORCE
	sync
	-$(FUMOUNT) build/filesystem/ || true
	rm -rf build/filesystem/
