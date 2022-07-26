INITFS_RM_BINS=\
	redoxfs-ar \
	redoxfs-mkfs

build/initfs.img: initfs.toml prefix
	$(HOST_CARGO) build --manifest-path cookbook/Cargo.toml --release
	$(HOST_CARGO) build --manifest-path installer/Cargo.toml --release
	rm -rf build/initfs
	mkdir -p build/initfs
	$(INSTALLER) -c $< build/initfs/
	#TODO: HACK FOR SMALLER INITFS, FIX IN PACKAGING
	rm -rf build/initfs/pkg
	for bin in $(INITFS_RM_BINS); do \
		rm -f build/initfs/bin/$$bin; \
	done
	cargo run --manifest-path redox-initfs/tools/Cargo.toml --bin redox-initfs-ar -- build/initfs -o $@
