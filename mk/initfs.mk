build/initfs.tag: initfs.toml
	cd kernel && xargo clean
	rm -rf build/initfs
	mkdir -p build/initfs
	cargo run --manifest-path installer/Cargo.toml -- $(INSTALLER_FLAGS) -c $< build/initfs/
	touch $@
