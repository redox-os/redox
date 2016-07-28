#!/bin/sh
dhcpd
wget http://static.redox-os.org/test.bmp > /home/test.bmp
launcher /home/test.bmp
#rm /home/test.bmp
