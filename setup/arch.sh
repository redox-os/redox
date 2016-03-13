#!/bin/sh

#Script designed to setup an arch linux computer to use Redox

#Install necessary packages
echo "Installing needed packages"
sudo pacman -S --needed base-devel nasm lib32-glibc glibc

#Install Virtual Box and Setup
echo "Installing virtualbox and it's dependencies"
sudo pacman -S --needed virtualbox virtualbox-host-dkms virtualbox-guest-iso qt4

#Load kernel modules and set for persistence
echo "Setting up virtualbox for persistence across reboots"
sudo modprobe vboxdrv
sudo mkdir -p /etc/modules-load.d
sudo touch /etc/modules-load.d/virtualbox.conf
#This hackish priviledge elevation seems to be the only way to get this to work
sudo su -c "echo vboxdrv > /etc/modules-load.d/virtualbox.conf"
sudo su -c "echo vboxpci >> /etc/modules-load.d/virtualbox.conf"

#Add user to vboxusers group
sudo gpasswd -a $USER vboxusers

echo "Script complete"

