#!/bin/bash
source environ.sh

UNSTABLE

SRC=http://downloads.sourceforge.net/project/dosbox/dosbox/0.74/dosbox-0.74.tar.gz
DIR=dosbox-0.74

export CXXFLAGS="-Os -static -T ${PWD}/../program.ld"
CONFIGURE_ARGS="--host=${HOST} --prefix=${PREFIX} --with-sdl-prefix=${PREFIX} --disable-opengl --disable-sdltest"
configure_template $*
