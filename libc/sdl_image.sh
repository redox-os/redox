#!/bin/bash
source environ.sh

cd SDL_image-1.2.12
./configure --host=i386-elf-redox --prefix="${PREFIX}" --with-sysroot="${SYSROOT}" --disable-shared

make -j `nproc`
make -j `nproc` install
