#!/bin/bash
source environ.sh

STABLE
DEPENDS zlib

SRC=http://download.sourceforge.net/libpng/libpng-1.2.53.tar.xz
DIR=libpng-1.2.53

CONFIGURE_ARGS="--host=${HOST} --disable-shared"
configure_template $*
