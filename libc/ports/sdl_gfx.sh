#!/bin/bash
source environ.sh

STABLE

SRC=http://www.ferzkopp.net/Software/SDL_gfx-2.0/SDL_gfx-2.0.25.tar.gz
DIR=SDL_gfx-2.0.25

CONFIGURE_ARGS="--host=${HOST} --disable-shared --disable-sdltest"
configure_template $*
