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

comma := ,

# Invoke clean.sh for one or more targets separated by comma
c.%: $(FSTOOLS_TAG) FORCE
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) $(MAKE) $@
else
	@if echo "$*" | grep -q ','; then \
		$(MAKE) $(foreach f,$(subst $(comma), ,$*),c.$(f)); \
	else \
		export PATH="$(PREFIX_PATH):$$PATH" && \
		export COOKBOOK_HOST_SYSROOT="$(ROOT)/$(PREFIX_INSTALL)" && \
		cd cookbook && \
		./clean.sh $*; \
	fi
endif

# Invoke fetch.sh for one or more targets separated by comma
f.%: $(FSTOOLS_TAG) FORCE
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) $(MAKE) $@
else
	@if echo "$*" | grep -q ','; then \
		$(MAKE) $(foreach f,$(subst $(comma), ,$*),f.$(f)); \
	else \
		export PATH="$(PREFIX_PATH):$$PATH" && \
		export COOKBOOK_HOST_SYSROOT="$(ROOT)/$(PREFIX_INSTALL)" && \
		cd cookbook && \
		./fetch.sh $*; \
	fi
endif

# Invoke repo.sh for one or more targets separated by comma
r.%: $(FSTOOLS_TAG) FORCE
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) $(MAKE) $@
else
	@if echo "$*" | grep -q ','; then \
		$(MAKE) $(foreach f,$(subst $(comma), ,$*),r.$(f)); \
	else \
		export PATH="$(PREFIX_PATH):$$PATH" && \
		export COOKBOOK_HOST_SYSROOT="$(ROOT)/$(PREFIX_INSTALL)" && \
		cd cookbook && \
		./repo.sh $*; \
	fi
endif

# Invoke unfetch.sh for one or more targets separated by comma
u.%: $(FSTOOLS_TAG) FORCE
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) $(MAKE) $@
else
	@if echo "$*" | grep -q ','; then \
		$(MAKE) $(foreach f,$(subst $(comma), ,$*),u.$(f)); \
	else \
		export PATH="$(PREFIX_PATH):$$PATH" && \
		export COOKBOOK_HOST_SYSROOT="$(ROOT)/$(PREFIX_INSTALL)" && \
		cd cookbook && \
		./unfetch.sh $*; \
	fi
endif

# Invoke clean.sh, and repo.sh for one of more targets separated by comma
cr.%: $(FSTOOLS_TAG) FORCE
	$(MAKE) c.$*
	$(MAKE) r.$*

# Invoke unfetch.sh, clean.sh, and repo.sh for one or more targets separated by comma
ucr.%: $(FSTOOLS_TAG) FORCE
	$(MAKE) u.$*
	$(MAKE) cr.$*

# Invoke unfetch.sh and clean.sh for one or more targets separated by comma
uc.%: $(FSTOOLS_TAG) FORCE
	$(MAKE) u.$*
	$(MAKE) c.$*

# Invoke unfetch, clean.sh and fetch.sh for one or more targets separated by comma
ucf.%: $(FSTOOLS_TAG) FORCE
	$(MAKE) uc.$*
	$(MAKE) f.$*
