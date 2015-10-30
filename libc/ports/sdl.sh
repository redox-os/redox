#!/bin/bash
source environ.sh

SRC=https://www.libsdl.org/release/SDL-1.2.15.tar.gz
DIR=SDL-1.2.15

CONFIGURE_ARGS="--host=${HOST} --disable-shared --disable-cdrom --disable-loadso --disable-threads \
    --disable-timers --disable-pulseaudio --disable-video-x11 \
    --enable-audio --enable-dummyaudio --enable-video-orbital"
autogen_template $*
