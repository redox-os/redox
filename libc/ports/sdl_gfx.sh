#!/bin/bash
source environ.sh

DIR=SDL_gfx-2.0.25
CONFIGURE_ARGS="--host=${HOST} --disable-shared --disable-sdltest"
configure_template $*
