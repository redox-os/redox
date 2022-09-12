build/fetch.tag: cookbook installer prefix $(FILESYSTEM_CONFIG)
	$(HOST_CARGO) build --manifest-path cookbook/Cargo.toml --release
	$(HOST_CARGO) build --manifest-path installer/Cargo.toml --release
	PACKAGES="$$($(INSTALLER) --list-packages -c $(FILESYSTEM_CONFIG))" && \
	cd cookbook && \
	./fetch.sh "$${PACKAGES}"
	touch $@

build/repo.tag: build/fetch.tag
	$(HOST_CARGO) build --manifest-path cookbook/Cargo.toml --release
	$(HOST_CARGO) build --manifest-path installer/Cargo.toml --release
	export PATH="$(PREFIX_PATH):$$PATH" && \
	PACKAGES="$$($(INSTALLER) --list-packages -c $(FILESYSTEM_CONFIG))" && \
	cd cookbook && \
	./repo.sh "$${PACKAGES}"
	mkdir -p build
	touch $@
