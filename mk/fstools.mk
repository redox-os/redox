# Configuration file for redox-installer, Cookbook and RedoxFS FUSE

fstools: $(FSTOOLS_TAG) $(HOST_FSTOOLS)

# These tools run inside Podman if it is used, or on the host if Podman is not used
$(FSTOOLS): cookbook installer $(CONTAINER_TAG)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	rm -rf $@ $@.partial
	mkdir -p $@.partial
	$(HOST_CARGO) build --manifest-path cookbook/Cargo.toml --release
	$(HOST_CARGO) build --manifest-path cookbook/pkgar/Cargo.toml --release
	$(HOST_CARGO) install --root $@.partial --path installer --bin redox_installer
	$(HOST_CARGO) install --root $@.partial --path redoxfs --bin redoxfs --bin redoxfs-mkfs --bin redoxfs-resize
	mv $@.partial $@
	touch $@
endif

## TODO: remove this
$(FSTOOLS_TAG): $(FSTOOLS)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	touch $@
endif

fstools_clean: FORCE $(CONTAINER_TAG)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	$(HOST_CARGO) clean --manifest-path cookbook/Cargo.toml
	$(HOST_CARGO) clean --manifest-path cookbook/pkgar/Cargo.toml
	$(HOST_CARGO) clean --manifest-path installer/Cargo.toml
	$(HOST_CARGO) clean --manifest-path redoxfs/Cargo.toml
	rm -rf $(FSTOOLS)
	rm -f $(FSTOOLS_TAG)
endif
