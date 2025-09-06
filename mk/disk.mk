# Configuration file with the commands configuration of the Redox image

$(BUILD)/harddrive.img: $(HOST_FSTOOLS) $(REPO_TAG)
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

$(BUILD)/redox-live.iso: $(HOST_FSTOOLS) $(REPO_TAG) redox.ipxe
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

$(BUILD)/filesystem.img: $(HOST_FSTOOLS) $(REPO_TAG)
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

mount: $(HOST_FSTOOLS) FORCE
	mkdir -p $(MOUNT_DIR)
	$(REDOXFS) $(BUILD)/harddrive.img $(MOUNT_DIR)
	sleep 2
	pgrep redoxfs

mount_extra: $(HOST_FSTOOLS) FORCE
	mkdir -p $(MOUNT_DIR)
	$(REDOXFS) $(BUILD)/extra.img $(MOUNT_DIR)
	sleep 2
	pgrep redoxfs

unmount: FORCE
	sync
	-$(FUMOUNT) $(MOUNT_DIR) || true
	rm -rf $(MOUNT_DIR)
	-$(FUMOUNT) /tmp/redox_installer || true
