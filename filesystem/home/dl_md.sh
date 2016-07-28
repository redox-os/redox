#!/bin/sh
dhcpd
wget http://static.redox-os.org/test.md > /home/redox_online.md
mdless /home/redox_online.md
rm /home/redox_online.md
