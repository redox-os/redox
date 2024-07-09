# This script install Redox in the free space of your storage device
# and add a boot entry (if you are using the systemd-boot boot loader)

#!/usr/bin/env bash

set -e

if [ -n "$1" ]
then
    DISK="$1"
else
    DISK=/dev/disk/by-partlabel/REDOX_INSTALL
fi

if [ ! -b "${DISK}" ]
then
    echo "$0: '${DISK}' is not a block device" >&2
    exit 1
fi

if [ -z "${ARCH}" ]
then
    export ARCH=x86_64
fi

if [ -z "${CONFIG_NAME}" ]
then
    export CONFIG_NAME=demo
fi

IMAGE="build/${ARCH}/${CONFIG_NAME}/filesystem.img"
set -x
make "${IMAGE}"
sudo popsicle "${IMAGE}" "${DISK}"
set +x

ESP="$(bootctl --print-esp-path)"
if [ -z "${ESP}" ]
then
    echo "$0: no ESP found" >&2
    exit 1
fi

BOOTLOADER="cookbook/recipes/core/bootloader/target/${ARCH}-unknown-redox/stage/boot/bootloader.efi"
set -x
sudo mkdir -pv "${ESP}/EFI" "${ESP}/loader/entries"
sudo cp -v "${BOOTLOADER}" "${ESP}/EFI/redox.efi"
sudo tee "${ESP}/loader/entries/redox.conf" <<EOF
title Redox OS
efi /EFI/redox.efi
EOF
set +x

sync

echo "Finished installing Redox OS dual boot"
