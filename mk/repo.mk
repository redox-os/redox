# Configuration file for recipe commands

$(REPO_TAG): prefix $(FILESYSTEM_CONFIG) | $(FSTOOLS) $(FSTOOLS_TAG) $(CONTAINER_TAG)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	export PATH="$(PREFIX_PATH):$$PATH" && \
	export COOKBOOK_HOST_SYSROOT="$(ROOT)/$(PREFIX_INSTALL)" && \
	$(REPO_BIN) cook $(COOKBOOK_OPTS) --with-package-deps
	mkdir -p $(BUILD)
	touch $@
endif

comma := ,

# List all recipes in a cook-tree fashion specified by the filesystem config
repo-tree: $(FSTOOLS_TAG) $(CONTAINER_TAG)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	@$(REPO_BIN) cook-tree $(COOKBOOK_OPTS) --with-package-deps
endif

# List all recipes in a push-tree fashion specified by the filesystem config
image-tree: $(FSTOOLS_TAG) $(CONTAINER_TAG)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	@$(REPO_BIN) push-tree $(COOKBOOK_OPTS) --with-package-deps
endif

# Fetch all recipes source or binary from filesystem config
fetch: $(FSTOOLS_TAG) FORCE
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	export PATH="$(PREFIX_PATH):$$PATH" && \
	export COOKBOOK_HOST_SYSROOT="$(ROOT)/$(PREFIX_INSTALL)" && \
	$(REPO_BIN) fetch $(COOKBOOK_OPTS) --with-package-deps
endif

# Find recipe for one or more targets separated by comma
find.%: $(FSTOOLS_TAG) FORCE
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	@$(REPO_BIN) find $(foreach f,$(subst $(comma), ,$*),$(f))
endif

# Invoke clean for relibc in recipe and relibc in sysroot
c.relibc: $(FSTOOLS_TAG) FORCE
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	$(REPO_BIN) clean relibc
	rm -rf $(PREFIX)/relibc-install $(PREFIX)/sysroot $(REPO_TAG)
	@echo "\033[1;36;49mSysroot cleaned\033[0m"
endif

# Invoke clean for one or more targets separated by comma
c.%: $(FSTOOLS_TAG) FORCE
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	$(REPO_BIN) clean $(foreach f,$(subst $(comma), ,$*),$(f))
endif

# Invoke fetch for one or more targets separated by comma
f.%: $(FSTOOLS_TAG) FORCE
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	export PATH="$(PREFIX_PATH):$$PATH" && \
	export COOKBOOK_HOST_SYSROOT="$(ROOT)/$(PREFIX_INSTALL)" && \
	$(REPO_BIN) fetch $(foreach f,$(subst $(comma), ,$*),$(f)) $(COOKBOOK_OPTS)
endif

# Invoke cook for one or more targets separated by comma
r.%: prefix $(FSTOOLS_TAG) FORCE
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	export PATH="$(PREFIX_PATH):$$PATH" && \
	export COOKBOOK_HOST_SYSROOT="$(ROOT)/$(PREFIX_INSTALL)" && \
	$(REPO_BIN) cook $(foreach f,$(subst $(comma), ,$*),$(f)) $(COOKBOOK_OPTS)
endif

# Show what to cook
rt.%: $(FSTOOLS_TAG) FORCE
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	$(REPO_BIN) cook-tree $(foreach f,$(subst $(comma), ,$*),$(f)) $(COOKBOOK_OPTS)
endif

MOUNTED_TAG=$(MOUNT_DIR)~

# Push compiled package into existing image
# DO NOT RUN THIS WHILE QEMU ALIVE, THE DISK MIGHT CORRUPT IN DOING SO
p.%: $(FSTOOLS_TAG) FORCE
ifeq ($(ALLOW_FSTOOLS),1)
	@rm -f $(MOUNTED_TAG)
	@if [ ! -d "$(MOUNT_DIR)" ]; then \
		$(MAKE) mount; \
		touch $(MOUNTED_TAG); \
	fi
endif
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@ ALLOW_FSTOOLS=$(FSTOOLS_IN_PODMAN)
else
	$(REPO_BIN) push $(foreach f,$(subst $(comma), ,$*),$(f)) "--sysroot=$(MOUNT_DIR)"
endif
ifeq ($(ALLOW_FSTOOLS),1)
	@if [ -f $(MOUNTED_TAG) ]; then \
		$(MAKE) unmount && rm -f $(MOUNTED_TAG); \
	else echo "\033[0;33;49mNot unmounting by ourself, don't forget to do it\033[0m"; \
	fi
endif

# Push compiled package with their package dependencies
pp.%: $(FSTOOLS_TAG) FORCE
	$(MAKE) p.$*,--with-package-deps

# Show what to push
pt.%: $(FSTOOLS_TAG) FORCE
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	$(REPO_BIN) push-tree $(foreach f,$(subst $(comma), ,$*),$(f)) $(COOKBOOK_OPTS)
endif

# Show what to push (with deps)
ppt.%: prefix $(FSTOOLS_TAG) FORCE
	$(MAKE) pt.$*,--with-package-deps

# Push all recipes specified by the filesystem config
push: $(FSTOOLS_TAG) FORCE
ifeq ($(ALLOW_FSTOOLS),1)
	@rm -f $(MOUNTED_TAG)
	@if [ ! -d "$(MOUNT_DIR)" ]; then \
		$(MAKE) mount; \
		touch $(MOUNTED_TAG); \
	fi
endif
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@ ALLOW_FSTOOLS=$(FSTOOLS_IN_PODMAN)
else
	$(REPO_BIN) push $(COOKBOOK_OPTS) --with-package-deps "--sysroot=$(MOUNT_DIR)"
endif
ifeq ($(ALLOW_FSTOOLS),1)
	@if [ -f $(MOUNTED_TAG) ]; then \
		$(MAKE) unmount && rm -f $(MOUNTED_TAG); \
	else echo "\033[1;33;49mNot unmounting by ourself, don't forget to do it\033[0m"; \
	fi
endif

# Invoke unfetch for one or more targets separated by comma
u.%: $(FSTOOLS_TAG) FORCE
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	$(REPO_BIN) unfetch $(foreach f,$(subst $(comma), ,$*),$(f))
endif

# Invoke clean, and repo.sh for one of more targets separated by comma
cr.%: $(FSTOOLS_TAG) FORCE
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	$(MAKE) c.$*
	$(MAKE) r.$*
endif

# Invoke unfetch, clean, and repo.sh for one or more targets separated by comma
ucr.%: $(FSTOOLS_TAG) FORCE
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	$(MAKE) u.$*
	$(MAKE) cr.$*
endif

# Invoke unfetch and clean for one or more targets separated by comma
uc.%: $(FSTOOLS_TAG) FORCE
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	$(MAKE) u.$*
	$(MAKE) c.$*
endif

# Invoke unfetch, clean and fetch for one or more targets separated by comma
ucf.%: $(FSTOOLS_TAG) FORCE
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	$(MAKE) uc.$*
	$(MAKE) f.$*
endif

# Invoke repo.sh and push for one of more targets separated by comma
# Don't use podman here, as the p target cannot mount inside podman
rp.%: $(FSTOOLS_TAG) FORCE
	$(MAKE) r.$*
	$(MAKE) p.$*

# Invoke clean, repo.sh and push for one of more targets separated by comma
crp.%: $(FSTOOLS_TAG) FORCE
	$(MAKE) cr.$*
	$(MAKE) p.$*

# Invoke unfetch. clean, repo.sh and push for one of more targets separated by comma
ucrp.%: $(FSTOOLS_TAG) FORCE
	$(MAKE) ucr.$*
	$(MAKE) p.$*

export DEBUG_BIN?=

# Debug a statically linked program with gdbgui, for example: debug.drivers-initfs DEBUG_BIN=pcid
# Enable debug symbols with `REPO_DEBUG=1 make cr.recipe rebuild`, make sure `file` outputs "debug_info, not stripped"
# Open http://localhost:5000/dashboard, start QEMU with `make qemu kvm=no QEMU_SMP=1 gdb=yes` before opening a session
# Experimental, may not work if ARCH is different with what host is running
debug.%: $(FSTOOLS_TAG) FORCE
ifeq ($(PODMAN_BUILD),1)
	@cd $(shell make find.$* | grep ^recipes) && \
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
else
	@cd $(shell make find.$* | grep ^recipes) && \
		export RECIPE_STAGE=target/$(TARGET)/stage && \
		export BIN_PATH=$$(find $$RECIPE_STAGE -type f -name "$(DEBUG_BIN)" -or -type f -name "$*") && \
		file $$BIN_PATH 2> /dev/null || ( echo "Binary is not found, please set DEBUG_BIN" && exit 1 ) && \
		echo "Opening gdbgui for debugging $* with binary '$$BIN_PATH'" && echo "----------" && \
		gdbgui.pex --gdb-cmd "gdb -ex 'set confirm off' \
			-ex 'add-symbol-file $$BIN_PATH' \
			-ex 'target remote localhost:1234'"
endif
