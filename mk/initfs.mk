build/initfs.tag: initfs.toml prefix
	rm -f build/libkernel.a
	rm -rf build/initfs
	mkdir -p build/initfs
	export PATH="$(PREFIX_PATH):$$PATH" && \
	cargo run --manifest-path installer/Cargo.toml -- $(INSTALLER_FLAGS) -c $< build/initfs/
	touch $@

build/initfs_coreboot.tag: initfs_coreboot.toml prefix
	rm -f build/libkernel_coreboot.a
	rm -rf build/initfs_coreboot
	mkdir -p build/initfs_coreboot
	export PATH="$(PREFIX_PATH):$$PATH" && \
	cargo run --manifest-path installer/Cargo.toml -- $(INSTALLER_FLAGS) -c $< build/initfs_coreboot/
	touch $@

build/initfs_live.tag: initfs_live.toml prefix
	rm -f build/libkernel_live.a
	rm -rf build/initfs_live
	mkdir -p build/initfs_live
	export PATH="$(PREFIX_PATH):$$PATH" && \
	cargo run --manifest-path installer/Cargo.toml -- $(INSTALLER_FLAGS) -c $< build/initfs_live/
	touch $@
