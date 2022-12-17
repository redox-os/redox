$(BUILD)/fetch.tag: cookbook installer prefix $(FILESYSTEM_CONFIG) $(CONTAINER_TAG)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) $(MAKE) $@
else
	$(HOST_CARGO) build --manifest-path cookbook/Cargo.toml --release
	$(HOST_CARGO) build --manifest-path installer/Cargo.toml --release
	PACKAGES="$$($(INSTALLER) --list-packages -c $(FILESYSTEM_CONFIG))" && \
	cd cookbook && \
	./fetch.sh "$${PACKAGES}"
	mkdir -p $(BUILD)
	touch $@
endif

$(BUILD)/repo.tag: $(BUILD)/fetch.tag $(CONTAINER_TAG)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) $(MAKE) $@
else
	$(HOST_CARGO) build --manifest-path cookbook/Cargo.toml --release
	$(HOST_CARGO) build --manifest-path installer/Cargo.toml --release
	export PATH="$(PREFIX_PATH):$$PATH" && \
	PACKAGES="$$($(INSTALLER) --list-packages -c $(FILESYSTEM_CONFIG))" && \
	cd cookbook && \
	./repo.sh "$${PACKAGES}"
	mkdir -p $(BUILD)
	# make sure fetch.tag is newer than the things repo modifies
	touch $<
	touch $@
endif

# Invoke repo.sh for a single target
r.%: FORCE
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) $(MAKE) $@
else
	export PATH="$(PREFIX_PATH):$$PATH" && \
	cd cookbook && \
	./repo.sh $*
endif
