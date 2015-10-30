#!/bin/bash
source environ.sh

GIT=https://github.com/ebuc99/pacman.git
DIR=pacman

AUTOGEN_ARGS="--host=i386-elf-redox --prefix=$PREFIX --with-sdl-prefix=$PREFIX --disable-sdltest"
autogen_template $*
