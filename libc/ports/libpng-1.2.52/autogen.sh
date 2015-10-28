#! /bin/sh
# a quick hack script to generate necessary files from 
# auto* tools.
#
# WARNING: if you run this you will change the versions
# of the tools which are used and, maybe, required!
        touch Makefile.am configure.ac
{
	echo "running libtoolize" >&2
	libtoolize --force --copy --automake
} && {
	echo "running aclocal" >&2
	aclocal
} && {
	echo "running autoheader [ignore the warnings]" >&2
	autoheader
} && {
	echo "running automake" >&2
	automake --force-missing --foreign -a -c
} && {
	echo "running autoconf" >&2
	autoconf
} &&
	echo "autogen complete" >&2 ||
	echo "ERROR: autogen.sh failed, autogen is incomplete" >&2
