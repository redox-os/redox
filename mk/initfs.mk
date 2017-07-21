build/initfs.tag: initfs.toml
	cd kernel && xargo clean
	cargo run --manifest-path installer/Cargo.toml -- $(INSTALLER_FLAGS) $<
	touch $@
