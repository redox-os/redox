ifeq ($(ARCH),i686)
	efi=no
	QEMU_ARCH=i386
	QEMU_MACHINE=q35
	QEMU_CPU=pentium2
	#TODO: support higher CPU counts
	QEMUFLAGS=-smp 1 -m 2048
else ifeq ($(ARCH),x86_64)
	QEMU_ARCH=x86_64
	QEMU_MACHINE=q35
	QEMU_CPU=max
	QEMU_EFI=/usr/share/OVMF/OVMF_CODE.fd
	QEMUFLAGS=-smp 4 -m 2048
else ifeq ($(ARCH),aarch64)
	efi=yes
	kvm=no
	#TODO: support vga
	vga=no
	QEMU_ARCH=aarch64
	QEMU_MACHINE=virt
	QEMU_CPU=max
	QEMU_EFI=/usr/share/AAVMF/AAVMF_CODE.fd
	QEMUFLAGS=-smp 1 -m 2048
	ifneq ($(vga),no)
		QEMUFLAGS+=-device virtio-gpu-pci
	endif
	ifneq ($(usb),no)
		QEMUFLAGS+=-device usb-ehci -device usb-kbd -device usb-mouse
	endif
else
$(error Unsupported ARCH for QEMU "$(ARCH)"))
endif

ifeq ($(efi),yes)
	FIRMWARE=build/firmware.rom
	QEMUFLAGS+=-bios build/firmware.rom
	ifeq ($(live),yes)
		HARDDRIVE=build/livedisk-efi.bin
	else
		HARDDRIVE=build/harddrive-efi.bin
	endif
else
	FIRMWARE=
	ifeq ($(live),yes)
		HARDDRIVE=build/livedisk.bin
	else
		HARDDRIVE=build/harddrive.bin
	endif
endif

QEMU=SDL_VIDEO_X11_DGAMOUSE=0 qemu-system-$(QEMU_ARCH)
QEMUFLAGS+=-d cpu_reset,guest_errors
ifeq ($(serial),no)
	QEMUFLAGS+=-chardev stdio,id=debug -device isa-debugcon,iobase=0x402,chardev=debug
else
	QEMUFLAGS+=-chardev stdio,id=debug,signal=off,mux=on,"$(if $(qemu_serial_logfile),logfile=$(qemu_serial_logfile))"
	QEMUFLAGS+=-serial chardev:debug -mon chardev=debug
endif
ifeq ($(iommu),yes)
	QEMUFLAGS+=-machine $(QEMU_MACHINE),iommu=on
else
	QEMUFLAGS+=-machine $(QEMU_MACHINE)
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
	    ifeq ($(net),redir)
			# port 8080 and 8083 - webservers
			# port 64126 - our gdbserver implementation
			QEMUFLAGS+=-netdev user,id=net0,hostfwd=tcp::8080-:8080,hostfwd=tcp::8083-:8083,hostfwd=tcp::64126-:64126 -device e1000,netdev=net0,id=nic0
		else
			QEMUFLAGS+=-netdev user,id=net0 -device e1000,netdev=net0 \
						-object filter-dump,id=f1,netdev=net0,file=build/network.pcap
		endif
	endif
endif
ifeq ($(vga),no)
	QEMUFLAGS+=-nographic -vga none
endif
ifneq ($(usb),no)
	QEMUFLAGS+=-device nec-usb-xhci,id=xhci
endif
ifeq ($(gdb),yes)
	QEMUFLAGS+=-s -S
endif
ifeq ($(UNAME),Linux)
	ifneq ($(kvm),no)
		QEMUFLAGS+=-enable-kvm -cpu host
	else
		QEMUFLAGS+=-cpu $(QEMU_CPU)
	endif
endif
#,int,pcall
#-device intel-iommu

ifeq ($(UNAME),Linux)
build/extra.bin:
	fallocate --posix --length 1G $@
else
build/extra.bin:
	truncate -s 1g $@
endif

build/firmware.rom:
	cp $(QEMU_EFI) $@

qemu: $(HARDDRIVE) $(FIRMWARE) build/extra.bin
	$(QEMU) $(QEMUFLAGS) \
		-drive file=$(HARDDRIVE),format=raw \
		-drive file=build/extra.bin,format=raw

qemu_no_build: $(FIRMWARE) build/extra.bin
	$(QEMU) $(QEMUFLAGS) \
		-drive file=$(HARDDRIVE),format=raw \
		-drive file=build/extra.bin,format=raw

qemu_nvme: $(HARDDRIVE) $(FIRMWARE) build/extra.bin
	$(QEMU) $(QEMUFLAGS) \
		-drive file=$(HARDDRIVE),format=raw,if=none,id=drv0 -device nvme,drive=drv0,serial=NVME_SERIAL \
		-drive file=build/extra.bin,format=raw,if=none,id=drv1 -device nvme,drive=drv1,serial=NVME_EXTRA

qemu_nvme_no_build: $(FIRMWARE) build/extra.bin
	$(QEMU) $(QEMUFLAGS) \
		-drive file=$(HARDDRIVE),format=raw,if=none,id=drv0 -device nvme,drive=drv0,serial=NVME_SERIAL \
		-drive file=build/extra.bin,format=raw,if=none,id=drv1 -device nvme,drive=drv1,serial=NVME_EXTRA

qemu_iso: build/livedisk.iso $(FIRMWARE) build/extra.bin
	$(QEMU) $(QEMUFLAGS) \
		-boot d -cdrom build/livedisk.iso \
		-drive file=build/extra.bin,format=raw

qemu_iso_no_build: $(FIRMWARE) build/extra.bin
	$(QEMU) $(QEMUFLAGS) \
		-boot d -cdrom build/livedisk.iso \
		-drive file=build/extra.bin,format=raw

qemu_extra: $(FIRMWARE) build/extra.bin
	$(QEMU) $(QEMUFLAGS) \
		-drive file=build/extra.bin,format=raw

qemu_nvme_extra: $(FIRMWARE) build/extra.bin
	$(QEMU) $(QEMUFLAGS) \
		-drive file=build/extra.bin,format=raw,if=none,id=drv1 -device nvme,drive=drv1,serial=NVME_EXTRA
