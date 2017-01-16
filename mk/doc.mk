doc: $(KBUILD)/libkernel.a $(BUILD)/libstd.rlib FORCE
	$(KCARGO) doc --target $(KTARGET) --manifest-path kernel/Cargo.toml
	$(CARGO) doc --target $(TARGET) --manifest-path rust/src/libstd/Cargo.toml

ref: FORCE
	rm -rf filesystem/ref/
	mkdir -p filesystem/ref/
	#cargo run --manifest-path docgen/Cargo.toml -- programs/binutils/src/bin/ filesystem/ref/
	cargo run --manifest-path docgen/Cargo.toml -- programs/coreutils/src/bin/ filesystem/ref/
	cargo run --manifest-path docgen/Cargo.toml -- programs/extrautils/src/bin/ filesystem/ref/
	cargo run --manifest-path docgen/Cargo.toml -- programs/netutils/src/ filesystem/ref/
	cargo run --manifest-path docgen/Cargo.toml -- programs/pkgutils/src/ filesystem/ref/
	cargo run --manifest-path docgen/Cargo.toml -- programs/userutils/src/ filesystem/ref/
