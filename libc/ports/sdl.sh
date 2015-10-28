#!/bin/bash
source environ.sh

DIR=SDL-1.2.15
CONFIGURE_ARGS="--host=${HOST} --disable-shared --disable-audio --disable-cdrom --disable-joystick \
    --disable-loadso --disable-threads --disable-timers --disable-video-x11 --enable-video-orbital"
configure_template $*
