build/harddrive.bin: build/filesystem.bin build/bootloader.bin
	dd if=/dev/zero of=$@.partial bs=1M count=$$(expr $$(du -m $< | cut -f1) + 2)
	$(PARTED) -s -a minimal $@.partial mklabel msdos
	$(PARTED) -s -a minimal $@.partial mkpart primary 2048s $$(expr $$(du -m $< | cut -f1) \* 2048 + 2048)s
	dd if=build/bootloader.bin of=$@.partial bs=1 count=446 conv=notrunc
	dd if=build/bootloader.bin of=$@.partial bs=512 skip=1 seek=1 conv=notrunc
	dd if=$< of=$@.partial bs=1M seek=1 conv=notrunc
	mv $@.partial $@

build/livedisk: build/kernel_live
	mkdir -p $@.partial
	cp $< $@.partial/kernel
	mv $@.partial $@

build/livedisk.bin: build/filesystem-live.bin build/bootloader.bin
	dd if=/dev/zero of=$@.partial bs=1M count=$$(expr $$(du -m $< | cut -f1) + 2)
	$(PARTED) -s -a minimal $@.partial mklabel msdos
	$(PARTED) -s -a minimal $@.partial mkpart primary 2048s $$(expr $$(du -m $< | cut -f1) \* 2048 + 2048)s
	dd if=build/bootloader.bin of=$@.partial bs=1 count=446 conv=notrunc
	dd if=build/bootloader.bin of=$@.partial bs=512 skip=1 seek=1 conv=notrunc
	dd if=$< of=$@.partial bs=1M seek=1 conv=notrunc
	mv $@.partial $@

build/livedisk.iso: build/livedisk.bin.gz
	rm -rf build/iso/
	mkdir -p build/iso/
	cp -RL isolinux build/iso/
	cp $< build/iso/livedisk.gz
	genisoimage -o $@ -b isolinux/isolinux.bin -c isolinux/boot.cat \
					-no-emul-boot -boot-load-size 4 -boot-info-table \
					build/iso/
	isohybrid $@

build/harddrive-efi.bin: build/bootloader.efi build/filesystem.bin
	# TODO: Validate the correctness of this \
	# Populate an EFI system partition \
	dd if=/dev/zero of=$@.esp bs=1048576 count=$$(expr $$(du -m $< | cut -f1) + 1) && \
	mkfs.vfat $@.esp && \
	mmd -i $@.esp efi && \
	mmd -i $@.esp efi/boot && \
	mcopy -i $@.esp $< ::efi/boot/bootx64.efi && \
	# Create the disk \
	dd if=/dev/zero of=$@ bs=1048576 count=$$(expr $$(du -m $< | cut -f1) + 2 + $$(du -m build/filesystem.bin | cut -f1)) && \
	# Create partition table \
	$(PARTED) -s -a minimal $@ mklabel gpt && \
	efi_disk_size=$$(du -m $< | cut -f1) && \
	efi_disk_blkcount=$$(expr $$efi_disk_size \* $$(expr 1048576 / 512)) && \
	efi_end=$$(expr 2048 + $$efi_disk_blkcount) && \
	efi_last=$$(expr $$efi_end - 1) && \
	$(PARTED) -s -a minimal $@ mkpart EFI fat32 2048s "$${efi_last}s" && \
	fs_disk_size=$$(du -m build/filesystem.bin | cut -f1) && \
	fs_disk_blkcount=$$(expr $$fs_disk_size \* $$(expr 1048576 / 512)) && \
	$(PARTED) -s -a minimal $@ mkpart redox ext4 "$${efi_end}s" $$(expr $$efi_end + $$fs_disk_blkcount)s && \
	$(PARTED) -s -a minimal $@ set 1 boot on && \
	$(PARTED) -s -a minimal $@ set 1 esp on && \
	# Write the partitions \
	dd if=$@.esp bs=512 seek=2048 conv=notrunc count=$$efi_disk_blkcount of=$@ && \
	dd if=build/filesystem.bin seek=$$efi_end bs=512 conv=notrunc of=$@ count=$$fs_disk_blkcount

build/livedisk-efi.bin: build/bootloader.efi build/filesystem-live.bin
	# TODO: Validate the correctness of this \
	# Populate an EFI system partition \
	dd if=/dev/zero of=$@.esp bs=1048576 count=$$(expr $$(du -m $< | cut -f1) + 1) && \
	mkfs.vfat $@.esp && \
	mmd -i $@.esp efi && \
	mmd -i $@.esp efi/boot && \
	mcopy -i $@.esp $< ::efi/boot/bootx64.efi && \
	# Create the disk \
	dd if=/dev/zero of=$@ bs=1048576 count=$$(expr $$(du -m $< | cut -f1) + 2 + $$(du -m build/filesystem-live.bin | cut -f1)) && \
	# Create partition table \
	$(PARTED) -s -a minimal $@ mklabel gpt && \
	efi_disk_size=$$(du -m $< | cut -f1) && \
	efi_disk_blkcount=$$(expr $$efi_disk_size \* $$(expr 1048576 / 512)) && \
	efi_end=$$(expr 2048 + $$efi_disk_blkcount) && \
	efi_last=$$(expr $$efi_end - 1) && \
	$(PARTED) -s -a minimal $@ mkpart EFI fat32 2048s "$${efi_last}s" && \
	fs_disk_size=$$(du -m build/filesystem-live.bin | cut -f1) && \
	fs_disk_blkcount=$$(expr $$fs_disk_size \* $$(expr 1048576 / 512)) && \
	$(PARTED) -s -a minimal $@ mkpart redox ext4 "$${efi_end}s" $$(expr $$efi_end + $$fs_disk_blkcount)s && \
	$(PARTED) -s -a minimal $@ set 1 boot on && \
	$(PARTED) -s -a minimal $@ set 1 esp on && \
	# Write the partitions \
	dd if=$@.esp bs=512 seek=2048 conv=notrunc count=$$efi_disk_blkcount of=$@ && \
	dd if=build/filesystem-live.bin seek=$$efi_end bs=512 conv=notrunc of=$@ count=$$fs_disk_blkcount
