build/filesystem.bin: filesystem.toml
	-$(FUMOUNT) build/filesystem/ || true
	rm -rf $@  $@.partial build/filesystem/
	dd if=/dev/zero of=$@.partial bs=1048576 count=1024
	cargo run --manifest-path installer/redoxfs/Cargo.toml --quiet --release --bin redoxfs-mkfs $@.partial
	mkdir -p build/filesystem/
	cargo build --manifest-path installer/redoxfs/Cargo.toml --quiet --release --bin redoxfs
	cargo run --manifest-path installer/redoxfs/Cargo.toml --quiet --release --bin redoxfs -- $@.partial build/filesystem/
	sleep 2
	pgrep redoxfs
	cargo run --manifest-path installer/Cargo.toml -- --cookbook=cookbook $<
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
	mv $@.partial $@

mount: FORCE
	mkdir -p build/filesystem/
	cargo build --manifest-path installer/redoxfs/Cargo.toml --quiet --release --bin redoxfs
	cargo run --manifest-path installer/redoxfs/Cargo.toml --quiet --release --bin redoxfs -- build/harddrive.bin build/filesystem/
	sleep 2
	pgrep redoxfs

unmount: FORCE
	sync
	-$(FUMOUNT) build/filesystem/ || true
	rm -rf build/filesystem/

.PHONY: build/filesystem.bin
