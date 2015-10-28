#!/bin/bash
source environ.sh

DIR=SDL_ttf-2.0.11
CONFIGURE_ARGS="--host=${HOST} --disable-shared --with-freetype-prefix=${PREFIX} --with-sdl-prefix=${PREFIX} \
    --disable-sdltest --without-harfbuzz"
configure_template $*
