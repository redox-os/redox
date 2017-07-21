build/initfs.tag: initfs.toml
	cd kernel && xargo clean
	rm -rf build/initfs
	cargo run --manifest-path installer/Cargo.toml -- $(INSTALLER_FLAGS) $<
	touch $@
