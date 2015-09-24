#!/bin/bash

set -e

NEWLIB=newlib-2.2.0.20150824

mkdir -p build
cd build

if [ ! -d i386-elf-redox ]
then
    mkdir i386-elf-redox
    pushd i386-elf-redox
        ln -s "`which ar`" i386-elf-redox-ar
        ln -s "`which gcc-4.6`" i386-elf-redox-gcc
        ln -s "`which ranlib`" i386-elf-redox-ranlib
        ln -s "`which readelf`" i386-elf-redox-readelf
    popd
fi

export PATH="${PWD}/i386-elf-redox:$PATH"

if [ ! -f "${NEWLIB}.tar.gz" ]
then
    curl "ftp://sourceware.org/pub/newlib/${NEWLIB}.tar.gz" -o "${NEWLIB}.tar.gz"
fi

if [ ! -d "${NEWLIB}" ]
then
    tar xvf "${NEWLIB}.tar.gz"
fi

cp -r ../newlib-redox/* "${NEWLIB}"

pushd "${NEWLIB}/newlib/libc/sys"
    aclocal-1.11 -I ../..
    autoconf
    automake-1.11 --cygnus Makefile
popd

pushd "${NEWLIB}/newlib/libc/sys/redox"
    aclocal-1.11 -I ../../..
    autoconf
    automake-1.11 --cygnus Makefile
popd

rm -rf "build-${NEWLIB}"
mkdir "build-${NEWLIB}"
pushd "build-${NEWLIB}"
    "../${NEWLIB}/configure" --build=i686-linux-gnu --target=i386-elf-redox "CFLAGS=-m32" "LDFLAGS=-m32"
    make all
popd
