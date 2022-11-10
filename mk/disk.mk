$(BUILD)/harddrive.img: $(REPO_TAG)
	mkdir -p $(BUILD)
	rm -rf $@  $@.partial
	-$(FUMOUNT) /tmp/redox_installer || true
	fallocate --posix --length "$(FILESYSTEM_SIZE)MiB" $@.partial
	$(INSTALLER) -c $(FILESYSTEM_CONFIG) $@.partial
	mv $@.partial $@

$(BUILD)/livedisk.iso: $(REPO_TAG)
	mkdir -p $(BUILD)
	rm -rf $@  $@.partial
	-$(FUMOUNT) /tmp/redox_installer || true
	fallocate --posix --length "$(FILESYSTEM_SIZE)MiB" $@.partial
	$(INSTALLER) -c $(FILESYSTEM_CONFIG) --live $@.partial
	mv $@.partial $@

$(BUILD)/filesystem.img: $(REPO_TAG)
	mkdir -p $(BUILD)
	$(HOST_CARGO) build --manifest-path redoxfs/Cargo.toml --release
	-$(FUMOUNT) $(BUILD)/filesystem/ || true
	rm -rf $@  $@.partial $(BUILD)/filesystem/
	-$(FUMOUNT) /tmp/redox_installer || true
	fallocate --posix --length "$(FILESYSTEM_SIZE)MiB" $@.partial
	redoxfs/target/release/redoxfs-mkfs $(REDOXFS_MKFS_FLAGS) $@.partial
	mkdir -p $(BUILD)/filesystem/
	redoxfs/target/release/redoxfs $@.partial $(BUILD)/filesystem/
	sleep 1
	pgrep redoxfs
	$(INSTALLER) -c $(FILESYSTEM_CONFIG) $(BUILD)/filesystem/
	sync
	-$(FUMOUNT) $(BUILD)/filesystem/ || true
	rm -rf $(BUILD)/filesystem/
	mv $@.partial $@

mount: FORCE
	mkdir -p $(BUILD)/filesystem/
	$(HOST_CARGO) build --manifest-path redoxfs/Cargo.toml --release --bin redoxfs
	redoxfs/target/release/redoxfs $(BUILD)/harddrive.img $(BUILD)/filesystem/
	sleep 2
	pgrep redoxfs

mount_extra: FORCE
	mkdir -p $(BUILD)/filesystem/
	$(HOST_CARGO) build --manifest-path redoxfs/Cargo.toml --release --bin redoxfs
	redoxfs/target/release/redoxfs $(BUILD)/extra.img $(BUILD)/filesystem/
	sleep 2
	pgrep redoxfs

unmount: FORCE
	sync
	-$(FUMOUNT) $(BUILD)/filesystem/ || true
	rm -rf $(BUILD)/filesystem/
	-$(FUMOUNT) /tmp/redox_installer || true
