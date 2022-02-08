bootloader/build/$(BOOTLOADER_TARGET)/bootloader.bin: FORCE
	env --unset=RUST_TARGET_PATH --unset=RUSTUP_TOOLCHAIN --unset=XARGO_RUST_SRC \
	$(MAKE) -C bootloader build/$(BOOTLOADER_TARGET)/bootloader.bin TARGET=$(BOOTLOADER_TARGET)

build/bootloader.bin: bootloader/build/$(BOOTLOADER_TARGET)/bootloader.bin
	mkdir -p build
	cp -v $< $@

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
