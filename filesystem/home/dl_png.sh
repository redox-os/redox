#!/bin/sh
dhcpd
#dnsd static.redox-os.org
wget http://23.21.162.66:80/test.png static.redox-os.org > /home/test.png
launcher /home/test.png
#rm /home/test.png
