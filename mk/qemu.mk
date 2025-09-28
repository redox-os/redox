# Configuration file for QEMU

QEMU=SDL_VIDEO_X11_DGAMOUSE=0 qemu-system-$(QEMU_ARCH)
QEMUFLAGS=-d guest_errors -name "Redox OS $(ARCH)"
netboot?=no
VGA_SUPPORTED=no

ifeq ($(ARCH),i686)
	audio?=ac97
	gpu?=vga
	uefi=no
	VGA_SUPPORTED=yes
	QEMU_ARCH=i386
	QEMU_MACHINE?=pc
	QEMU_CPU?=pentium2
	QEMU_SMP?=1
	QEMU_MEM?=1024

	# Default to using kvm when arch is i686 and host is x86_64
	ifeq ($(HOST_ARCH),x86_64)
		kvm?=yes
	endif
else ifeq ($(ARCH),x86_64)
	gpu?=vga
	uefi?=yes
	VGA_SUPPORTED=yes
	QEMU_ARCH=x86_64
	QEMU_MACHINE?=q35
	QEMU_CPU?=core2duo
	QEMU_SMP?=4
	QEMU_MEM?=2048
	ifeq ($(uefi),yes)
		FIRMWARE=$(firstword \
			$(wildcard /usr/share/ovmf/OVMF.fd) \
			$(wildcard /usr/share/OVMF/OVMF_CODE.fd) \
		)
		ifeq ($(FIRMWARE),)
			PFLASH0=$(firstword \
				$(wildcard /usr/share/qemu/edk2-x86_64-code.fd) \
				$(wildcard /opt/homebrew/opt/qemu/share/qemu/edk2s-x86_64-code.fd) \
			)
		endif
	endif
	ifneq ($(usb),no)
		QEMUFLAGS+=-device nec-usb-xhci,id=xhci
	endif
else ifeq ($(ARCH),aarch64)
	# Default to UEFI as U-Boot doesn't set up a framebuffer for us and we don't yet support
	# setting up a framebuffer ourself.
	uefi?=yes
	live?=yes
	gpu?=ramfb
	QEMU_ARCH=aarch64
	QEMU_MACHINE?=virt
	QEMU_CPU=max
	QEMU_SMP?=1
	QEMU_MEM?=2048
	ifeq ($(BOARD),raspi3bp)
		QEMU_KERNEL=$(BUILD)/raspi3bp_uboot.rom
		disk?=sdcard
		QEMU_MACHINE:=raspi3b
		QEMU_SMP:=4
		QEMU_MEM:=1024
		net:=usb-net
		audio:=no
		ifneq ($(usb),no)
			QEMUFLAGS+=-usb -device usb-kbd -device usb-tablet
		endif
	else
		ifeq ($(uefi),yes)
			FIRMWARE=$(firstword \
				$(wildcard /usr/share/AAVMF/AAVMF_CODE.fd) \
			)
			ifeq ($(FIRMWARE),)
				PFLASH0=$(firstword \
					$(wildcard /usr/share/qemu/edk2-aarch64-code.fd) \
					$(wildcard /opt/homebrew/opt/qemu/share/qemu/edk2-aarch64-code.fd) \
				)
			endif
		else
			FIRMWARE=$(BUILD)/qemu_uboot.rom
		endif
		ifneq ($(usb),no)
			QEMUFLAGS+=-device qemu-xhci -device usb-kbd -device usb-tablet
		endif
	endif

	# Default to using HVF when host is MacOS Silicon
	ifeq ($(HOST_ARCH),arm64)
		kvm?=yes
	endif
else ifeq ($(ARCH),riscv64gc)
	live=no
	audio=no
	gpu?=ramfb
	net=bridge
	QEMU_ARCH=riscv64
	# QEMU_MACHINE=virt  for ACPI mode instead of DTB
	QEMU_MACHINE=virt,acpi=off
#	QEMU_MACHINE:=${QEMU_MACHINE},aclint=on
#	QEMU_MACHINE:=${QEMU_MACHINE},aia=aplic
#	QEMU_MACHINE:=${QEMU_MACHINE},aia=aplic-imsic
	QEMU_SMP?=4
	QEMU_MEM?=2048
	QEMU_CPU=max
	disk?=nvme
	PFLASH0=$(firstword \
		$(wildcard /usr/share/qemu-efi-riscv64/RISCV_VIRT_CODE.fd) \
		$(wildcard /usr/share/qemu/edk2-riscv-code.fd) \
		$(wildcard /opt/homebrew/opt/qemu/share/qemu/edk2-riscv-code.fd) \
	)
	PFLASH1=$(firstword \
		$(wildcard /usr/share/qemu-efi-riscv64/RISCV_VIRT_VARS.fd) \
		$(wildcard /usr/share/qemu/edk2-riscv-vars.fd) \
		$(wildcard /opt/homebrew/opt/qemu/share/qemu/edk2-riscv-vars.fd) \
	)
	ifneq ($(usb),no)
		QEMUFLAGS+=-device qemu-xhci -device usb-kbd -device usb-tablet
	endif
else
$(error Unsupported ARCH for QEMU "$(ARCH)"))
endif

QEMUFLAGS+=-smp $(QEMU_SMP) -m $(QEMU_MEM)

# If host and target arch do not match, disable kvm
# (unless overridden above or by environment)
ifneq ($(ARCH),$(HOST_ARCH))
	kvm?=no
endif

# wsl2: run qemu on windows instead
ifeq ($(QEMU_ON_WINDOWS),1)
	QEMU:=$(QEMU).exe
	WINDOWS_DISK=/mnt/c/ProgramData/redox.img
	disk=windows
	net=windows
	QEMU_MACHINE=pc
	FIRMWARE=
	QEMU_KERNEL=
	QEMUFLAGS+=-device usb-tablet
endif

ifneq ($(FIRMWARE),)
	QEMUFLAGS+=-bios $(FIRMWARE)
endif

ifneq ($(QEMU_KERNEL),)
	QEMUFLAGS+=-kernel $(QEMU_KERNEL)
endif

ifeq ($(live),yes)
	DISK=$(BUILD)/redox-live.iso
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
else
	ifeq ($(net),rtl8139) # RTL8139
		QEMUFLAGS+=-device rtl8139,netdev=net0
	else ifeq ($(net),virtio) # virtio-net
		QEMUFLAGS+=-device virtio-net,netdev=net0
	else ifeq ($(net),usb-net)
		QEMUFLAGS+=-device usb-net,netdev=net0
	else
		QEMUFLAGS+=-device e1000,netdev=net0,id=nic0
	endif

	EXTRANETARGS=
	ifeq ($(netboot),yes)
		EXTRANETARGS+=,tftp=$(BUILD),bootfile=redox.ipxe
		QEMUFLAGS+=-kernel /usr/lib/ipxe/ipxe-amd64.efi
	endif

	ifneq ($(bridge),)
		QEMUFLAGS+=-netdev bridge,br=$(bridge),id=net0
	else ifeq ($(net),redir)
		# port 8022 - ssh
		# port 8080-8083 - webservers
		# port 64126 - our gdbserver implementation
		FWD_PORTS := 8081 8082 8083 64126
		FWD_FLAGS := hostfwd=tcp::8022-:22,hostfwd=tcp::8080-:80
		FWD_FLAGS2 := $(foreach p,$(FWD_PORTS),,hostfwd=tcp::$(p)-:$(p))
		QEMUFLAGS += -netdev user,id=net0,$(FWD_FLAGS)$(subst $(eval ) ,,$(FWD_FLAGS2))$(EXTRANETARGS)
	else ifeq ($(net),windows)
		QEMUFLAGS+=-netdev user,id=net0$(EXTRANETARGS)
	else
		QEMUFLAGS+=-netdev user,id=net0$(EXTRANETARGS) -object filter-dump,id=f1,netdev=net0,file=$(BUILD)/network.pcap
	endif
endif

ifeq ($(gpu),no)
	QEMUFLAGS+=-nographic -vga none
else ifeq ($(gpu),vga)
	ifeq ($(VGA_SUPPORTED),yes)
		QEMUFLAGS+=-vga std
	else
		QEMUFLAGS+=-vga none -device secondary-vga
	endif
else ifeq ($(gpu),ramfb)
	QEMUFLAGS+=-vga none -device ramfb
else ifeq ($(gpu),multi)
	ifeq ($(VGA_SUPPORTED),yes)
		QEMUFLAGS+=-display sdl -vga none -device virtio-vga,max_outputs=2
	else
		QEMUFLAGS+=-display sdl -vga none -device virtio-gpu,max_outputs=2
	endif
else ifeq ($(gpu),virtio)
	ifeq ($(VGA_SUPPORTED),yes)
		QEMUFLAGS+=-vga none -device virtio-vga
	else
		QEMUFLAGS+=-vga none -device virtio-gpu
	endif
else ifeq ($(gpu),virtio-gl)
	ifeq ($(VGA_SUPPORTED),yes)
		QEMUFLAGS+=-display gtk,gl=on -vga none -device virtio-vga-gl
	else
		QEMUFLAGS+=-display gtk,gl=on -vga none -device virtio-gpu-gl
	endif
endif

EXTRA_DISK=$(BUILD)/extra.img
disk?=ata
ifeq ($(disk),ata)
	# For i386, ata will use ided
	# For aarch64 and x86_64, ata will use ahcid
	QEMUFLAGS+= \
		-drive file=$(DISK),format=raw \
		-drive file=$(EXTRA_DISK),format=raw
else ifeq ($(disk),nvme)
	QEMUFLAGS+= \
		-drive file=$(DISK),format=raw,if=none,id=drv0 -device nvme,drive=drv0,serial=NVME_SERIAL \
		-drive file=$(EXTRA_DISK),format=raw,if=none,id=drv1 -device nvme,drive=drv1,serial=NVME_EXTRA
else ifeq ($(disk),usb)
	QEMUFLAGS+= \
		-drive if=none,id=usbstick,format=raw,file=$(DISK) \
		-device usb-storage,drive=usbstick
else ifeq ($(disk),virtio)
	QEMUFLAGS+= \
		-drive file=$(DISK),format=raw,if=virtio \
		-drive file=$(EXTRA_DISK),format=raw,if=virtio
else ifeq ($(disk),cdrom)
	QEMUFLAGS+= \
		-boot d -cdrom $(DISK) \
		-drive file=$(EXTRA_DISK),format=raw
else ifeq ($(disk),sdcard)
	QEMUFLAGS+=-drive file=$(DISK),if=sd,format=raw
else ifeq ($(disk),windows)
	QEMUFLAGS+=-drive file="$(shell wslpath -w $(WINDOWS_DISK))",format=raw,if=virtio
endif

ifeq ($(gdb),yes)
	QEMUFLAGS+=-d cpu_reset -s -S
else ifeq ($(gdb),nonblock)
	# Allow attaching gdb, but don't block for it
	QEMUFLAGS+=-d cpu_reset -s
endif

ifeq ($(UNAME),Linux)
	ifneq ($(kvm),no)
		ifeq ($(QEMU_ON_WINDOWS),1)
			QEMUFLAGS+=-accel whpx,kernel-irqchip=off -cpu Broadwell,x2apic=off
		else
			QEMUFLAGS+=-enable-kvm -cpu host
		endif
	else
		QEMUFLAGS+=-cpu $(QEMU_CPU)
	endif
endif

ifeq ($(UNAME),Darwin)
    ifneq ($(kvm),no)
        QEMUFLAGS+=-accel hvf -cpu max
    else
        QEMUFLAGS+=-cpu $(QEMU_CPU)
    endif
endif

ifneq ($(PFLASH0),)
	QEMUFLAGS+=-drive if=pflash,format=raw,unit=0,file=$(PFLASH0),readonly=on
endif

ifneq ($(PFLASH1),)
	QEMUFLAGS+=-drive if=pflash,format=raw,unit=1,file=$(BUILD)/fw_vars.bin
endif

.PHONY: qemu-deps

qemu-deps: $(DISK)

ifeq ($(disk),usb)
else ifeq ($(disk),sdcard)
else ifeq ($(disk),windows)
qemu-deps: $(WINDOWS_DISK)
else
qemu-deps: $(EXTRA_DISK)
endif

qemu-deps:$(FIRMWARE)

qemu-deps:$(QEMU_KERNEL)

qemu-deps: $(PFLASH0)

ifneq ($(PFLASH1),)
qemu-deps: $(BUILD)/fw_vars.bin

.PRECIOUS: $(BUILD)/fw_vars.bin
$(BUILD)/fw_vars.bin: $(PFLASH1)
	cp "$<" "$@"
endif

$(EXTRA_DISK):
	truncate -s 1g $@

$(WINDOWS_DISK): $(BUILD)/harddrive.img
	rm -f $@
	mkdir -p $(shell dirname $@)
	cp "$<" "$@"

$(BUILD)/raspi3bp_uboot.rom:
	wget -O $@ https://gitlab.redox-os.org/Ivan/redox_firmware/-/raw/main/platform/raspberry_pi/rpi3/u-boot-rpi-3-b-plus.bin

$(BUILD)/qemu_uboot.rom:
	wget -O $@ https://gitlab.redox-os.org/Ivan/redox_firmware/-/raw/main/platform/qemu/qemu_arm64/u-boot-qemu-arm64.bin

qemu: qemu-deps
	$(QEMU) $(QEMUFLAGS)

# You probably want to use disk=no when using the *_extra targets
qemu_extra: qemu-deps
	$(QEMU) $(QEMUFLAGS) \
		-drive file=$(EXTRA_DISK),format=raw

qemu_nvme_extra: qemu-deps
	$(QEMU) $(QEMUFLAGS) \
		-drive file=$(EXTRA_DISK),format=raw,if=none,id=drv1 -device nvme,drive=drv1,serial=NVME_EXTRA

#additional steps for $(DISK) are required!!!
qemu_raspi: qemu-deps
	$(QEMU) -M raspi3b -smp 4,cores=1 \
		-kernel $(FIRMWARE) \
		-serial stdio -display none
