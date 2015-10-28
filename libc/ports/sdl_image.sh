#!/bin/bash
source environ.sh

DIR=SDL_image-1.2.12
CONFIGURE_ARGS="--host=${HOST} --disable-shared"
configure_template $*
