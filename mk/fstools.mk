# Configuration file for redox-installer, Cookbook and RedoxFS FUSE

fstools: $(FSTOOLS_TAG)

## The installer runs on the host, even when using Podman build
$(FSTOOLS_TAG): cookbook installer redoxfs $(CONTAINER_TAG)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) $(MAKE) $@
else
	rm -rf build/fstools
	mkdir -p build/fstools
	$(HOST_CARGO) build --manifest-path cookbook/Cargo.toml --release
	$(HOST_CARGO) build --manifest-path cookbook/pkgar/Cargo.toml --release
	$(HOST_CARGO) install --root build/fstools --path installer --bin list_packages --bin redox_installer
	$(HOST_CARGO) install --root build/fstools --path redoxfs --bin redoxfs --bin redoxfs-mkfs
	mkdir -p build
	touch $@
endif

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
