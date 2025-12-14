# Configuration file for redox-installer, Cookbook and RedoxFS FUSE

fstools: $(FSTOOLS_TAG) $(FSTOOLS)

# These tools run inside Podman if it is used, or on the host if Podman is not used
$(FSTOOLS): $(CONTAINER_TAG)
ifeq ($(PODMAN_BUILD),1)
ifeq ($(FSTOOLS_IN_PODMAN),1)
	$(PODMAN_RUN) make $@
else
	$(MAKE) $@ PODMAN_BUILD=0 SKIP_CHECK_TOOLS=1
endif
else
	rm -rf $@ $@.partial
	mkdir -p $@.partial
	ln -sr recipes $@.partial/recipes

	# Install cookbook, installer, and redoxfs for host (may be outside of podman container)
	#TODO: Build and install installer and redoxfs using cookbook?
	cd $@.partial && \
		export CARGO_TARGET_DIR=../$@-target && \
		$(HOST_CARGO) install --root . --path ../.. --locked && \
		env -u RUSTUP_TOOLCHAIN ./bin/repo fetch installer redoxfs && \
		$(HOST_CARGO) install --root . --path recipes/core/installer/source && \
		$(HOST_CARGO) install --root . --path recipes/core/redoxfs/source

	mv $@.partial $@
	touch $@
endif

$(FSTOOLS_TAG): $(FSTOOLS)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	$(HOST_CARGO) build --manifest-path Cargo.toml --release --locked
	touch $@
endif

fstools_clean: FORCE
	rm -rf target
	rm -rf $(FSTOOLS)
	rm -rf $(FSTOOLS)-target
	rm -f $(FSTOOLS_TAG)
