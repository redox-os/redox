# Configuration file for redox-installer, Cookbook and RedoxFS FUSE

fstools: $(FSTOOLS_TAG) $(FSTOOLS)

# These tools run inside Podman if it is used, or on the host if Podman is not used
$(FSTOOLS): installer redoxfs $(CONTAINER_TAG)
ifeq ($(PODMAN_BUILD),1)
ifeq ($(FSTOOLS_IN_PODMAN),1)
	$(PODMAN_RUN) make $@
else
	$(MAKE) $@ PODMAN_BUILD=0 SKIP_CHECK_TOOLS=1
endif
else
	rm -rf $@ $@.partial
	mkdir -p $@.partial
	$(HOST_CARGO) install --root $@.partial --path installer --bin redox_installer -Zgit=shallow-deps
	$(HOST_CARGO) install --root $@.partial --path redoxfs --bin redoxfs --bin redoxfs-mkfs --bin redoxfs-resize
	mv $@.partial $@
	touch $@
endif

$(FSTOOLS_TAG): cookbook $(FSTOOLS)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	$(HOST_CARGO) build --manifest-path cookbook/Cargo.toml --release
	$(HOST_CARGO) build --manifest-path cookbook/pkgar/Cargo.toml --release
	touch $@
endif

fstools_clean: FORCE
	rm -rf cookbook/target
	rm -rf cookbook/pkgar/target
	rm -rf installer/target
	rm -rf redoxfs/target
	rm -rf $(FSTOOLS)
	rm -f $(FSTOOLS_TAG)
