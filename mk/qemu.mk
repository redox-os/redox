QEMU=SDL_VIDEO_X11_DGAMOUSE=0 qemu-system-$(ARCH)
QEMUFLAGS=-serial mon:stdio -d cpu_reset -d guest_errors
QEMUFLAGS+=-smp 4 -m 1024
ifeq ($(iommu),yes)
	QEMUFLAGS+=-machine q35,iommu=on
else
	QEMUFLAGS+=-machine q35
endif
ifeq ($(net),no)
	QEMUFLAGS+=-net none
else
	QEMUFLAGS+=-net nic,model=e1000 -net user -net dump,file=build/network.pcap
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
ifeq ($(UNAME),Linux)
	ifneq ($(kvm),no)
		QEMUFLAGS+=-enable-kvm -cpu host
	endif
endif
#,int,pcall
#-device intel-iommu

build/extra.qcow2:
	qemu-img create -f qcow2 $@ 256M

qemu: build/harddrive.bin build/extra.qcow2
	$(QEMU) $(QEMUFLAGS) \
		-drive file=build/harddrive.bin,format=raw \
		-drive file=build/extra.qcow2

qemu_no_build: build/extra.qcow2
	$(QEMU) $(QEMUFLAGS) \
		-drive file=build/harddrive.bin,format=raw \
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
