build/harddrive.bin: build/filesystem.bin $(BOOTLOADER_EFI) $(BOOTLOADER_BIOS)
	rm -f $@ $@.partial
	env \
		PARTED=$(PARTED) \
		BOOTLOADER_EFI=$(BOOTLOADER_EFI) \
		BOOTLOADER_EFI_PATH=$(BOOTLOADER_EFI_PATH) \
		BOOTLOADER_BIOS=$(BOOTLOADER_BIOS) \
		./mk/disk.sh $@.partial $<
	mv "$@.partial" "$@"

build/livedisk.bin: build/filesystem.bin $(BOOTLOADER_EFI_LIVE) $(BOOTLOADER_BIOS_LIVE)
	rm -f $@ $@.partial
	env \
		PARTED=$(PARTED) \
		BOOTLOADER_EFI=$(BOOTLOADER_EFI_LIVE) \
		BOOTLOADER_EFI_PATH=$(BOOTLOADER_EFI_PATH) \
		BOOTLOADER_BIOS=$(BOOTLOADER_BIOS_LIVE) \
		./mk/disk.sh $@.partial $<
	mv "$@.partial" "$@"

build/livedisk.iso: build/livedisk.bin.gz
	rm -rf build/iso/
	mkdir -p build/iso/
	cp -RL isolinux build/iso/
	cp $< build/iso/livedisk.gz
	genisoimage -o $@ -b isolinux/isolinux.bin -c isolinux/boot.cat \
					-no-emul-boot -boot-load-size 4 -boot-info-table \
					build/iso/
	isohybrid $@
