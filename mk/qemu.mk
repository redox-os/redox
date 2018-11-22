QEMU=SDL_VIDEO_X11_DGAMOUSE=0 qemu-system-$(ARCH)
QEMUFLAGS=-serial mon:stdio -d cpu_reset -d guest_errors
QEMUFLAGS+=-smp 4 -m 2048
ifeq ($(iommu),yes)
	QEMUFLAGS+=-machine q35,iommu=on
else
	QEMUFLAGS+=-machine q35
endif
ifneq ($(audio),no)
	QEMUFLAGS+=-device ich9-intel-hda -device hda-duplex
endif
ifeq ($(net),no)
	QEMUFLAGS+=-net none
else
	ifneq ($(bridge),)
		QEMUFLAGS+=-netdev bridge,br=$(bridge),id=net0 -device e1000,netdev=net0,id=nic0
	else
		QEMUFLAGS+=-netdev user,id=net0 -device e1000,netdev=net0 \
					-object filter-dump,id=f1,netdev=net0,file=build/network.pcap
	endif
	ifeq ($(net),redir)
		QEMUFLAGS+=-redir tcp:8023::8023 -redir tcp:8080::8080
	endif
endif
ifeq ($(vga),no)
	QEMUFLAGS+=-nographic -vga none
endif
ifneq ($(usb),no)
	QEMUFLAGS+=-device nec-usb-xhci,id=xhci -device usb-tablet,bus=xhci.0
endif
ifneq ($(gdb),yes)
	QEMUFLAGS+=-s
endif
ifeq ($(UNAME),Linux)
	ifneq ($(kvm),no)
		QEMUFLAGS+=-enable-kvm -cpu host
	endif
endif
#,int,pcall
#-device intel-iommu

build/extra.qcow2:
	qemu-img create -f qcow2 $@ 1G

qemu: build/harddrive.bin build/extra.qcow2
	$(QEMU) $(QEMUFLAGS) \
		-drive file=build/harddrive.bin,format=raw \
		-drive file=build/extra.qcow2

qemu_no_build: build/extra.qcow2
	$(QEMU) $(QEMUFLAGS) \
		-drive file=build/harddrive.bin,format=raw \
		-drive file=build/extra.qcow2

qemu_efi: build/harddrive-efi.bin build/extra.qcow2
	$(QEMU) $(QEMUFLAGS) \
		-bios /usr/share/OVMF/OVMF_CODE.fd \
		-drive file=build/harddrive-efi.bin,format=raw \
		-drive file=build/extra.qcow2

qemu_efi_no_build: build/extra.qcow2
	$(QEMU) $(QEMUFLAGS) \
		-bios /usr/share/OVMF/OVMF_CODE.fd \
		-drive file=build/harddrive-efi.bin,format=raw \
		-drive file=build/extra.qcow2

qemu_nvme: build/harddrive.bin build/extra.qcow2
	$(QEMU) $(QEMUFLAGS) \
		-drive file=build/harddrive.bin,format=raw -drive file=build/extra.qcow2,if=none,id=drv0 -device nvme,drive=drv0,serial=NVME_SERIAL \
		-drive file=build/extra.qcow2

qemu_nvme_no_build: build/extra.qcow2
	$(QEMU) $(QEMUFLAGS) \
		-drive file=build/harddrive.bin,format=raw -drive file=build/extra.qcow2,if=none,id=drv0 -device nvme,drive=drv0,serial=NVME_SERIAL \
		-drive file=build/extra.qcow2

qemu_live: build/livedisk.bin build/extra.qcow2
	$(QEMU) $(QEMUFLAGS) \
		-device usb-ehci,id=flash_bus -drive id=flash_drive,file=build/livedisk.bin,format=raw,if=none -device usb-storage,drive=flash_drive,bus=flash_bus.0 \
		-drive file=build/extra.qcow2

qemu_live_no_build: build/extra.qcow2
	$(QEMU) $(QEMUFLAGS) \
		-device usb-ehci,id=flash_bus -drive id=flash_drive,file=build/livedisk.bin,format=raw,if=none -device usb-storage,drive=flash_drive,bus=flash_bus.0 \
		-drive file=build/extra.qcow2

qemu_iso: build/livedisk.iso build/extra.qcow2
	$(QEMU) $(QEMUFLAGS) \
		-boot d -cdrom build/livedisk.iso \
		-drive file=build/extra.qcow2

qemu_iso_no_build: build/extra.qcow2
	$(QEMU) $(QEMUFLAGS) \
		-boot d -cdrom build/livedisk.iso \
		-drive file=build/extra.qcow2

qemu_iso_efi: build/livedisk-efi.iso build/extra.qcow2
	$(QEMU) $(QEMUFLAGS) \
		-bios /usr/share/OVMF/OVMF_CODE.fd \
		-boot d -cdrom build/livedisk-efi.iso \
		-drive file=build/extra.qcow2

qemu_iso_efi_no_build: build/extra.qcow2
	$(QEMU) $(QEMUFLAGS) \
		-bios /usr/share/OVMF/OVMF_CODE.fd \
		-boot d -cdrom build/livedisk-efi.iso \
		-drive file=build/extra.qcow2

qemu_extra: build/extra.qcow2
	$(QEMU) $(QEMUFLAGS) \
		-drive file=build/extra.qcow2
