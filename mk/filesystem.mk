build/filesystem.bin: userspace
	-$(FUMOUNT) build/filesystem/ || true
	rm -rf $@ build/filesystem/
	dd if=/dev/zero of=$@ bs=1048576 count=64
	cargo run --manifest-path schemes/redoxfs/Cargo.toml --release --bin redoxfs-mkfs $@
	mkdir -p build/filesystem/
	cargo build --manifest-path schemes/redoxfs/Cargo.toml --release --bin redoxfs
	cargo run --manifest-path schemes/redoxfs/Cargo.toml --release --bin redoxfs -- $@ build/filesystem/
	sleep 2
	pgrep redoxfs
	cp -RL filesystem/* build/filesystem/
	chown -R 0:0 build/filesystem
	chown -R 1000:1000 build/filesystem/home/user
	chmod -R uog+rX build/filesystem
	chmod -R u+w build/filesystem
	chmod -R og-w build/filesystem
	chmod -R 755 build/filesystem/bin
	chmod -R u+rwX build/filesystem/root
	chmod -R og-rwx build/filesystem/root
	chmod -R u+rwX build/filesystem/home/user
	chmod -R og-rwx build/filesystem/home/user
	chmod +s build/filesystem/bin/passwd
	chmod +s build/filesystem/bin/su
	chmod +s build/filesystem/bin/sudo
	mkdir build/filesystem/tmp
	chmod 1777 build/filesystem/tmp
	sync
	-$(FUMOUNT) build/filesystem/ || true
	rm -rf build/filesystem/

mount: FORCE
	mkdir -p build/filesystem/
	cargo build --manifest-path schemes/redoxfs/Cargo.toml --release --bin redoxfs
	cargo run --manifest-path schemes/redoxfs/Cargo.toml --release --bin redoxfs -- build/harddrive.bin build/filesystem/
	sleep 2
	pgrep redoxfs

unmount: FORCE
	sync
	-$(FUMOUNT) build/filesystem/ || true
	rm -rf build/filesystem/
