#!/bin/bash
export PATH=${PWD}/../build/prefix/bin:${PATH}
./configure --host=i386-elf-redox --disable-audio --disable-cdrom --disable-events --disable-joystick --disable-threads --disable-timers --disable-loadso --disable-shared --disable-video-dummy --disable-video-x11 -enable-video-orbital
make
i386-elf-redox-gcc -Os -static -T ../program.ld -o testbitmap.bin -I include/ test/testbitmap.c build/.libs/libSDL.a
