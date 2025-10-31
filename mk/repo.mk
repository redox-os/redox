# Configuration file for recipe commands

$(REPO_TAG): $(FSTOOLS_TAG) $(CONTAINER_TAG)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	export PATH="$(PREFIX_PATH):$$PATH" && \
	export COOKBOOK_HOST_SYSROOT="$(ROOT)/$(PREFIX_INSTALL)" && \
	./cookbook/repo.sh $(REPO_NONSTOP) $(REPO_OFFLINE) --with-package-deps "--filesystem=../$(FILESYSTEM_CONFIG)"
	mkdir -p $(BUILD)
	# make sure fstools.tag are newer than the things repo modifies
	touch $(FSTOOLS_TAG)
	touch $@
endif

comma := ,

# List all recipes in a tree fashion specified by the filesystem config
tree: $(FSTOOLS_TAG) $(CONTAINER_TAG)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	@cd ./cookbook && ./target/release/repo tree --with-package-deps "--filesystem=../$(FILESYSTEM_CONFIG)"
endif

# Find recipe for one or more targets separated by comma
find.%: $(FSTOOLS_TAG) FORCE
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	@cd ./cookbook && ./target/release/repo find $(foreach f,$(subst $(comma), ,$*),$(f))
endif

# Invoke clean for one or more targets separated by comma
c.%: $(FSTOOLS_TAG) FORCE
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	cd ./cookbook && ./target/release/repo clean $(foreach f,$(subst $(comma), ,$*),$(f))
endif

# Invoke fetch for one or more targets separated by comma
f.%: $(FSTOOLS_TAG) FORCE
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	export PATH="$(PREFIX_PATH):$$PATH" && \
	export COOKBOOK_HOST_SYSROOT="$(ROOT)/$(PREFIX_INSTALL)" && \
	cd ./cookbook && ./target/release/repo fetch $(foreach f,$(subst $(comma), ,$*),$(f))
endif

# Invoke repo.sh for one or more targets separated by comma
r.%: $(FSTOOLS_TAG) FORCE
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	export PATH="$(PREFIX_PATH):$$PATH" && \
	export COOKBOOK_HOST_SYSROOT="$(ROOT)/$(PREFIX_INSTALL)" && \
	./cookbook/repo.sh $(REPO_OFFLINE) $(foreach f,$(subst $(comma), ,$*),$(f))
endif

MOUNTED_TAG=$(MOUNT_DIR)~

# Push compiled package into existing image
# DO NOT RUN THIS WHILE QEMU ALIVE, THE DISK MIGHT CORRUPT IN DOING SO
p.%: $(FSTOOLS_TAG) FORCE
	@rm -f $(MOUNTED_TAG)
	@if [ ! -d "$(MOUNT_DIR)" ]; then \
		$(MAKE) mount; \
		touch $(MOUNTED_TAG); \
	fi
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	cd ./cookbook && ./target/release/repo push $(foreach f,$(subst $(comma), ,$*),$(f)) "--sysroot=../$(MOUNT_DIR)"
endif
	@if [ -f $(MOUNTED_TAG) ]; then \
		$(MAKE) unmount && rm -f $(MOUNTED_TAG); \
	else echo "Not unmounting by ourself, don't forget to do it"; \
	fi

# Push compiled package with their package dependencies
pp.%: $(FSTOOLS_TAG) FORCE
	$(MAKE) p.$*,--with-package-deps

# Push all recipes specified by the filesystem config
push: $(FSTOOLS_TAG) FORCE
	@rm -f $(MOUNTED_TAG)
	@if [ ! -d "$(MOUNT_DIR)" ]; then \
		$(MAKE) mount; \
		touch $(MOUNTED_TAG); \
	fi
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	cd ./cookbook && ./target/release/repo push --with-package-deps "--filesystem=../$(FILESYSTEM_CONFIG)" "--sysroot=../$(MOUNT_DIR)"
endif
	@if [ -f $(MOUNTED_TAG) ]; then \
		$(MAKE) unmount && rm -f $(MOUNTED_TAG); \
	else echo "Not unmounting by ourself, don't forget to do it"; \
	fi

# Invoke unfetch for one or more targets separated by comma
u.%: $(FSTOOLS_TAG) FORCE
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	cd ./cookbook && ./target/release/repo unfetch $(foreach f,$(subst $(comma), ,$*),$(f))
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

# Invoke repo.sh and push for one of more targets separated by comma
rp.%: $(FSTOOLS_TAG) FORCE
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	$(MAKE) r.$*
	$(MAKE) p.$*
endif

# Invoke clean.sh, repo.sh and push for one of more targets separated by comma
crp.%: $(FSTOOLS_TAG) FORCE
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	$(MAKE) c.$*
	$(MAKE) r.$*
	$(MAKE) p.$*
endif

# Invoke unfetch.sh. clean.sh, repo.sh and push for one of more targets separated by comma
ucrp.%: $(FSTOOLS_TAG) FORCE
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	$(MAKE) u.$*
	$(MAKE) c.$*
	$(MAKE) r.$*
	$(MAKE) p.$*
endif

export DEBUG_BIN?=

# Debug a statically linked program with gdbgui, for example: debug.drivers-initfs DEBUG_BIN=pcid
# Enable debug symbols with `REPO_DEBUG=1 make cr.recipe rebuild`, make sure `file` outputs "debug_info, not stripped"
# Open http://localhost:5000/dashboard, start QEMU with `make qemu kvm=no QEMU_SMP=1 gdb=yes` before opening a session
# Experimental, may not work if ARCH is different with what host is running
debug.%: $(FSTOOLS_TAG) FORCE
ifeq ($(PODMAN_BUILD),1)
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
else
	@cd cookbook/$(shell make find.$* | grep ^recipes) && \
		export RECIPE_STAGE=target/$(TARGET)/stage && \
		export BIN_PATH=$$(find $$RECIPE_STAGE -type f -name "$(DEBUG_BIN)" -or -type f -name "$*") && \
		file $$BIN_PATH 2> /dev/null || ( echo "Binary is not found, please set DEBUG_BIN" && exit 1 ) && \
		echo "Opening gdbgui for debugging $* with binary '$$BIN_PATH'" && echo "----------" && \
		gdbgui.pex --gdb-cmd "gdb -ex 'set confirm off' \
			-ex 'add-symbol-file $$BIN_PATH' \
			-ex 'target remote localhost:1234'"
endif
