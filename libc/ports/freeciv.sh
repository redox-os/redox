#!/bin/bash
source environ.sh

SRC=http://sourceforge.net/projects/freeciv/files/Freeciv%202.0/2.0.4/freeciv-2.0.4.tar.bz2
DIR=freeciv-2.0.4

AUTOGEN_ARGS="--host=${HOST} --prefix=${PREFIX} --enable-client=sdl"
autogen_template $*
