build/filesystem.bin: prefix filesystem.toml build/bootloader.bin build/kernel build/initfs.img
	cargo build --manifest-path cookbook/Cargo.toml --release
	cargo build --manifest-path installer/Cargo.toml --release
	cargo build --manifest-path redoxfs/Cargo.toml --release
	-$(FUMOUNT) build/filesystem/ || true
	rm -rf $@  $@.partial build/filesystem/
	fallocate --posix --length "$(FILESYSTEM_SIZE)MiB" $@.partial
	cargo run --release \
		--manifest-path redoxfs/Cargo.toml \
		--bin redoxfs-mkfs \
		-- $(REDOXFS_MKFS_FLAGS) $@.partial
	mkdir -p build/filesystem/
	redoxfs/target/release/redoxfs $@.partial build/filesystem/
	sleep 2
	pgrep redoxfs
	cp -v filesystem.toml build/filesystem/filesystem.toml
	cp -v build/bootloader.bin build/filesystem/bootloader
	cp -v build/kernel build/filesystem/kernel
	mkdir -v build/filesystem/pkg
	cp -v cookbook/build/id_ed25519.pub.toml build/filesystem/pkg/id_ed25519.pub.toml
	#TODO cp -r $(ROOT)/$(PREFIX_INSTALL)/$(TARGET)/include build/filesystem/include
	#TODO cp -r $(ROOT)/$(PREFIX_INSTALL)/$(TARGET)/lib build/filesystem/lib
	$(INSTALLER) -c filesystem.toml build/filesystem/
	cp build/initfs.img build/filesystem/initfs
	sync
	-$(FUMOUNT) build/filesystem/ || true
	rm -rf build/filesystem/
	mv $@.partial $@

mount: FORCE
	mkdir -p build/filesystem/
	cargo build --manifest-path redoxfs/Cargo.toml --release --bin redoxfs
	redoxfs/target/release/redoxfs build/harddrive.bin build/filesystem/
	sleep 2
	pgrep redoxfs

mount_extra: FORCE
	mkdir -p build/filesystem/
	cargo build --manifest-path redoxfs/Cargo.toml --release --bin redoxfs
	redoxfs/target/release/redoxfs build/extra.bin build/filesystem/
	sleep 2
	pgrep redoxfs

unmount: FORCE
	sync
	-$(FUMOUNT) build/filesystem/ || true
	rm -rf build/filesystem/
