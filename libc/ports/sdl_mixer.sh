#!/bin/bash
source environ.sh

UNSTABLE

SRC=https://www.libsdl.org/projects/SDL_mixer/release/SDL_mixer-1.2.12.tar.gz
DIR=SDL_mixer-1.2.12

CONFIGURE_ARGS="--host=${HOST} --disable-shared --disable-sdltest --disable-music-cmd --disable-smpegtest"
configure_template $*
