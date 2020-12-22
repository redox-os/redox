build/initfs.tag: initfs.toml prefix
	cargo build --manifest-path cookbook/Cargo.toml --release
	cargo build --manifest-path installer/Cargo.toml --release
	rm -f build/libkernel.a
	rm -rf build/initfs
	mkdir -p build/initfs
	$(INSTALLER) -c $< build/initfs/
	#TODO: HACK FOR SMALLER INITFS, FIX IN PACKAGING
	for bin in alxd e1000d ihdad ixgbed pcspkrd redoxfs-ar redoxfs-mkfs rtl8168d usbctl usbhidd usbscsid xhcid; do \
		rm -f build/initfs/bin/$$bin; \
	done
	touch $@

build/initfs_coreboot.tag: initfs_coreboot.toml prefix
	cargo build --manifest-path cookbook/Cargo.toml --release
	cargo build --manifest-path installer/Cargo.toml --release
	rm -f build/libkernel_coreboot.a
	rm -rf build/initfs_coreboot
	mkdir -p build/initfs_coreboot
	$(INSTALLER) -c $< build/initfs_coreboot/
	touch $@

build/initfs_live.tag: initfs_live.toml prefix
	cargo build --manifest-path cookbook/Cargo.toml --release
	cargo build --manifest-path installer/Cargo.toml --release
	rm -f build/libkernel_live.a
	rm -rf build/initfs_live
	mkdir -p build/initfs_live
	$(INSTALLER) -c $< build/initfs_live/
	touch $@
