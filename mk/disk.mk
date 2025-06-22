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

$(BUILD)/redox-live.iso: $(HOST_FSTOOLS) $(REPO_TAG)
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

$(BUILD)/tftproot: $(HOST_FSTOOLS) $(REPO_TAG) $(BUILD)/redox-live.iso
	rm -r $(BUILD)/tftproot || true
	mkdir $(BUILD)/tftproot
	cp $(BUILD)/bootloader-live.efi $(BUILD)/tftproot/bootloader-live.efi
	ln -s ../redox-live.iso $(BUILD)/tftproot/redox-live.iso
	cp redox.ipxe $(BUILD)/tftproot/redox.ipxe

$(BUILD)/filesystem.img: $(HOST_FSTOOLS) $(REPO_TAG)
	mkdir -p $(BUILD)
	-$(FUMOUNT) $(BUILD)/filesystem/ || true
	rm -rf $@  $@.partial $(BUILD)/filesystem/
	-$(FUMOUNT) /tmp/redox_installer || true
	FILESYSTEM_SIZE=$(FILESYSTEM_SIZE) && \
	if [ -z "$$FILESYSTEM_SIZE" ] ; then \
	FILESYSTEM_SIZE=$(shell $(INSTALLER) --filesystem-size -c $(FILESYSTEM_CONFIG)); \
	fi && \
	truncate -s "$$FILESYSTEM_SIZE"m $@.partial
	$(REDOXFS_MKFS) $(REDOXFS_MKFS_FLAGS) $@.partial
	mkdir -p $(BUILD)/filesystem/
	$(REDOXFS) $@.partial $(BUILD)/filesystem/
	sleep 1
	pgrep redoxfs
	umask 002 && $(INSTALLER) $(INSTALLER_OPTS) -c $(FILESYSTEM_CONFIG) $(BUILD)/filesystem/
	sync
	-$(FUMOUNT) $(BUILD)/filesystem/ || true
	rm -rf $(BUILD)/filesystem/
	mv $@.partial $@

mount: $(HOST_FSTOOLS) FORCE
	mkdir -p $(BUILD)/filesystem/
	$(REDOXFS) $(BUILD)/harddrive.img $(BUILD)/filesystem/
	sleep 2
	pgrep redoxfs

mount_extra: $(HOST_FSTOOLS) FORCE
	mkdir -p $(BUILD)/filesystem/
	$(REDOXFS) $(BUILD)/extra.img $(BUILD)/filesystem/
	sleep 2
	pgrep redoxfs

unmount: FORCE
	sync
	-$(FUMOUNT) $(BUILD)/filesystem/ || true
	rm -rf $(BUILD)/filesystem/
	-$(FUMOUNT) /tmp/redox_installer || true
