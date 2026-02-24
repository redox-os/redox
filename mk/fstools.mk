# Configuration file for redox-installer, Cookbook and RedoxFS FUSE

fstools: $(FSTOOLS_TAG) $(FSTOOLS)

GOING_TO_PODMAN_AGAIN?=0

# These tools run inside Podman if it is used, or on the host if Podman is not used
$(FSTOOLS): | prefix $(CONTAINER_TAG) $(FSTOOLS_TAG)
ifeq ($(PODMAN_BUILD),1)
ifeq ($(FSTOOLS_IN_PODMAN),1)
	$(PODMAN_RUN) make $@
else
	$(MAKE) $@ PODMAN_BUILD=0 SKIP_CHECK_TOOLS=1 GOING_TO_PODMAN_AGAIN=1
endif
else
	rm -rf $@ $@.partial
	mkdir -p $@.partial
	ln -s ../../recipes $@.partial/recipes
	$(MAKE) fstools_fetch PODMAN_BUILD=$(GOING_TO_PODMAN_AGAIN)

	# Compile installer and redoxfs for host (may be outside of podman container)
	cd $@.partial && \
		export CARGO_TARGET_DIR=../$@-target && \
		$(HOST_CARGO) install --root . --path recipes/core/installer/source $(INSTALLER_FEATURES) && \
		$(HOST_CARGO) install --root . --path recipes/core/redoxfs/source $(REDOXFS_FEATURES)

	mv $@.partial $@
	touch $@
endif

fstools_fetch: $(FSTOOLS_TAG) FORCE
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	./target/release/repo fetch installer redoxfs
endif

$(FSTOOLS_TAG): $(CONTAINER_TAG)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	$(HOST_CARGO) build --manifest-path Cargo.toml --release --locked
	mkdir -p $(@D)
	touch $@
endif

fstools_clean: FORCE
	rm -rf target
	rm -rf $(FSTOOLS)
	rm -rf $(FSTOOLS)-target
	rm -f $(FSTOOLS_TAG)
