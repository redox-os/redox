build/bootloader: bootloader/$(ARCH)/**
	mkdir -p build
	nasm -f bin -o $@ -D ARCH_$(ARCH) -ibootloader/$(ARCH)/ bootloader/$(ARCH)/disk.asm

build/harddrive.bin: build/filesystem.bin bootloader/$(ARCH)/**
	nasm -f bin -o build/bootsector.bin -D ARCH_$(ARCH) -ibootloader/$(ARCH)/ bootloader/$(ARCH)/disk.asm
	dd if=/dev/zero of=$@.partial bs=1M count=$$(expr $$(du -m $< | cut -f1) + 2)
	$(PARTED) -s -a minimal $@.partial mklabel msdos
	$(PARTED) -s -a minimal $@.partial mkpart primary 2048s $$(expr $$(du -m $< | cut -f1) \* 2048 + 2048)s
	dd if=build/bootsector.bin of=$@.partial bs=1 count=446 conv=notrunc
	dd if=build/bootsector.bin of=$@.partial bs=512 skip=1 seek=1 conv=notrunc
	dd if=$< of=$@.partial bs=1M seek=1 conv=notrunc
	mv $@.partial $@

build/livedisk.bin: build/kernel_live bootloader/$(ARCH)/**
	nasm -f bin -o $@ -D ARCH_$(ARCH) -D KERNEL=$< -ibootloader/$(ARCH)/ bootloader/$(ARCH)/disk.asm

build/livedisk.iso: build/livedisk.bin.gz
	rm -rf build/iso/
	mkdir -p build/iso/
	cp -RL isolinux build/iso/
	cp $< build/iso/livedisk.gz
	genisoimage -o $@ -b isolinux/isolinux.bin -c isolinux/boot.cat \
					-no-emul-boot -boot-load-size 4 -boot-info-table \
					build/iso/
	isohybrid $@

bootloader-coreboot/build/bootloader: build/kernel_coreboot
	env --unset=RUST_TARGET_PATH --unset=RUSTUP_TOOLCHAIN --unset=XARGO_RUST_SRC \
	$(MAKE) -C bootloader-coreboot clean build/bootloader KERNEL="$(ROOT)/$<"

build/coreboot.elf: bootloader-coreboot/build/bootloader
	mkdir -p build
	cp -v $< $@

bootloader-efi/build/$(EFI_TARGET)/boot.efi: FORCE
	env --unset=RUST_TARGET_PATH --unset=RUSTUP_TOOLCHAIN --unset=XARGO_RUST_SRC \
	$(MAKE) -C bootloader-efi build/$(EFI_TARGET)/boot.efi TARGET=$(EFI_TARGET)

build/bootloader.efi: bootloader-efi/build/$(EFI_TARGET)/boot.efi
	mkdir -p build
	cp -v $< $@

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

build/livedisk-efi.iso: build/bootloader.efi build/kernel_live
	dd if=/dev/zero of=$@.partial bs=1048576 count=$$(expr $$(du -mc $^ | grep 'total$$' | cut -f1) + 1)
	mkfs.vfat $@.partial
	mmd -i $@.partial efi
	mmd -i $@.partial efi/boot
	mcopy -i $@.partial $< ::efi/boot/bootx64.efi
	mmd -i $@.partial redox_bootloader
	mcopy -i $@.partial -s build/kernel_live ::redox_bootloader/kernel
	mv $@.partial $@
