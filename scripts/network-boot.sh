#!/usr/bin/env bash

# Your host must use the static IP ${NETWORK}.1 and subnet mask 255.255.255.0
# 'Rx' in ascii is 82 and 120, adjust to taste
NETWORK=10.82.120

set -ex

trap 'kill -HUP 0' EXIT

eval $(make setenv)
make "${BUILD}/redox-live.iso"

echo "Allowing packet forwarding"
echo 1 | sudo tee /proc/sys/net/ipv4/ip_forward

iface="$(route | grep '^default ' | grep -o '[^ ]*$' | head -n 1)"
echo "Forwarding packets to '$iface'"
if ! sudo iptables -t nat -C POSTROUTING -o "$iface" -j MASQUERADE
then
    echo "Forwarding rule does not exist, adding"
    sudo iptables -t nat -A POSTROUTING -o "$iface" -j MASQUERADE
else
    echo "Forwarding rule already exists"
fi

ARGS=(
    "--no-daemon"
    "--bind-interfaces"
    "--listen-address=${NETWORK}.1"
    "--port=0"
    "--dhcp-range=${NETWORK}.3,${NETWORK}.254,255.255.255.0,1h"
    "--dhcp-option=6,1.1.1.1,1.0.0.1"
    "--enable-tftp"
    "--tftp-root=$(realpath "${BUILD}")"
    # BIOS
    "--dhcp-match=set:bios,option:client-arch,0"
    "--dhcp-boot=tag:!ipxe,tag:bios,undionly.kpxe"
    # EFI x86_64
    "--dhcp-match=set:efi-x86_64,option:client-arch,7"
    "--dhcp-match=set:efi-x86_64,option:client-arch,9"
    "--dhcp-boot=tag:!ipxe,tag:efi-x86_64,ipxe-x86_64.efi"
    # EFI aarch64
    "--dhcp-match=set:efi-aarch64,option:client-arch,11"
    "--dhcp-boot=tag:!ipxe,tag:efi-aarch64,ipxe-aarch64.efi"
    # IPXE
    "--dhcp-userclass=set:ipxe,iPXE"
    "--dhcp-boot=tag:ipxe,redox.ipxe"
)

sudo dnsmasq "${ARGS[@]}"&
python3 -m http.server -b "${NETWORK}.1" -d "${BUILD}" "8080"
