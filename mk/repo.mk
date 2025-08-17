# Configuration file for recipe commands

$(BUILD)/fetch.tag: prefix $(FSTOOLS_TAG) $(FILESYSTEM_CONFIG) $(CONTAINER_TAG)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	export PATH="$(PREFIX_PATH):$$PATH" && \
	PACKAGES="$$($(LIST_PACKAGES) $(LIST_PACKAGES_OPTS) -c $(FILESYSTEM_CONFIG))" && \
	export COOKBOOK_HOST_SYSROOT="$(ROOT)/$(PREFIX_INSTALL)" && \
	cd cookbook && \
	./fetch.sh $(REPO_NONSTOP) $(REPO_OFFLINE) "$${PACKAGES}"
	mkdir -p $(BUILD)
	touch $@
endif

$(REPO_TAG): $(BUILD)/fetch.tag $(FSTOOLS_TAG) $(CONTAINER_TAG)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	export PATH="$(PREFIX_PATH):$$PATH" && \
	export COOKBOOK_HOST_SYSROOT="$(ROOT)/$(PREFIX_INSTALL)" && \
	PACKAGES="$$($(LIST_PACKAGES) $(LIST_PACKAGES_OPTS) -c $(FILESYSTEM_CONFIG))" && \
	cd cookbook && \
	./repo.sh $(REPO_NONSTOP) $(REPO_OFFLINE) --with-package-deps "$${PACKAGES}"
	mkdir -p $(BUILD)
	# make sure fstools.tag and fetch.tag are newer than the things repo modifies
	touch $(FSTOOLS_TAG)
	touch $(BUILD)/fetch.tag
	touch $@
endif

# Find recipe
find.%: $(FSTOOLS_TAG) FORCE
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
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
	$(PODMAN_RUN) make $@
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
	$(PODMAN_RUN) make $@
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
	$(PODMAN_RUN) make $@
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
	$(PODMAN_RUN) make $@
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
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	$(MAKE) c.$*
	$(MAKE) r.$*
endif

# Invoke unfetch.sh, clean.sh, and repo.sh for one or more targets separated by comma
ucr.%: $(FSTOOLS_TAG) FORCE
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	$(MAKE) u.$*
	$(MAKE) cr.$*
endif

# Invoke unfetch.sh and clean.sh for one or more targets separated by comma
uc.%: $(FSTOOLS_TAG) FORCE
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	$(MAKE) u.$*
	$(MAKE) c.$*
endif

# Invoke unfetch, clean.sh and fetch.sh for one or more targets separated by comma
ucf.%: $(FSTOOLS_TAG) FORCE
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	$(MAKE) uc.$*
	$(MAKE) f.$*
endif


export DEBUG_BIN?=

# Debug a recipe with gdbgui inside podman, for example: debug.drivers-initfs DEBUG_BIN=pcid
# Please set REPO_DEBUG=1 to your .config to enable debug symbols and run `make cr.recipe rebuild`
# Also, before opening gdbgui at http://localhost:5000, start qemu with `make qemu gdb=yes`
# Experimental and may not work if ARCH is different with what podman is running
debug.%: $(FSTOOLS_TAG) FORCE
	@cd cookbook/$(shell make find.$* | grep ^recipes) && \
		export RECIPE_STAGE=target/$(TARGET)/stage && \
		export BIN_PATH=$$(find $$RECIPE_STAGE -type f -name "$(DEBUG_BIN)" -or -type f -name "$*") && \
		file $$BIN_PATH 2> /dev/null || ( echo "Binary is not found, please set DEBUG_BIN" && exit 1 ) && \
		echo "Opening gdbgui for debugging $* with binary '$$BIN_PATH'" && echo "----------" && \
		podman build -t redox-kernel-debug - < $(ROOT)/podman/redox-gdb-containerfile > /dev/null && \
		podman run --rm -p 5000:5000 -it --name redox-gdb \
		-v "./$$BIN_PATH:/binary" \
		-v "./source:/source" -w "/source" \
		redox-kernel-debug --gdb-cmd "gdb -ex 'set confirm off' \
			-ex 'add-symbol-file /binary' \
			-ex 'target remote host.containers.internal:1234'"
