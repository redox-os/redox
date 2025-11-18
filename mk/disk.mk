# Configuration file with the commands configuration of the Redox image

$(BUILD)/harddrive.img: $(HOST_FSTOOLS) $(REPO_TAG)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	mkdir -p $(BUILD)
	rm -rf $@  $@.partial
	-$(FUMOUNT) /tmp/redox_installer || true
	FILESYSTEM_SIZE=$(FILESYSTEM_SIZE) && \
	if [ -z "$$FILESYSTEM_SIZE" ] ; then \
	FILESYSTEM_SIZE=$(shell $(INSTALLER) --filesystem-size -c $(FILESYSTEM_CONFIG)); \
	fi && \
	truncate -s "$$FILESYSTEM_SIZE"m $@.partial
	umask 002 && $(INSTALLER) $(INSTALLER_OPTS) -c $(FILESYSTEM_CONFIG) $@.partial
	mv $@.partial $@
endif

$(BUILD)/redox-live.iso: $(HOST_FSTOOLS) $(REPO_TAG) redox.ipxe
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	mkdir -p $(BUILD)
	rm -rf $@  $@.partial
	-$(FUMOUNT) /tmp/redox_installer || true
	FILESYSTEM_SIZE=$(FILESYSTEM_SIZE) && \
	if [ -z "$$FILESYSTEM_SIZE" ] ; then \
	FILESYSTEM_SIZE=$(shell $(INSTALLER) --filesystem-size -c $(FILESYSTEM_CONFIG)); \
	fi && \
	truncate -s "$$FILESYSTEM_SIZE"m $@.partial
	umask 002 && $(INSTALLER) $(INSTALLER_OPTS) -c $(FILESYSTEM_CONFIG) --write-bootloader="$(BUILD)/bootloader-live.efi" --live $@.partial
	mv $@.partial $@
	cp redox.ipxe $(BUILD)/redox.ipxe
endif

$(BUILD)/filesystem.img: $(HOST_FSTOOLS) $(REPO_TAG)
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	mkdir -p $(BUILD)
	-$(FUMOUNT) $(MOUNT_DIR) || true
	rm -rf $@  $@.partial $(MOUNT_DIR)
	-$(FUMOUNT) /tmp/redox_installer || true
	FILESYSTEM_SIZE=$(FILESYSTEM_SIZE) && \
	if [ -z "$$FILESYSTEM_SIZE" ] ; then \
	FILESYSTEM_SIZE=$(shell $(INSTALLER) --filesystem-size -c $(FILESYSTEM_CONFIG)); \
	fi && \
	truncate -s "$$FILESYSTEM_SIZE"m $@.partial
	$(REDOXFS_MKFS) $(REDOXFS_MKFS_FLAGS) $@.partial
	mkdir -p $(MOUNT_DIR)
	$(REDOXFS) $@.partial $(MOUNT_DIR)
	sleep 1
	pgrep redoxfs
	umask 002 && $(INSTALLER) $(INSTALLER_OPTS) -c $(FILESYSTEM_CONFIG) $(MOUNT_DIR)
	sync
	-$(FUMOUNT) $(MOUNT_DIR) || true
	rm -rf $(MOUNT_DIR)
	mv $@.partial $@
endif

DAEMONIZE_MOUNT=0

mount: $(HOST_FSTOOLS) FORCE
ifeq ($(PODMAN_BUILD),1)
	podman unshare mount --make-shared --bind $(ROOT) $(ROOT)
	touch $(MOUNT_DIR).tag
	$(PODMAN_RUN_DETACHED) make $@ DAEMONIZE_MOUNT=1
	sleep 2
else
	mkdir -p $(MOUNT_DIR)
	$(REDOXFS) $(BUILD)/harddrive.img $(MOUNT_DIR)
ifeq ($(DAEMONIZE_MOUNT),1)
	@while test -f $(MOUNT_DIR).tag; do \
		sleep 1; \
	done
	make unmount
else
	sleep 2
	pgrep redoxfs
endif
endif

mount_extra: $(HOST_FSTOOLS) FORCE
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	mkdir -p $(MOUNT_DIR)
	$(REDOXFS) $(BUILD)/extra.img $(MOUNT_DIR)
	sleep 2
	pgrep redoxfs
endif

mount_live: $(HOST_FSTOOLS) FORCE
ifeq ($(PODMAN_BUILD),1)
	$(PODMAN_RUN) make $@
else
	mkdir -p $(MOUNT_DIR)
	$(REDOXFS) $(BUILD)/redox-live.iso $(MOUNT_DIR)
	sleep 2
	pgrep redoxfs
endif

unmount: FORCE
ifeq ($(PODMAN_BUILD),1)
	rm -f $(MOUNT_DIR).tag
else
	sync
	-$(FUMOUNT) $(MOUNT_DIR) || true
	rm -rf $(MOUNT_DIR)
	-$(FUMOUNT) /tmp/redox_installer || true
endif
