# Configuration file for recipe commands

$(BUILD)/fetch.tag: prefix $(FSTOOLS_TAG) $(FILESYSTEM_CONFIG) $(CONTAINER_TAG)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) $(MAKE) $@
else
	export PATH="$(PREFIX_PATH):$$PATH" && \
	PACKAGES="$$($(LIST_PACKAGES) $(LIST_PACKAGES_OPTS) -c $(FILESYSTEM_CONFIG))" && \
	export COOKBOOK_HOST_SYSROOT="$(ROOT)/$(PREFIX_INSTALL)" && \
	cd cookbook && \
	./fetch.sh "$${PACKAGES}"
	mkdir -p $(BUILD)
	touch $@
endif

$(REPO_TAG): $(BUILD)/fetch.tag $(FSTOOLS_TAG) $(CONTAINER_TAG)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) $(MAKE) $@
else
	export PATH="$(PREFIX_PATH):$$PATH" && \
	export COOKBOOK_HOST_SYSROOT="$(ROOT)/$(PREFIX_INSTALL)" && \
	PACKAGES="$$($(LIST_PACKAGES) $(LIST_PACKAGES_OPTS) -c $(FILESYSTEM_CONFIG))" && \
	cd cookbook && \
	./repo.sh $(REPO_NONSTOP) "$${PACKAGES}"
	mkdir -p $(BUILD)
	# make sure fstools.tag and fetch.tag are newer than the things repo modifies
	touch $(FSTOOLS_TAG)
	touch $(BUILD)/fetch.tag
	touch $@
endif

# Find recipe
find.%: $(FSTOOLS_TAG) FORCE
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) $(MAKE) $@
else
	export PATH="$(PREFIX_PATH):$$PATH" && \
	export COOKBOOK_HOST_SYSROOT="$(ROOT)/$(PREFIX_INSTALL)" && \
	cd cookbook && \
	target/release/find_recipe $*
endif

# Invoke clean.sh for a single target
c.%: $(FSTOOLS_TAG) FORCE
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) $(MAKE) $@
else
	export PATH="$(PREFIX_PATH):$$PATH" && \
	export COOKBOOK_HOST_SYSROOT="$(ROOT)/$(PREFIX_INSTALL)" && \
	cd cookbook && \
	./clean.sh $*
endif

comma := ,

# Invoke clean.sh for multiple targets
cl.%: $(FSTOOLS_TAG) FORCE
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) $(MAKE) $@
else
	$(MAKE) $(foreach target,$(subst $(comma), ,$*),c.$(target))
endif

# Invoke fetch.sh for a single target
f.%: $(FSTOOLS_TAG) FORCE
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) $(MAKE) $@
else
	export PATH="$(PREFIX_PATH):$$PATH" && \
	export COOKBOOK_HOST_SYSROOT="$(ROOT)/$(PREFIX_INSTALL)" && \
	cd cookbook && \
	./fetch.sh $*
endif

# Invoke fetch.sh for multiple targets
fl.%: $(FSTOOLS_TAG) FORCE
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) $(MAKE) $@
else
	$(MAKE) $(foreach target,$(subst $(comma), ,$*),f.$(target))
endif

# Invoke repo.sh for a single target
r.%: $(FSTOOLS_TAG) FORCE
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) $(MAKE) $@
else
	export PATH="$(PREFIX_PATH):$$PATH" && \
	export COOKBOOK_HOST_SYSROOT="$(ROOT)/$(PREFIX_INSTALL)" && \
	cd cookbook && \
	./repo.sh $*
endif

# Invoke repo.sh for multiple targets
rl.%: $(FSTOOLS_TAG) FORCE
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) $(MAKE) $@
else
	$(MAKE) $(foreach target,$(subst $(comma), ,$*),r.$(target))
endif

# Invoke unfetch.sh for a single target
u.%: $(FSTOOLS_TAG) FORCE
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) $(MAKE) $@
else
	export PATH="$(PREFIX_PATH):$$PATH" && \
	export COOKBOOK_HOST_SYSROOT="$(ROOT)/$(PREFIX_INSTALL)" && \
	cd cookbook && \
	./unfetch.sh $*
endif

# Invoke unfetch.sh for multiple targets
ul.%: $(FSTOOLS_TAG) FORCE
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) $(MAKE) $@
else
	$(MAKE) $(foreach target,$(subst $(comma), ,$*),u.$(target))
endif

# Invoke clean.sh, and repo.sh for a single target
cr.%: $(FSTOOLS_TAG) FORCE
	$(MAKE) c.$*
	$(MAKE) r.$*

# Invoke clean.sh and repo.sh for multiple targets
crl.%: $(FSTOOLS_TAG) FORCE
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) $(MAKE) $@
else
	$(MAKE) $(foreach target,$(subst $(comma), ,$*),c.$(target))
	$(MAKE) $(foreach target,$(subst $(comma), ,$*),r.$(target))
endif

# Invoke unfetch.sh, clean.sh, and repo.sh for a single target
ucr.%: $(FSTOOLS_TAG) FORCE
	$(MAKE) u.$*
	$(MAKE) cr.$*

# Invoke unfetch.sh, clean.sh and repo.sh for multiple targets
ucrl.%: $(FSTOOLS_TAG) FORCE
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) $(MAKE) $@
else
	$(MAKE) $(foreach target,$(subst $(comma), ,$*),u.$(target))
	$(MAKE) $(foreach target,$(subst $(comma), ,$*),cr.$(target))
endif

uc.%: $(FSTOOLS_TAG) FORCE
	$(MAKE) u.$*
	$(MAKE) c.$*

# Invoke unfetch.sh and clean.sh for multiple targets
ucl.%: $(FSTOOLS_TAG) FORCE
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) $(MAKE) $@
else
	$(MAKE) $(foreach target,$(subst $(comma), ,$*),u.$(target))
	$(MAKE) $(foreach target,$(subst $(comma), ,$*),c.$(target))
endif

ucf.%: $(FSTOOLS_TAG) FORCE
	$(MAKE) uc.$*
	$(MAKE) f.$*

# Invoke unfetch, clean.sh and fetch.sh for multiple targets
ucfl.%: $(FSTOOLS_TAG) FORCE
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) $(MAKE) $@
else
	$(MAKE) $(foreach target,$(subst $(comma), ,$*),uc.$(target))
	$(MAKE) $(foreach target,$(subst $(comma), ,$*),f.$(target))
endif
