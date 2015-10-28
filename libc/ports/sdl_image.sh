#!/bin/bash
source environ.sh

SRC=https://www.libsdl.org/projects/SDL_image/release/SDL_image-1.2.12.tar.gz
DIR=SDL_image-1.2.12

CONFIGURE_ARGS="--host=${HOST} --disable-shared"
configure_template $*
