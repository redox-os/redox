#!/bin/sh
dhcpd
#dnsd static.redox-os.org
wget http://23.21.162.66:80/test.bmp static.redox-os.org > /home/test.bmp
launcher /home/test.bmp
#rm /home/test.bmp
