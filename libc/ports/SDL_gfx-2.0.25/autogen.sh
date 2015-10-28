#!/bin/sh
#

# Hack for MacPorts
cp /usr/local/share/aclocal/sdl.m4 m4

# Prep
make distclean
aclocal --force -I /usr/local/share/aclocal
libtoolize --force --copy
autoreconf -fvi
rm -rf autom4te.cache

# Setup
aclocal -I /usr/local/share/aclocal
autoheader
automake --foreign
autoconf

#./configure $*
echo "Now you are ready to run ./configure"
