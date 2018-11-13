build/bootloader: bootloader/$(ARCH)/**
	mkdir -p build
	nasm -f bin -o $@ -D ARCH_$(ARCH) -ibootloader/$(ARCH)/ bootloader/$(ARCH)/disk.asm

build/harddrive.bin: build/filesystem.bin bootloader/$(ARCH)/**
	nasm -f bin -o $@ -D ARCH_$(ARCH) -D FILESYSTEM=$< -ibootloader/$(ARCH)/ bootloader/$(ARCH)/disk.asm

build/harddrive.bin.gz: build/harddrive.bin
	gzip -k -f $<

build/livedisk.bin: build/kernel_live bootloader/$(ARCH)/**
	nasm -f bin -o $@ -D ARCH_$(ARCH) -D KERNEL=$< -ibootloader/$(ARCH)/ bootloader/$(ARCH)/disk.asm

build/livedisk.bin.gz: build/livedisk.bin
	gzip -k -f $<

build/livedisk.iso: build/livedisk.bin.gz
	rm -rf build/iso/
	mkdir -p build/iso/
	cp -RL isolinux build/iso/
	cp $< build/iso/livedisk.gz
	genisoimage -o $@ -b isolinux/isolinux.bin -c isolinux/boot.cat \
					-no-emul-boot -boot-load-size 4 -boot-info-table \
					build/iso/
	isohybrid $@

bootloader-efi/build/redox_bootloader/boot.efi:
	$(MAKE) -C bootloader-efi build/redox_bootloader/boot.efi

build/bootloader.efi: bootloader-efi/build/redox_bootloader/boot.efi
	cp $< $@

build/harddrive-efi.bin: build/bootloader.efi build/filesystem.bin
	dd if=/dev/zero of=$@.partial bs=1048576 count=4
	mkfs.vfat $@.partial
	mmd -i $@.partial efi
	mmd -i $@.partial efi/boot
	mcopy -i $@.partial $< ::efi/boot/bootx64.efi
	mmd -i $@.partial redox_bootloader
	mcopy -i $@.partial -s bootloader-efi/res ::redox_bootloader
	cat $@.partial build/filesystem.bin > $@

build/harddrive-efi.bin.gz: build/harddrive-efi.bin
	gzip -k -f $<

build/livedisk-efi.iso: build/bootloader.efi build/kernel_live
	dd if=/dev/zero of=$@.partial bs=1048576 count=272
	mkfs.vfat $@.partial
	mmd -i $@.partial efi
	mmd -i $@.partial efi/boot
	mcopy -i $@.partial $< ::efi/boot/bootx64.efi
	mmd -i $@.partial redox_bootloader
	mcopy -i $@.partial -s bootloader-efi/res ::redox_bootloader
	mcopy -i $@.partial -s build/kernel_live ::redox_bootloader/kernel
	mv $@.partial $@

build/livedisk-efi.bin.gz: build/livedisk-efi.bin
	gzip -k -f $<
