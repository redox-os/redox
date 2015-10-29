#!/bin/bash
source environ.sh

SRC=https://www.libsdl.org/projects/SDL_ttf/release/SDL_ttf-2.0.11.tar.gz
DIR=SDL_ttf-2.0.11

CONFIGURE_ARGS="--host=${HOST} --disable-shared --with-freetype-prefix=${PREFIX} --with-sdl-prefix=${PREFIX} \
    --disable-sdltest --without-harfbuzz"
configure_template $*
