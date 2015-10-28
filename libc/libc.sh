#!/bin/bash
set -e

mkdir -p build

PREFIX="${PWD}/build/prefix"
mkdir -p "${PREFIX}"
mkdir -p "${PREFIX}/bin"
export PATH="${PREFIX}/bin:$PATH"

SYSROOT="${PWD}/build/sysroot"
mkdir -p "${SYSROOT}"

cd build

###################BINUTILS#########################
BINUTILS=binutils-2.24.90

if [ ! -f "${BINUTILS}.tar.bz2" ]
then
    curl "ftp://sourceware.org/pub/binutils/snapshots/${BINUTILS}.tar.bz2" -o "${BINUTILS}.tar.bz2"
fi

if [ ! -d "${BINUTILS}" ]
then
    tar xvf "${BINUTILS}.tar.bz2"
fi

cp -r ../binutils-redox/* "${BINUTILS}"

rm -rf "build-${BINUTILS}"
mkdir "build-${BINUTILS}"
pushd "build-${BINUTILS}"
    "../${BINUTILS}/configure" --target=i386-elf-redox --prefix="${PREFIX}" --with-sysroot="${SYSROOT}" --disable-nls --disable-werror
    make -j `nproc`
    make -j `nproc` install
popd

##################GCC FREESTANDING##############################
GCC=gcc-4.6.4

if [ ! -f "${GCC}.tar.bz2" ]
then
    curl "http://ftp.gnu.org/gnu/gcc/${GCC}/${GCC}.tar.bz2" -o "${GCC}.tar.bz2"
fi

if [ ! -d "${GCC}" ]
then
    tar xvf "${GCC}.tar.bz2"
    pushd "${GCC}"
        ./contrib/download_prerequisites
    popd
fi

cp -r ../gcc-redox/* "${GCC}"

pushd "${GCC}/libstdc++-v3"
    autoconf2.64
popd

rm -rf "build-freestanding-${GCC}"
mkdir "build-freestanding-${GCC}"
pushd "build-freestanding-${GCC}"
    "../${GCC}/configure" --target=i386-elf-redox --prefix="${PREFIX}" --disable-nls --enable-languages=c,c++ --without-headers
    make -j `nproc` all-gcc
    make -j `nproc` all-target-libgcc
    make -j `nproc` install-gcc
    make -j `nproc` install-target-libgcc
popd

##################NEWLIB###########################
NEWLIB=newlib-2.2.0.20150824

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
    "../${NEWLIB}/configure" --target=i386-elf-redox --prefix="${PREFIX}"
    make -j `nproc` all
    make -j `nproc` install
popd

mkdir -p "${SYSROOT}/usr"
cp -r "${PREFIX}/i386-elf-redox/include" "${SYSROOT}/usr"

######################GCC############################
rm -rf "build-${GCC}"
mkdir "build-${GCC}"
pushd "build-${GCC}"
    "../${GCC}/configure" --target=i386-elf-redox --prefix="${PREFIX}" --with-sysroot="${SYSROOT}" --disable-nls --enable-languages=c,c++
    make -j `nproc` all-gcc
    make -j `nproc` all-target-libgcc
    make -j `nproc` install-gcc
    make -j `nproc` install-target-libgcc
    make -j `nproc` all-target-libstdc++-v3
    make -j `nproc` install-target-libstdc++-v3
popd
