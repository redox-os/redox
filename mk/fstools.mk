fstools: $(FSTOOLS_TAG)

$(FSTOOLS_TAG): cookbook installer redoxfs $(CONTAINER_TAG)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) $(MAKE) $@
else
	$(HOST_CARGO) build --manifest-path cookbook/Cargo.toml --release
	$(HOST_CARGO) build --manifest-path installer/Cargo.toml --release
	$(HOST_CARGO) build --manifest-path redoxfs/Cargo.toml --release --bin redoxfs
	$(HOST_CARGO) build --manifest-path redoxfs/Cargo.toml --release --bin redoxfs-mkfs
	mkdir -p build
	touch $@
endif

fstools_clean: FORCE $(CONTAINER_TAG)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) $(MAKE) $@
else
	$(HOST_CARGO) clean --manifest-path cookbook/Cargo.toml
	$(HOST_CARGO) clean --manifest-path installer/Cargo.toml
	$(HOST_CARGO) clean --manifest-path redoxfs/Cargo.toml
	rm -f $(FSTOOLS_TAG)
endif
