QEMU=SDL_VIDEO_X11_DGAMOUSE=0 qemu-system-$(QEMU_ARCH)
QEMUFLAGS=-d cpu_reset,guest_errors -name "Redox OS $(ARCH)"

ifeq ($(ARCH),i686)
	audio?=ac97
	efi=no
	QEMU_ARCH=i386
	QEMU_MACHINE?=pc
	QEMU_CPU?=pentium2
	QEMUFLAGS+=-smp 1 -m 1024
else ifeq ($(ARCH),x86_64)
	QEMU_ARCH=x86_64
	QEMU_MACHINE?=q35
	QEMU_CPU?=core2duo
	QEMU_EFI=/usr/share/OVMF/OVMF_CODE.fd
	QEMUFLAGS+=-smp 4 -m 2048
else ifeq ($(ARCH),aarch64)
	efi=yes
	live=yes
	QEMU_ARCH=aarch64
	QEMU_MACHINE=virt
	QEMU_CPU=max
	ifeq ($(BOARD),raspi3bp)
		QEMU_EFI=https://gitlab.redox-os.org/Ivan/redox_firmware/-/raw/main/platform/raspberry_pi/rpi3/u-boot-rpi-3-b-plus.bin
	else
		QEMU_EFI=https://gitlab.redox-os.org/Ivan/redox_firmware/-/raw/main/platform/qemu/qemu_arm64/u-boot-qemu-arm64.bin
	endif
	QEMUFLAGS+=-smp 1 -m 2048
	ifneq ($(vga),no)
		QEMUFLAGS+=-device ramfb
	endif
	ifneq ($(usb),no)
		QEMUFLAGS+=-device usb-ehci -device usb-kbd -device usb-mouse
	endif
else
$(error Unsupported ARCH for QEMU "$(ARCH)"))
endif

ifneq ($(ARCH),$(HOST_ARCH))
	kvm?=no
endif

ifeq ($(efi),yes)
	FIRMWARE=$(BUILD)/firmware.rom
	QEMUFLAGS+=-bios $(BUILD)/firmware.rom
else
	FIRMWARE=
endif

ifeq ($(live),yes)
	DISK=$(BUILD)/livedisk.iso
else
	DISK=$(BUILD)/harddrive.img
endif

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

ifeq ($(audio),no)
	# No audio
else ifeq ($(audio),ac97)
	# AC'97
	QEMUFLAGS+=-device AC97
else
	# Intel High Definition Audio
	QEMUFLAGS+=-device ich9-intel-hda -device hda-output
endif

ifeq ($(net),no)
	QEMUFLAGS+=-net none
else ifeq ($(net),rtl8139)
	# RTL8139
	QEMUFLAGS+=-netdev user,id=net0 -device rtl8139,netdev=net0 \
				-object filter-dump,id=f1,netdev=net0,file=$(BUILD)/network.pcap
else ifeq ($(net),virtio)
	# virtio-net
	QEMUFLAGS+=-netdev user,id=net0 -device virtio-net,netdev=net0 \
				-object filter-dump,id=f1,netdev=net0,file=$(BUILD)/network.pcap
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
						-object filter-dump,id=f1,netdev=net0,file=$(BUILD)/network.pcap
		endif
	endif
endif

ifeq ($(vga),no)
	QEMUFLAGS+=-nographic -vga none
else ifeq ($(vga),multi)
	QEMUFLAGS+=-display sdl -vga std -device secondary-vga
else ifeq ($(vga),virtio)
	QEMUFLAGS+=-vga virtio
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

ifeq ($(UNAME),Darwin)
	QEMUFLAGS+=-cpu $(QEMU_CPU)
endif

ifeq ($(UNAME),Linux)
$(BUILD)/extra.img:
	fallocate --posix --length 1G $@
else
$(BUILD)/extra.img:
	truncate -s 1g $@
endif

$(BUILD)/firmware.rom:
ifeq ($(ARCH),aarch64)
	wget -O $@ $(QEMU_EFI)
else
	cp $(QEMU_EFI) $@
endif

qemu: $(DISK) $(FIRMWARE) $(BUILD)/extra.img
	$(QEMU) $(QEMUFLAGS) \
		-drive file=$(DISK),format=raw \
		-drive file=$(BUILD)/extra.img,format=raw

qemu_no_build: $(FIRMWARE) $(BUILD)/extra.img
	$(QEMU) $(QEMUFLAGS) \
		-drive file=$(DISK),format=raw \
		-drive file=$(BUILD)/extra.img,format=raw

qemu_cdrom: $(DISK) $(FIRMWARE) $(BUILD)/extra.img
	$(QEMU) $(QEMUFLAGS) \
		-boot d -cdrom $(DISK) \
		-drive file=$(BUILD)/extra.img,format=raw

qemu_cdrom_no_build: $(FIRMWARE) $(BUILD)/extra.img
	$(QEMU) $(QEMUFLAGS) \
		-boot d -cdrom $(DISK) \
		-drive file=$(BUILD)/extra.img,format=raw

qemu_nvme: $(DISK) $(FIRMWARE) $(BUILD)/extra.img
	$(QEMU) $(QEMUFLAGS) \
		-drive file=$(DISK),format=raw,if=none,id=drv0 -device nvme,drive=drv0,serial=NVME_SERIAL \
		-drive file=$(BUILD)/extra.img,format=raw,if=none,id=drv1 -device nvme,drive=drv1,serial=NVME_EXTRA

qemu_nvme_no_build: $(FIRMWARE) $(BUILD)/extra.img
	$(QEMU) $(QEMUFLAGS) \
		-drive file=$(DISK),format=raw,if=none,id=drv0 -device nvme,drive=drv0,serial=NVME_SERIAL \
		-drive file=$(BUILD)/extra.img,format=raw,if=none,id=drv1 -device nvme,drive=drv1,serial=NVME_EXTRA

qemu_usb: $(DISK) $(FIRMWARE)
	$(QEMU) $(QEMUFLAGS) \
		-drive if=none,id=usbstick,format=raw,file=$(DISK) \
		-device usb-storage,drive=usbstick

qemu_extra: $(FIRMWARE) $(BUILD)/extra.img
	$(QEMU) $(QEMUFLAGS) \
		-drive file=$(BUILD)/extra.img,format=raw

qemu_nvme_extra: $(FIRMWARE) $(BUILD)/extra.img
	$(QEMU) $(QEMUFLAGS) \
		-drive file=$(BUILD)/extra.img,format=raw,if=none,id=drv1 -device nvme,drive=drv1,serial=NVME_EXTRA

#additional steps for $(DISK) are required!!!
qemu_raspi: $(FIRMWARE) $(DISK)
	$(QEMU) -M raspi3b -smp 4,cores=1 \
		-kernel $(FIRMWARE) \
		-serial stdio -display none -sd $(DISK)
