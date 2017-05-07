#!/bin/bash
set -e

ISO="build/livedisk.iso"
DISK="/dev/disk/by-id/usb-Generic_USB_SD_Reader_12345678901234567890-0:0"

if [ ! -f "$ISO" ]
then
  echo "Did not find ISO $ISO"
  exit 1
fi

if [ ! -b "$DISK" ]
then
  echo "Did not find disk $DISK"
  exit 1
fi

echo "Flashing $ISO to $DISK"
pv "$ISO" | sudo dd of="$DISK"
sync
sudo eject "$DISK"
echo "Successfully flashed $DISK"
