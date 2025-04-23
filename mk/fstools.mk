# Configuration file for redox-installer, Cookbook and RedoxFS FUSE

fstools: $(FSTOOLS_TAG)

# These tools run inside Podman if it is used, or on the host if Podman is not used
$(FSTOOLS_TAG): cookbook installer build/fstools $(CONTAINER_TAG)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) $(MAKE) $@
else
	$(HOST_CARGO) build --manifest-path cookbook/Cargo.toml --release
	$(HOST_CARGO) build --manifest-path cookbook/pkgar/Cargo.toml --release
	$(HOST_CARGO) build --manifest-path installer/Cargo.toml --bin list_packages
	mkdir -p build
	touch $@
endif

## The installer and redoxfs run on the host, even when using Podman build
build/fstools: installer redoxfs
	rm -rf $@ $@.partial
	mkdir -p $@.partial
	$(HOST_CARGO) install --root $@.partial --path installer --bin redox_installer
	$(HOST_CARGO) install --root $@.partial --path redoxfs --bin redoxfs --bin redoxfs-mkfs
	mv $@.partial $@
	touch $@

fstools_clean: FORCE $(CONTAINER_TAG)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) $(MAKE) $@
else
	$(HOST_CARGO) clean --manifest-path cookbook/Cargo.toml
	$(HOST_CARGO) clean --manifest-path cookbook/pkgar/Cargo.toml
	$(HOST_CARGO) clean --manifest-path installer/Cargo.toml
	$(HOST_CARGO) clean --manifest-path redoxfs/Cargo.toml
	rm -rf build/fstools
	rm -f $(FSTOOLS_TAG)
endif
