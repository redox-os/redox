#!/bin/bash
source environ.sh

SRC=https://www.libsdl.org/release/SDL-1.2.15.tar.gz
DIR=SDL-1.2.15

CONFIGURE_ARGS="--host=${HOST} --disable-shared --disable-audio --disable-cdrom --disable-loadso \
    --disable-threads --disable-timers --disable-video-x11 --enable-video-orbital"
autogen_template $*
