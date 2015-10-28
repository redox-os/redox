#!/bin/sh
#

set -e

aclocal -I acinclude
automake --foreign --include-deps --add-missing --copy
autoconf

#./configure $*
echo "Now you are ready to run ./configure"
