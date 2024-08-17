# Configuration file for redox-installer, Cookbook and RedoxFS FUSE

fstools: $(FSTOOLS_TAG)

## The installer runs on the host, even when using Podman build
$(FSTOOLS_TAG): cookbook installer redoxfs $(CONTAINER_TAG)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) $(MAKE) $@
else
	$(HOST_CARGO) build --manifest-path cookbook/Cargo.toml --release
	$(HOST_CARGO) build --bin list_packages --manifest-path installer/Cargo.toml --release
	$(HOST_CARGO) build --manifest-path redoxfs/Cargo.toml --release --bin redoxfs --bin redoxfs-mkfs
	mkdir -p build
	touch $@
endif

$(INSTALLER): installer
	$(HOST_CARGO) build --bin redox_installer --manifest-path installer/Cargo.toml --release

fstools_clean: FORCE $(CONTAINER_TAG)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) $(MAKE) $@
else
	$(HOST_CARGO) clean --manifest-path cookbook/Cargo.toml
	$(HOST_CARGO) clean --manifest-path installer/Cargo.toml
	$(HOST_CARGO) clean --manifest-path redoxfs/Cargo.toml
	rm -f $(FSTOOLS_TAG)
endif
