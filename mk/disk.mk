# Configuration file with the commands configuration of the Redox image

$(BUILD)/harddrive.img: $(INSTALLER) $(FSTOOLS_TAG) $(REPO_TAG)
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

$(BUILD)/livedisk.iso: $(INSTALLER) $(FSTOOLS_TAG) $(REPO_TAG)
	mkdir -p $(BUILD)
	rm -rf $@  $@.partial
	-$(FUMOUNT) /tmp/redox_installer || true
	FILESYSTEM_SIZE=$(FILESYSTEM_SIZE) && \
	if [ -z "$$FILESYSTEM_SIZE" ] ; then \
	FILESYSTEM_SIZE=$(shell $(INSTALLER) --filesystem-size -c $(FILESYSTEM_CONFIG)); \
	fi && \
	truncate -s "$$FILESYSTEM_SIZE"m $@.partial
	umask 002 && $(INSTALLER) $(INSTALLER_OPTS) -c $(FILESYSTEM_CONFIG) --live $@.partial
	mv $@.partial $@

$(BUILD)/filesystem.img: $(INSTALLER) $(FSTOOLS_TAG) $(REPO_TAG)
	mkdir -p $(BUILD)
	-$(FUMOUNT) $(BUILD)/filesystem/ || true
	rm -rf $@  $@.partial $(BUILD)/filesystem/
	-$(FUMOUNT) /tmp/redox_installer || true
	FILESYSTEM_SIZE=$(FILESYSTEM_SIZE) && \
	if [ -z "$$FILESYSTEM_SIZE" ] ; then \
	FILESYSTEM_SIZE=$(shell $(INSTALLER) --filesystem-size -c $(FILESYSTEM_CONFIG)); \
	fi && \
	truncate -s "$$FILESYSTEM_SIZE"m $@.partial
	redoxfs/target/release/redoxfs-mkfs $(REDOXFS_MKFS_FLAGS) $@.partial
	mkdir -p $(BUILD)/filesystem/
	redoxfs/target/release/redoxfs $@.partial $(BUILD)/filesystem/
	sleep 1
	pgrep redoxfs
	umask 002 && $(INSTALLER) $(INSTALLER_OPTS) -c $(FILESYSTEM_CONFIG) $(BUILD)/filesystem/
	sync
	-$(FUMOUNT) $(BUILD)/filesystem/ || true
	rm -rf $(BUILD)/filesystem/
	mv $@.partial $@

mount: $(FSTOOLS_TAG) FORCE
	mkdir -p $(BUILD)/filesystem/
	redoxfs/target/release/redoxfs $(BUILD)/harddrive.img $(BUILD)/filesystem/
	sleep 2
	pgrep redoxfs

mount_extra: $(FSTOOLS_TAG) FORCE
	mkdir -p $(BUILD)/filesystem/
	redoxfs/target/release/redoxfs $(BUILD)/extra.img $(BUILD)/filesystem/
	sleep 2
	pgrep redoxfs

unmount: FORCE
	sync
	-$(FUMOUNT) $(BUILD)/filesystem/ || true
	rm -rf $(BUILD)/filesystem/
	-$(FUMOUNT) /tmp/redox_installer || true
