build/initfs.tag: initfs.toml
	cargo run --manifest-path installer/Cargo.toml -- --cookbook=cookbook $<
	touch $@
