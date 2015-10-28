#!/bin/bash
source environ.sh

cd zlib-1.2.8
CC=i386-elf-redox-gcc ./configure --prefix="${PREFIX}" --static

make -j `nproc`
make -j `nproc` install
