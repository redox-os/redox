#!/usr/bin/env bash

set -ex

rm -f build/libkernel.a build/kernel
touch kernel
touch kernel/src/arch/aarch64/init/pre_kstart/early_init.S
make build/kernel

mkimage \
	-A arm64 \
	-O "linux" \
	-T kernel \
	-C none \
	-a 0x40000000 \
	-e 0x40001000 \
	-n 'Redox kernel (qemu AArch64 virt)' \
	-d build/kernel \
   	build/kernel.uimage

qemu-system-aarch64 \
	-M virt \
	-cpu cortex-a57 \
	-bios u-boot.bin \
	-device loader,file=build/kernel.uimage,addr=0x41000000,force-raw=on \
	-serial mon:stdio \
	-nographic \
	-s
