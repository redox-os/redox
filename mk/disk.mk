build/harddrive.bin: build/bootloader.efi build/filesystem.bin $(BOOTLOADER_BIOS)
	env PARTED=$(PARTED) ./mk/disk.sh $@ build/filesystem.bin $< $(BOOTLOADER_EFI_PATH) $(BOOTLOADER_BIOS)

build/livedisk.bin: build/bootloader-live.efi build/filesystem.bin $(BOOTLOADER_BIOS_LIVE)
	env PARTED=$(PARTED) ./mk/disk.sh $@ build/filesystem.bin $< $(BOOTLOADER_EFI_PATH) $(BOOTLOADER_BIOS_LIVE)

build/livedisk.iso: build/livedisk.bin.gz
	rm -rf build/iso/
	mkdir -p build/iso/
	cp -RL isolinux build/iso/
	cp $< build/iso/livedisk.gz
	genisoimage -o $@ -b isolinux/isolinux.bin -c isolinux/boot.cat \
					-no-emul-boot -boot-load-size 4 -boot-info-table \
					build/iso/
	isohybrid $@
