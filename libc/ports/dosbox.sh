#!/bin/bash
source environ.sh

UNSTABLE

SRC=http://downloads.sourceforge.net/project/dosbox/dosbox/0.74/dosbox-0.74.tar.gz
DIR=dosbox-0.74

CONFIGURE_ARGS="--host=${HOST} --prefix=${PREFIX} --with-sdl-prefix=${PREFIX} --disable-opengl --disable-sdltest"
configure_template $*
