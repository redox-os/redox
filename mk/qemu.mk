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

qemu: build/harddrive.bin
	$(QEMU) $(QEMUFLAGS) -drive file=$<,format=raw

qemu_extra: build/harddrive.bin
	if [ ! -e build/extra.bin ]; then dd if=/dev/zero of=build/extra.bin bs=1048576 count=1024; fi
	$(QEMU) $(QEMUFLAGS) -drive file=$<,format=raw -drive file=build/extra.bin,format=raw

qemu_no_build:
	$(QEMU) $(QEMUFLAGS) -drive file=build/harddrive.bin,format=raw

qemu_live: build/livedisk.bin
	$(QEMU) $(QEMUFLAGS) -device usb-ehci,id=flash_bus -drive id=flash_drive,file=$<,format=raw,if=none -device usb-storage,drive=flash_drive,bus=flash_bus.0

qemu_live_no_build:
	$(QEMU) $(QEMUFLAGS) -device usb-ehci,id=flash_bus -drive id=flash_drive,file=build/livedisk.bin,format=raw,if=none -device usb-storage,drive=flash_drive,bus=flash_bus.0

qemu_iso: build/livedisk.iso
	$(QEMU) $(QEMUFLAGS) -boot d -cdrom $<

qemu_iso_no_build:
		$(QEMU) $(QEMUFLAGS) -boot d -cdrom build/livedisk.iso
