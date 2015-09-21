#!/bin/bash
NEWLIB=newlib-2.2.0.20150824

rm -rf bin ${NEWLIB} build-${NEWLIB} ${NEWLIB}.tar.gz

mkdir bin
ln -s `which i386-elf-ar` bin/i386-elf-redox-ar
ln -s `which i386-elf-as` bin/i386-elf-redox-as
ln -s `which i386-elf-gcc` bin/i386-elf-redox-gcc
ln -s `which i386-elf-ld` bin/i386-elf-redox-ld
ln -s `which i386-elf-ranlib` bin/i386-elf-redox-ranlib

export PATH=$PATH:${PWD}/bin

curl ftp://sourceware.org/pub/newlib/${NEWLIB}.tar.gz -o ${NEWLIB}.tar.gz
tar xvf ${NEWLIB}.tar.gz

cp -r newlib-redox/* ${NEWLIB}

pushd ${NEWLIB}/newlib/libc/sys
autoconf264
cd redox
autoreconf264
popd

read -p "Verify"

mkdir build-${NEWLIB}
cd build-${NEWLIB}
../${NEWLIB}/configure --target=i386-elf-redox
make all
