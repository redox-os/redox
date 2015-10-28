#!/bin/sh

#
# Remove the '-g' debug flag from Makefiles to build release versions
# of the libraries in one go.
#
# Run after './configure' and before 'make'.
#

echo "Removing debug flags from Makefile."
echo

TARGET="Makefile"

if [ "$1" != "" ]; then
 TARGET="$1"
fi

for i in `find . -name "$TARGET" -print`; do
 echo "Patching $i ..."
 cat $i | sed 's/-g -O2/-O -Wl,-s/' | sed 's/-shared/-shared -Wl,-s/' >$i.new
 cp -f $i.new $i
done
