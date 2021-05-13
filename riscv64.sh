#!/usr/bin/env bash

set -ex

MACHINE=virt

U_BOOT="build/u-boot/${MACHINE}.bin"
IMAGE="build/kernel_live.uimage"
case "${MACHINE}" in
	virt)
		# UART at 0x9000000
		U_BOOT_CONFIG=qemu_riscv64_defconfig
		LOAD_ADDR=0x40000000
		ENTRY_ADDR=0x40001000
		IMAGE_ADDR=0x44000000
		QEMU_ARGS=(
			-M virt
			-m 1G
			-bios "${U_BOOT}"
			-device "loader,file=${IMAGE},addr=${IMAGE_ADDR},force-raw=on"
			-nographic
			-serial mon:stdio
			-s
		)
		;;
esac

if [ ! -f "${U_BOOT}" ]
then
	make prefix

	make -C u-boot distclean
	make -C u-boot "${U_BOOT_CONFIG}"

	sed -i \
		's/^CONFIG_BOOTCOMMAND=.*$/CONFIG_BOOTCOMMAND="bootm '"${IMAGE_ADDR}"' - ${fdtcontroladdr}"/' \
		u-boot/.config

	TARGET=riscv64-unknown-redox
	env CROSS_COMPILE="${TARGET}-" \
		PATH="${PWD}/prefix/${TARGET}/relibc-install/bin/:${PATH}" \
		make -C u-boot -j "$(nproc)"

	mkdir -pv build/u-boot
	cp -v u-boot/u-boot.bin "${U_BOOT}"
fi

mkdir -p build
# rm -f build/libkernel.a build/kernel
rm -f build/kernel
touch build/bootloader
touch kernel
make build/kernel
make build/initfs.tag
make build/filesystem.bin
make build/kernel_live

mkimage \
	-A riscv \
	-O linux \
	-T kernel \
	-C none \
	-a "${LOAD_ADDR}" \
	-e "${ENTRY_ADDR}" \
	-n "Redox kernel (riscv64 ${MACHINE})" \
	-d build/kernel_live \
   	"${IMAGE}"

if [ -n "${QEMU_ARGS}" ]
then
	qemu-system-riscv64 "${QEMU_ARGS[@]}" "$@"
else
	sudo cp -v "${IMAGE}" /srv/tftp
fi
