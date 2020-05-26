build/filesystem.bin: filesystem.toml build/bootloader build/kernel prefix
	cargo build --manifest-path cookbook/Cargo.toml --release
	cargo build --manifest-path installer/Cargo.toml --release
	cargo build --manifest-path redoxfs/Cargo.toml --release
	-$(FUMOUNT) build/filesystem/ || true
	rm -rf $@  $@.partial build/filesystem/
	dd if=/dev/zero of=$@.partial bs=1048576 count="$(FILESYSTEM_SIZE)"
	cargo run --manifest-path redoxfs/Cargo.toml --release --bin redoxfs-mkfs $@.partial
	mkdir -p build/filesystem/
	redoxfs/target/release/redoxfs $@.partial build/filesystem/
	sleep 2
	pgrep redoxfs
	cp $< build/filesystem/filesystem.toml
	cp build/bootloader build/filesystem/bootloader
	cp build/kernel build/filesystem/kernel
	cp -r $(ROOT)/$(PREFIX_INSTALL)/$(TARGET)/include build/filesystem/include
	cp -r $(ROOT)/$(PREFIX_INSTALL)/$(TARGET)/lib build/filesystem/lib
	$(INSTALLER) -c $< build/filesystem/
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
