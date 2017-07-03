build/initfs.tag: initfs.toml
	cd cookbook && ./setup.sh
	cargo run --manifest-path installer/Cargo.toml -- $(INSTALLER_FLAGS) $<
	touch $@
