# Configuration file for redox-installer, Cookbook and RedoxFS FUSE

fstools: $(FSTOOLS_TAG) $(FSTOOLS)

# These tools run inside Podman if it is used, or on the host if Podman is not used
$(FSTOOLS): cookbook $(CONTAINER_TAG)
ifeq ($(PODMAN_BUILD),1)
ifeq ($(FSTOOLS_IN_PODMAN),1)
	$(PODMAN_RUN) make $@
else
	$(MAKE) $@ PODMAN_BUILD=0 SKIP_CHECK_TOOLS=1
endif
else
	rm -rf $@ $@.partial
	mkdir -p $@.partial
	ln -sr cookbook/recipes $@.partial/recipes

	# Install cookbook, installer, and redoxfs for host (may be outside of podman container)
	#TODO: Build and install installer and redoxfs using cookbook?
	export CARGO_TARGET_DIR=$@-target && \
		$(HOST_CARGO) install --root $@.partial --path cookbook && \
		cd $@.partial && \
		./bin/repo fetch installer redoxfs && \
		cd ../.. && \
		$(HOST_CARGO) install --root $@.partial --path cookbook/recipes/core/installer/source && \
		$(HOST_CARGO) install --root $@.partial --path cookbook/recipes/core/redoxfs/source

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
	rm -rf $(FSTOOLS)
	rm -rf $(FSTOOLS)-target
	rm -f $(FSTOOLS_TAG)
