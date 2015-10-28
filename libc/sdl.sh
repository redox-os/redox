#!/bin/bash
source environ.sh

cd SDL-1.2.15
./configure --host=i386-elf-redox --prefix="${PREFIX}" --with-sysroot="${SYSROOT}" --disable-shared \
    --disable-audio --disable-cdrom --disable-events --disable-joystick --disable-threads \
    --disable-timers --disable-loadso --disable-video-dummy --disable-video-x11 \
    --enable-video-orbital

make -j `nproc`
make -j `nproc` install
